// R1: bounded authenticated receive-service experiment, not a socket or decoder benchmark.
// Nested in nvst::tests solely to reuse its private SRTP/RTP fixtures unchanged.
use super::*;
use serde_json::{Value, json};

const PERIOD_NS: u64 = 1_000_000_000 / 60;
const WARMUP: usize = 30;
const MEASURED: usize = 600;
const DATAGRAM_BYTES: usize = 1_200;
const PLAINTEXT_BYTES: usize = DATAGRAM_BYTES - SRTP_AEAD_AES_GCM_8_TAG_LEN;
const MEDIA_BYTES: usize = PLAINTEXT_BYTES - 32;
// Cross RTP rollover even in the small correctness fixture; benchmark size stays fixed.
const FIRST_SEQUENCE: u64 = 65_500;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Case {
    Paced,
    Burst,
    Repair,
}

#[derive(Clone, Copy)]
struct PacketIdentity {
    shard: usize,
    parity: bool,
    rtp_sequence: u16,
    rtp_index: u64,
    fixture_ordinal: usize,
}

struct ReplayPacket {
    identity: PacketIdentity,
    arrival_ns: u64,
    source: SocketAddr,
    datagram: Vec<u8>,
    frame: usize,
}

struct Fixture {
    packets: Vec<ReplayPacket>,
    payloads: Vec<Vec<u8>>,
    data_shards: usize,
    parity_shards: usize,
    repaired_frames: usize,
}

#[derive(Clone, Copy)]
struct PacketCost {
    replay_ordinal: u64,
    elapsed_ns: u64,
    process_ns: u64,
    forward_ns: u64,
    service_ns: u64,
    bracketed_ns: u64,
    inner_split_selected: bool,
    authenticated_delta: u64,
    frames_forwarded: u64,
    repaired_delta: u64,
}

// Process-global ordering of actual receive replays, not a shipping counter.
static REPLAY_ORDINAL: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

struct UnprotectCost {
    elapsed_ns: u64,
    unprotect_ns: u64,
}

fn replay_config() -> NvstVideoConfig {
    let mut config = config();
    config.codec = NvstVideoCodec::H265;
    config.max_access_unit_bytes = 512 * 1_024;
    config.video_packet_size = PLAINTEXT_BYTES - NVST_FEC_RTP_HEADER_ALLOWANCE;
    config
}

fn fixture(mbps: u64, case: Case, frames: usize) -> Fixture {
    let wire_packets = (mbps * 1_000_000).div_ceil(60 * DATAGRAM_BYTES as u64 * 8) as usize;
    // A 1% repair field yields one/two parity shards at these offered loads.
    // Reuse the exact Cauchy coefficients and envelope restoration of the existing FEC tests.
    let data_shards = if case == Case::Repair {
        (1..wire_packets)
            .find(|n| n + n.div_ceil(100) == wire_packets)
            .unwrap()
    } else {
        wire_packets
    };
    let parity_shards = if case == Case::Repair {
        data_shards.div_ceil(100)
    } else {
        0
    };
    let crypto = test_srtp(&replay_config());
    let mut packets = Vec::with_capacity(frames * wire_packets);
    let mut payloads = Vec::with_capacity(frames);
    let mut sequence = FIRST_SEQUENCE;
    let mut repaired_frames = 0;
    for frame in 0..frames {
        let repair = case == Case::Repair && frame % 60 == 0;
        repaired_frames += usize::from(repair);
        let frame_index = frame as u32 + 1;
        let mut expected = Vec::with_capacity(data_shards * MEDIA_BYTES);
        let mut data = Vec::with_capacity(data_shards);
        for shard in 0..data_shards {
            let mut media = (0..MEDIA_BYTES)
                .map(|byte| 0x80 | ((frame * 13 + shard * 29 + byte * 7) & 0x7f) as u8)
                .collect::<Vec<_>>();
            if shard == 0 {
                media[..6].copy_from_slice(&[
                    0,
                    0,
                    0,
                    1,
                    if frame % 60 == 0 { 0x26 } else { 0x02 },
                    1,
                ]);
            }
            expected.extend_from_slice(&media);
            let flags = FLAG_CONTAINS_PIC_DATA
                | if shard == 0 { FLAG_SOF } else { 0 }
                | if shard + 1 == data_shards {
                    FLAG_EOF
                } else {
                    0
                };
            let mut plain =
                build_plaintext_rtp((sequence + shard as u64) as u16, flags, frame_index, &media);
            plain[4..8].copy_from_slice(&(frame_index * 1_500).to_be_bytes());
            // Stream identity is sender-authored and must survive RTP rollover.
            plain[16..20].copy_from_slice(&(((sequence + shard as u64) as u32) << 8).to_le_bytes());
            if parity_shards != 0 {
                let fec_word = (1_u32 << 4) | ((shard as u32) << 12) | ((data_shards as u32) << 22);
                plain[28..32].copy_from_slice(&fec_word.to_le_bytes());
            }
            data.push(plain);
        }
        let mut all = data.clone();
        for parity_index in 0..parity_shards {
            let mut parity = vec![0_u8; PLAINTEXT_BYTES];
            for (data_index, shard) in data.iter().enumerate() {
                gf256_axpy(
                    &mut parity,
                    shard,
                    nvst_cauchy_coefficient(data_index, parity_index, parity_shards).unwrap(),
                );
            }
            // As in fec_block_reconstructs_a_missing_shard_inside_a_multiblock_frame:
            // parity has its own valid RTP envelope and transported fecInfo.
            parity[..16].copy_from_slice(&data[0][..16]);
            parity[2..4].copy_from_slice(
                &((sequence + data_shards as u64 + parity_index as u64) as u16).to_be_bytes(),
            );
            parity[20..24].copy_from_slice(&frame_index.to_le_bytes());
            parity[27] = 0;
            let fec_word = (1_u32 << 4)
                | (((data_shards + parity_index) as u32) << 12)
                | ((data_shards as u32) << 22);
            parity[28..32].copy_from_slice(&fec_word.to_le_bytes());
            all.push(parity);
        }
        for (shard, plain) in all.into_iter().enumerate() {
            if repair && shard == data_shards / 2 {
                continue;
            }
            let offset = if repair && shard >= data_shards {
                2_000_000 + (shard - data_shards) as u64 * 1_000
            } else if case == Case::Paced {
                shard as u64 * PERIOD_NS / wire_packets as u64
            } else {
                shard as u64 * 1_000_000 / wire_packets as u64
            };
            let datagram =
                protect_for_test(&crypto, plain, ((sequence + shard as u64) >> 16) as u32);
            assert_eq!(datagram.len(), DATAGRAM_BYTES);
            let rtp_index = sequence + shard as u64;
            let rtp_sequence = RtpHeader::parse(&datagram).unwrap().sequence_number;
            assert_eq!(rtp_sequence, rtp_index as u16);
            packets.push(ReplayPacket {
                identity: PacketIdentity {
                    shard,
                    parity: shard >= data_shards,
                    rtp_sequence,
                    rtp_index,
                    fixture_ordinal: packets.len(),
                },
                arrival_ns: frame as u64 * PERIOD_NS + offset,
                source: peer(),
                datagram,
                frame,
            });
        }
        payloads.push(expected);
        sequence += wire_packets as u64;
    }
    assert!(
        packets
            .windows(2)
            .all(|pair| pair[0].arrival_ns <= pair[1].arrival_ns)
    );
    Fixture {
        packets,
        payloads,
        data_shards,
        parity_shards,
        repaired_frames,
    }
}

#[test]
#[ignore = "bounded diagnostic hook overhead report; not target receive evidence"]
fn receive_diagnostic_hook_cost_report() {
    assert!(
        !cfg!(debug_assertions),
        "diagnostic overhead requires an optimized release build"
    );
    let fixture = fixture(100, Case::Repair, 630);
    let mut gate_costs = Vec::with_capacity(3);
    let mut gate_p99 = Vec::with_capacity(3);
    let mut legacy_costs = Vec::with_capacity(3);
    for (pair, instrumented_first) in [false, true, false].into_iter().enumerate() {
        let first = replay(&fixture, 30, instrumented_first);
        let second = replay(&fixture, 30, !instrumented_first);
        let (ordinary, instrumented) = if instrumented_first {
            (second, first)
        } else {
            (first, second)
        };
        let ordinary_total = ordinary.iter().map(|c| c.service_ns).sum::<u64>();
        let instrumented_total = instrumented.iter().map(|c| c.service_ns).sum::<u64>();
        let ordinary_bracketed = ordinary.iter().map(|c| c.bracketed_ns).sum::<u64>();
        let instrumented_bracketed = instrumented.iter().map(|c| c.bracketed_ns).sum::<u64>();
        let ordinary_distribution = distribution(ordinary.iter().map(|c| c.bracketed_ns));
        let instrumented_distribution = distribution(instrumented.iter().map(|c| c.bracketed_ns));
        let increase = |after: f64, before: f64| 100.0 * (after / before - 1.0);
        let symmetric = increase(instrumented_bracketed as f64, ordinary_bracketed as f64);
        let p99 = increase(
            instrumented_distribution["p99_us"].as_f64().unwrap(),
            ordinary_distribution["p99_us"].as_f64().unwrap(),
        );
        let legacy = increase(instrumented_bracketed as f64, ordinary_total as f64);
        gate_costs.push(symmetric);
        gate_p99.push(p99);
        legacy_costs.push(legacy);
        println!(
            "RECEIVE_DIAGNOSTIC_OVERHEAD {}",
            json!({
                "pair": pair + 1, "instrumented_first":instrumented_first,
                "packets_per_pass":ordinary.len(), "frames_per_pass":600,
                "selected_packets_measured":instrumented.iter().filter(|c|c.inner_split_selected).count(),"inner_stride":16,
                "ordinary_total_ns":ordinary_total,"instrumented_total_ns":instrumented_total,
                "total_increase_percent":100.0 * (instrumented_total as f64 / ordinary_total as f64 - 1.0),
                "ordinary_bracketed_ns":ordinary_bracketed,"instrumented_bracketed_ns":instrumented_bracketed,
                "symmetric_increase_percent":symmetric,"symmetric_p99_change_percent":p99,
                "asymmetric_legacy_increase_percent":legacy,
                "ordinary_bracketed":ordinary_distribution,"instrumented_bracketed":instrumented_distribution,
                "bracket":"Admission: loop-top through post-completion bracket-end in BOTH modes; same common reads. R1 service: finished-start. Legacy structure: instrumented bracket versus ordinary R1 service, informational only; not a historical rescore or identical historical workload.",
                "ordinary_service":distribution(ordinary.iter().map(|c|c.service_ns)),
                "instrumented_service":distribution(instrumented.iter().map(|c|c.service_ns)),
            "linux_bin_cpu_and_socket_hooks":cfg!(all(target_os="linux",feature="receive-diagnostics")),
            "limits":"H3 schema3 fixed1/16-inner/full-outer hook cost only; no recv syscall, exporter or live scheduling. Both modes include common loop-top/process/processed/finished/bracket-end reads; Linux exercises actual per-bin CPU and SO_MEMINFO on idle loopback. Unsampled inner phases not localized or extrapolated; wall-only phases; CPU snapshots do not assign per-span cause. Clock dominance unproved. Passing admits one capture, not a gain or historical FAIL rescore."
            })
        );
    }
    let median = |mut values: Vec<f64>| {
        values.sort_by(f64::total_cmp);
        values[1]
    };
    let cost = median(gate_costs);
    let p99 = median(gate_p99);
    let legacy = median(legacy_costs);
    println!(
        "RECEIVE_DIAGNOSTIC_ADMISSION {}",
        json!({"schema":3,"pairs":3,
        "median_symmetric_increase_percent":cost,"median_symmetric_p99_change_percent":p99,
        "median_asymmetric_legacy_increase_percent":legacy,"cost_threshold_percent":5.0,"p99_threshold_percent":5.0,
        "verdict":if cost <= 5.0 && p99 <= 5.0 { "PASS" } else { "FAIL" },
        "limits":"Prospective H3 gate only; validity assertions mandatory; no row-bearing retry or historical rescoring."})
    );
}

fn nanos(duration: Duration) -> u64 {
    duration.as_nanos().try_into().unwrap()
}

fn validate_frame(frame: &EncodedMediaFrame, fixture: &Fixture, index: usize, contiguous: bool) {
    assert_eq!(frame.frame_index, Some(index as u32 + 1));
    assert_eq!(frame.rtp_timestamp, (index as u64 + 1) * 1_500);
    assert_eq!(frame.codec, "H265");
    assert_eq!(frame.clock_rate_hz, 90_000);
    assert_eq!(frame.keyframe, index % 60 == 0);
    assert_eq!(frame.contiguous, contiguous);
    assert_eq!(frame.payload.as_ref(), fixture.payloads[index]);
    // received_at_us intentionally omitted: forward_receive_event uses real elapsed time.
}

fn replay(fixture: &Fixture, warmup: usize, diagnostic_hooks: bool) -> Vec<PacketCost> {
    let replay_ordinal = REPLAY_ORDINAL.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
    let config = replay_config();
    let feedback = config.feedback();
    let mut receiver = NvstVideoReceiver::new(config);
    let (consumer, output) = mpsc::sync_channel(8);
    let (sender, events) = mpsc::channel();
    let origin = Instant::now();
    let mut gap = false;
    let mut frame_count = 0;
    let mut diagnostic = diagnostic_hooks.then(super::super::receive_diagnostics::Capture::new);
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    let diagnostic_socket = diagnostic_hooks.then(|| UdpSocket::bind("127.0.0.1:0").unwrap());
    let mut costs = Vec::with_capacity(fixture.packets.len());
    for (ordinal, packet) in fixture.packets.iter().enumerate() {
        let now = origin + Duration::from_nanos(packet.arrival_ns);
        let accepted = receiver.authenticated_packets;
        let repaired = receiver.fec_repaired_packets;
        // Identical common reads/brackets in both modes; runtime already has loop-top/end clocks.
        let loop_top = Instant::now();
        #[cfg(not(all(target_os = "linux", feature = "receive-diagnostics")))]
        let sampled = None;
        #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
        let sampled = diagnostic_socket.as_ref().and_then(|socket| {
            super::super::receive_diagnostics::Capture::checkpoint(
                &mut diagnostic,
                socket,
                loop_top,
            )
        });
        let hook_start = diagnostic.as_mut().map(|d| {
            let entered = d.receive_start(loop_top, sampled, false);
            d.receive_returned(entered, Some(packet.datagram.len()), false)
        });
        let split_selected = diagnostic.as_ref().is_some_and(|_| {
            super::super::receive_diagnostics::Capture::split_selected(ordinal as u64 + 1)
        });
        let start = Instant::now();
        let received = receiver.process_datagram(packet.source, &packet.datagram, now);
        let processed = Instant::now();
        let hook_processed = diagnostic
            .as_mut()
            .map(|d| d.process_returned(hook_start.unwrap(), start, split_selected))
            .unwrap_or(super::super::receive_diagnostics::ProcessTiming::NotCalled);
        let mut forwarded = 0;
        for event in received {
            forwarded += u64::from(matches!(event, NvstReceiveEvent::Frame(_)));
            assert!(forward_receive_event(
                &consumer, &sender, &feedback, origin, now, &mut gap, event
            ));
        }
        let finished = Instant::now();
        if let Some(d) = diagnostic.as_mut() {
            d.complete_iteration(
                finished,
                hook_start.unwrap(),
                true,
                hook_processed,
                split_selected,
                [
                    receiver.authenticated_packets,
                    receiver.frames_emitted,
                    receiver.fec_repaired_packets,
                    0,
                    0,
                    0,
                ],
            );
        }
        let bracket_end = Instant::now();
        let cost = PacketCost {
            replay_ordinal,
            elapsed_ns: nanos(start - origin),
            process_ns: nanos(processed - start),
            forward_ns: nanos(finished - processed),
            service_ns: nanos(finished - start), // R1 definition unchanged in both modes.
            bracketed_ns: nanos(bracket_end - loop_top),
            inner_split_selected: split_selected,
            authenticated_delta: receiver.authenticated_packets - accepted,
            frames_forwarded: forwarded,
            repaired_delta: receiver.fec_repaired_packets - repaired,
        };
        // All semantic checking, admission feedback and draining stay outside timed service.
        assert_eq!(cost.service_ns, cost.process_ns + cost.forward_ns);
        assert!(cost.bracketed_ns >= cost.service_ns);
        assert_eq!(
            cost.authenticated_delta, 1,
            "every timed packet must authenticate"
        );
        assert!(
            events.try_recv().is_err(),
            "unexpected drop/recovery/progress event"
        );
        while let Ok(frame) = output.try_recv() {
            validate_frame(&frame, fixture, frame_count, true);
            feedback.publish_accepted_frame(
                frame.frame_index.unwrap(),
                frame.payload.len() as u32,
                now,
            );
            frame_count += 1;
        }
        assert!(!gap);
        if packet.frame >= warmup {
            costs.push(cost);
        }
    }
    assert_eq!(frame_count, fixture.payloads.len());
    assert_eq!(receiver.frames_emitted, frame_count as u64);
    assert_eq!(
        receiver.fec_repaired_packets,
        fixture.repaired_frames as u64
    );
    assert_eq!(receiver.authenticated_packets, fixture.packets.len() as u64);
    assert_eq!(receiver.replay_rejections, 0);
    assert_eq!(receiver.packet_gap_recoveries, 0);
    assert_eq!(receiver.stale_packets, 0);
    let stage = feedback.frame_stage_timings();
    assert_eq!(stage.admitted_frames_total, frame_count as u64);
    assert_eq!(stage.undelivered_frames_total, 0);
    assert_eq!(stage.pending_deliveries, 0);
    // Observe and validate accumulator work after all timed packet service.
    if diagnostic_hooks {
        let capture = std::hint::black_box(diagnostic.as_ref())
            .expect("instrumented fixture capture disappeared");
        capture.assert_fixture(
            fixture.packets.len() as u64,
            frame_count as u64,
            fixture.repaired_frames as u64,
            (fixture.packets.len() as u64)
                .div_ceil(super::super::receive_diagnostics::SPLIT_STRIDE),
        );
        let expected_selected = fixture
            .packets
            .iter()
            .enumerate()
            .filter(|(i, packet)| packet.frame >= warmup && i.is_multiple_of(16))
            .count();
        let actual_selected = costs.iter().filter(|c| c.inner_split_selected).count();
        assert_eq!(actual_selected, expected_selected);
        println!(
            "RECEIVE_DIAGNOSTIC_SELECTED_COVERAGE measured_packets={} measured_selected={} expected_selected={} includes_warmup_policy=true",
            costs.len(),
            actual_selected,
            expected_selected
        );
    }
    costs
}

fn traversal(fixture: &Fixture, warmup: usize) -> Vec<PacketCost> {
    let origin = Instant::now();
    let mut costs = Vec::with_capacity(fixture.packets.len());
    for packet in &fixture.packets {
        let start = Instant::now();
        std::hint::black_box((packet.source, packet.datagram.as_slice(), packet.arrival_ns));
        let processed = Instant::now();
        std::hint::black_box(());
        let finished = Instant::now();
        let cost = PacketCost {
            replay_ordinal: 0,
            elapsed_ns: nanos(start - origin),
            process_ns: nanos(processed - start),
            forward_ns: nanos(finished - processed),
            service_ns: nanos(finished - start),
            bracketed_ns: nanos(finished - start), // Traversal is not diagnostic admission.
            inner_split_selected: false,
            authenticated_delta: 0,
            frames_forwarded: 0,
            repaired_delta: 0,
        };
        if packet.frame >= warmup {
            costs.push(cost);
        }
    }
    costs
}

// Independent pass over identical protected bytes, with fresh SRTP/replay/ROC state.
// It is NOT an in-place subphase measurement of process_datagram: no subtraction.
fn independent_unprotect(fixture: &Fixture, warmup: usize) -> Vec<UnprotectCost> {
    let mut receiver = NvstVideoReceiver::new(replay_config());
    let mut costs = Vec::with_capacity(fixture.packets.len());
    let origin = Instant::now();
    for packet in &fixture.packets {
        let start = Instant::now();
        let unprotected = receiver.srtp.unprotect(&packet.datagram);
        let finished = Instant::now();
        // Validate and drop plaintext only after the call's measured interval.
        let unprotected = unprotected.expect("independent pass must authenticate every packet");
        assert_eq!(unprotected.index, packet.identity.rtp_index);
        assert_eq!(
            unprotected.header.sequence_number,
            packet.identity.rtp_sequence
        );
        if packet.frame >= warmup {
            costs.push(UnprotectCost {
                elapsed_ns: nanos(start - origin),
                unprotect_ns: nanos(finished - start),
            });
        }
    }
    costs
}

// Five-element ranking and metadata/report construction are outside all timed calls.
fn slowest_indices(values: impl Iterator<Item = u64>) -> Vec<usize> {
    let mut top = Vec::<(usize, u64)>::with_capacity(6);
    for (index, duration) in values.enumerate() {
        let position = top
            .iter()
            .position(|sample| duration > sample.1)
            .unwrap_or(top.len());
        if position < 5 {
            top.insert(position, (index, duration));
            if top.len() > 5 {
                top.pop();
            }
        }
    }
    top.into_iter().map(|sample| sample.0).collect()
}

fn packet_identity_json(packet: &ReplayPacket, sample_index: usize) -> Value {
    json!({"sample_index":sample_index, "fixture_packet_ordinal":packet.identity.fixture_ordinal,
        "fixture_frame":packet.frame, "frame_index":packet.frame as u32+1,
        "shard":packet.identity.shard, "parity":packet.identity.parity,
        "rtp_sequence":packet.identity.rtp_sequence, "rtp_index":packet.identity.rtp_index,
        "scripted_arrival_offset_ns":packet.arrival_ns})
}

#[derive(Debug, PartialEq)]
struct Backlog {
    waiting_packets: usize,
    waiting_bytes: u64,
    max_wait_ns: u64,
    max_boundary_lag_ns: u64,
    growing_three_periods: bool,
    boundary_waiting: Vec<(usize, usize, u64)>,
}

// Completion/start at the same timestamp precedes arrival; the in-service packet is excluded.
// This no-loss service-demand model does NOT replay protocol decisions with delayed timestamps.
fn estimate_backlog(
    arrivals: &[u64],
    lengths: &[usize],
    service: &[u64],
    frames: &[usize],
) -> Backlog {
    assert_eq!(arrivals.len(), service.len());
    assert_eq!(arrivals.len(), lengths.len());
    assert_eq!(arrivals.len(), frames.len());
    let mut starts = Vec::with_capacity(arrivals.len());
    let mut prefix = Vec::with_capacity(arrivals.len() + 1);
    prefix.push(0_u64);
    let mut frame_finishes = Vec::<(usize, u64)>::new();
    let mut finish = 0;
    let mut begun = 0;
    let mut result = Backlog {
        waiting_packets: 0,
        waiting_bytes: 0,
        max_wait_ns: 0,
        max_boundary_lag_ns: 0,
        growing_three_periods: false,
        boundary_waiting: Vec::with_capacity(frame_finishes.capacity()),
    };
    for (i, &arrival) in arrivals.iter().enumerate() {
        assert!(i == 0 || arrival >= arrivals[i - 1]);
        let start = arrival.max(finish);
        starts.push(start);
        prefix.push(prefix[i] + lengths[i] as u64);
        finish = start + service[i];
        while begun <= i && starts[begun] <= arrival {
            begun += 1;
        }
        result.waiting_packets = result.waiting_packets.max(i + 1 - begun);
        result.waiting_bytes = result.waiting_bytes.max(prefix[i + 1] - prefix[begun]);
        result.max_wait_ns = result.max_wait_ns.max(start - arrival);
        if let Some((frame, last)) = frame_finishes.last_mut()
            && *frame == frames[i]
        {
            *last = finish;
        } else {
            frame_finishes.push((frames[i], finish));
        }
    }
    let mut arrived = 0;
    let mut started = 0;
    let mut previous = None;
    let mut growth = 0;
    for (frame, finish) in frame_finishes {
        let boundary = (frame as u64 + 1) * PERIOD_NS;
        let lag = finish.saturating_sub(boundary);
        result.max_boundary_lag_ns = result.max_boundary_lag_ns.max(lag);
        // Sample after completions/starts at this boundary, but before the next
        // frame's equal-time arrivals. Exclude the packet already in service.
        while arrived < arrivals.len() && arrivals[arrived] < boundary {
            arrived += 1;
        }
        while started < arrived && starts[started] <= boundary {
            started += 1;
        }
        let waiting_packets = arrived - started;
        let waiting_bytes = prefix[arrived] - prefix[started];
        result
            .boundary_waiting
            .push((frame, waiting_packets, waiting_bytes));
        // Fixed-size replay datagrams make packet/byte growth equivalent.
        // Require three increasing intervals (four boundaries), not lag jitter.
        growth = if previous.is_some_and(|bytes| waiting_bytes > bytes) {
            growth + 1
        } else {
            0
        };
        result.growing_three_periods |= growth >= 3;
        previous = Some(waiting_bytes);
    }
    result
}

fn backlog_json(value: &Backlog) -> Value {
    json!({ "max_waiting_packets": value.waiting_packets, "max_waiting_datagram_bytes": value.waiting_bytes,
        "max_wait_us": value.max_wait_ns as f64 / 1_000.0,
        "max_frame_service_after_next_boundary_us": value.max_boundary_lag_ns as f64 / 1_000.0,
        "max_boundary_waiting_packets":value.boundary_waiting.iter().map(|sample| sample.1).max().unwrap_or(0),
        "max_boundary_waiting_datagram_bytes":value.boundary_waiting.iter().map(|sample| sample.2).max().unwrap_or(0),
        "growth_definition":"waiting bytes increase strictly over three consecutive intervals/four boundaries; fixed1200B packets",
        "grows_three_consecutive_periods": value.growing_three_periods,
        "boundary_sampling":"end of frame; completions/starts before sampling; equal-time next-frame arrivals after sampling; in-service packet excluded",
        "boundary_waiting_samples":value.boundary_waiting.iter().map(|&(frame, packets, bytes)|
            json!({"end_frame":frame,"boundary_ns":(frame as u64+1)*PERIOD_NS,"waiting_packets":packets,"waiting_datagram_bytes":bytes})).collect::<Vec<_>>() })
}

fn distribution(values: impl Iterator<Item = u64>) -> Value {
    let mut values = values.collect::<Vec<_>>();
    if values.is_empty() {
        return Value::Null;
    }
    values.sort_unstable();
    let quantile =
        |percent: usize| values[(values.len() * percent).div_ceil(100) - 1] as f64 / 1_000.0;
    json!({"samples":values.len(), "p50_us":quantile(50), "p95_us":quantile(95),
        "p99_us":quantile(99), "max_us":values.last().unwrap().to_owned() as f64 / 1_000.0,
        "total_ms":values.iter().sum::<u64>() as f64 / 1_000_000.0})
}

#[test]
fn authenticated_frames_survive_rollover_reorder_and_real_fec_repair() {
    // First frame crosses rollover and reconstructs a missing middle shard.
    // The ignored report, not this default-suite semantic check, owns the full fixed matrix.
    let repair = fixture(100, Case::Repair, 3);
    let costs = replay(&repair, 0, false);
    assert_eq!(costs.iter().map(|cost| cost.repaired_delta).sum::<u64>(), 1);
    assert_eq!(
        costs.iter().map(|cost| cost.frames_forwarded).sum::<u64>(),
        3
    );
    let instrumented = replay(&repair, 0, true);
    assert_eq!(
        instrumented
            .iter()
            .map(|cost| cost.repaired_delta)
            .sum::<u64>(),
        1
    );
    assert_eq!(
        instrumented
            .iter()
            .map(|cost| cost.frames_forwarded)
            .sum::<u64>(),
        3
    );
    assert_eq!(
        instrumented
            .iter()
            .map(|cost| cost.authenticated_delta)
            .sum::<u64>(),
        repair.packets.len() as u64
    );
    assert!(
        instrumented
            .iter()
            .all(|cost| cost.bracketed_ns >= cost.service_ns
                && cost.service_ns == cost.process_ns + cost.forward_ns)
    );
    let mut reordered = fixture(50, Case::Burst, 3);
    // Reorder packet contents, not the monotonic arrival clock.
    let (first, rest) = reordered.packets.split_at_mut(2);
    std::mem::swap(&mut first[1].datagram, &mut rest[0].datagram);
    std::mem::swap(&mut first[1].identity, &mut rest[0].identity);
    let costs = replay(&reordered, 0, false);
    assert_eq!(
        costs.iter().map(|cost| cost.frames_forwarded).sum::<u64>(),
        3
    );
}

#[test]
fn real_receive_handoff_preserves_eight_frame_backpressure_gap_and_closure() {
    let fixture = fixture(1, Case::Burst, 12);
    let config = replay_config();
    let feedback = config.feedback();
    let mut receiver = NvstVideoReceiver::new(config);
    let (consumer, output) = mpsc::sync_channel(8);
    let (sender, events) = mpsc::channel();
    let origin = Instant::now();
    let mut gap = false;
    let mut output = Some(output);
    let mut closure_seen = false;
    for (position, packet) in fixture.packets.iter().enumerate() {
        let now = origin + Duration::from_nanos(packet.arrival_ns);
        for event in receiver.process_datagram(packet.source, &packet.datagram, now) {
            let keep_running =
                forward_receive_event(&consumer, &sender, &feedback, origin, now, &mut gap, event);
            if packet.frame == 11 {
                assert!(!keep_running);
                closure_seen = true;
            } else {
                assert!(keep_running);
            }
        }
        let frame_end = fixture
            .packets
            .get(position + 1)
            .is_none_or(|next| next.frame != packet.frame);
        if !frame_end {
            continue;
        }
        if packet.frame == 8 {
            assert!(gap);
            assert!(matches!(
                events.try_recv().unwrap(),
                NvstReceiveEvent::Dropped(NvstDropReason::MediaConsumerBackpressured)
            ));
            let output = output.as_ref().unwrap();
            for index in 0..8 {
                let frame = output.try_recv().unwrap();
                validate_frame(&frame, &fixture, index, true);
                feedback.publish_accepted_frame(
                    frame.frame_index.unwrap(),
                    frame.payload.len() as u32,
                    now,
                );
            }
            assert!(output.try_recv().is_err());
        } else if packet.frame == 9 || packet.frame == 10 {
            let frame = output.as_ref().unwrap().try_recv().unwrap();
            validate_frame(&frame, &fixture, packet.frame, packet.frame == 10);
            feedback.publish_accepted_frame(
                frame.frame_index.unwrap(),
                frame.payload.len() as u32,
                now,
            );
            assert!(!gap);
            if packet.frame == 10 {
                drop(output.take());
            }
        }
    }
    assert!(closure_seen);
    assert!(matches!(
        events.try_recv().unwrap(),
        NvstReceiveEvent::Dropped(NvstDropReason::MediaConsumerClosed)
    ));
    assert!(events.try_recv().is_err());
    assert_eq!(receiver.authenticated_packets, fixture.packets.len() as u64);
    let stage = feedback.frame_stage_timings();
    assert_eq!(stage.assembled_frames_total, 12);
    assert_eq!(stage.admitted_frames_total, 10);
    assert_eq!(stage.undelivered_frames_total, 2);
    assert_eq!(stage.pending_deliveries, 0);
}

#[test]
fn backlog_models_pacing_burst_overload_and_long_service_with_equal_time_ordering() {
    let run = |a: &[u64], s: &[u64]| estimate_backlog(a, &vec![100; a.len()], s, &vec![0; a.len()]);
    assert_eq!(
        run(&[0, 10, 20], &[10, 10, 10]),
        Backlog {
            waiting_packets: 0,
            waiting_bytes: 0,
            max_wait_ns: 0,
            max_boundary_lag_ns: 0,
            growing_three_periods: false,
            boundary_waiting: vec![(0, 0, 0)],
        }
    );
    let burst = run(&[0, 0, 0, 30], &[10, 10, 10, 10]);
    assert_eq!(
        (
            burst.waiting_packets,
            burst.waiting_bytes,
            burst.max_wait_ns
        ),
        (2, 200, 20)
    );
    let long = run(&[0, 10, 20, 30, 100], &[100, 1, 1, 1, 1]);
    assert_eq!(
        (long.waiting_packets, long.waiting_bytes, long.max_wait_ns),
        (3, 300, 90)
    );
    // Regression: later frame completions can creep past each boundary while
    // only one packet is in service and the waiting queue stays empty.
    let lag_only = estimate_backlog(
        &[
            PERIOD_NS - 100,
            PERIOD_NS * 2 - 100,
            PERIOD_NS * 3 - 100,
            PERIOD_NS * 4 - 100,
        ],
        &[100; 4],
        &[110, 120, 130, 140],
        &[0, 1, 2, 3],
    );
    assert_eq!(lag_only.max_boundary_lag_ns, 40);
    assert_eq!(lag_only.waiting_packets, 0);
    assert_eq!(
        lag_only.boundary_waiting,
        vec![(0, 0, 0), (1, 0, 0), (2, 0, 0), (3, 0, 0)]
    );
    let unequal_bytes = estimate_backlog(&[0, 0, 0, 30], &[1_000, 20, 30, 40], &[10; 4], &[0; 4]);
    assert_eq!(
        (unequal_bytes.waiting_packets, unequal_bytes.waiting_bytes),
        (2, 50)
    );
    assert!(
        !lag_only.growing_three_periods,
        "increasing completion lag is not a growing waiting queue"
    );
    let overloaded = estimate_backlog(
        &[0, PERIOD_NS, PERIOD_NS * 2, PERIOD_NS * 3],
        &[100; 4],
        &[PERIOD_NS * 10; 4],
        &[0, 1, 2, 3],
    );
    assert_eq!(overloaded.waiting_packets, 3);
    assert_eq!(overloaded.max_boundary_lag_ns, PERIOD_NS * 36);
    assert_eq!(
        overloaded.boundary_waiting,
        vec![(0, 0, 0), (1, 1, 100), (2, 2, 200), (3, 3, 300)]
    );
    assert!(overloaded.growing_three_periods);
}

#[test]
#[ignore = "bounded R1 cost report; run optimized, one test thread"]
fn serial_receive_cost_report() {
    assert!(!cfg!(debug_assertions), "timing requires --release");
    // Metadata queries happen before fixture construction/timing; no production flag is added.
    let source_head = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .expect("source attribution");
    assert!(source_head.status.success());
    let source_head = String::from_utf8(source_head.stdout)
        .unwrap()
        .trim()
        .to_owned();
    let host_cpu = if cfg!(target_os = "macos") {
        std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
            .unwrap_or_else(|| "unavailable".to_owned())
    } else {
        "unavailable; see command receipt".to_owned()
    };
    let mut dispositions = Vec::new();
    for mbps in [50, 100] {
        for case in [Case::Paced, Case::Burst, Case::Repair] {
            let fixture = fixture(mbps, case, WARMUP + MEASURED);
            let packets = fixture
                .packets
                .iter()
                .filter(|p| p.frame >= WARMUP)
                .collect::<Vec<_>>();
            let arrivals = packets
                .iter()
                .map(|p| p.arrival_ns - WARMUP as u64 * PERIOD_NS)
                .collect::<Vec<_>>();
            let lengths = packets.iter().map(|p| p.datagram.len()).collect::<Vec<_>>();
            let frames = packets.iter().map(|p| p.frame - WARMUP).collect::<Vec<_>>();
            let frame_volume = fixture.data_shards + fixture.parity_shards;
            let mut pressure_repetitions = 0;
            for repetition in 1..=3 {
                let reference = traversal(&fixture, WARMUP);
                let costs = replay(&fixture, WARMUP, false);
                let unprotect = independent_unprotect(&fixture, WARMUP);
                assert_eq!(unprotect.len(), costs.len());
                let actual_service = costs.iter().map(|c| c.service_ns).collect::<Vec<_>>();
                let reference_service = reference.iter().map(|c| c.service_ns).collect::<Vec<_>>();
                let actual = estimate_backlog(&arrivals, &lengths, &actual_service, &frames);
                let overhead = estimate_backlog(&arrivals, &lengths, &reference_service, &frames);
                let pressure = actual
                    .waiting_packets
                    .saturating_sub(overhead.waiting_packets)
                    >= frame_volume
                    || actual.growing_three_periods;
                pressure_repetitions += usize::from(pressure);
                let mut per_frame = vec![0_u64; MEASURED];
                let mut finish = 0;
                let mut handoff_lags = Vec::with_capacity(MEASURED);
                for (i, (cost, &frame)) in costs.iter().zip(&frames).enumerate() {
                    per_frame[frame] += cost.service_ns;
                    finish = finish.max(arrivals[i]) + cost.service_ns;
                    if cost.frames_forwarded != 0 {
                        handoff_lags.push(finish.saturating_sub((frame as u64 + 1) * PERIOD_NS));
                    }
                }
                assert_eq!(handoff_lags.len(), MEASURED);
                assert_eq!(
                    costs.iter().map(|c| c.repaired_delta).sum::<u64>(),
                    if case == Case::Repair { 10 } else { 0 }
                );
                let complete = costs.iter().filter(|c| c.frames_forwarded != 0);
                let other = costs.iter().filter(|c| c.frames_forwarded == 0);
                let repair = costs.iter().filter(|c| c.repaired_delta != 0);
                let mut report = json!({"load_mbps":mbps, "case":format!("{case:?}"), "repetition":repetition,
                    "warmup_frames":WARMUP, "measured_frames":MEASURED, "datagrams":packets.len(),
                    "datagram_bytes":DATAGRAM_BYTES, "achieved_udp_payload_mbps":lengths.iter().sum::<usize>() as f64 * 8.0 / (MEASURED as f64 / 60.0) / 1_000_000.0,
                    "data_shards":fixture.data_shards, "parity_shards":fixture.parity_shards,
                    "first_sequence":FIRST_SEQUENCE, "srtp":"fixture AEAD_AES_256_GCM_8",
                    "fec_repair_percent":if case == Case::Repair { 1 } else { 0 },
                    "arrival":"paced per frame or first 1ms burst; missing shard repair at +2ms every60 frames",
                    "source_head":source_head, "source_worktree":"test-only R1; hashes in ledger", "host_cpu":host_cpu,
                    "authenticated":costs.iter().map(|c| c.authenticated_delta).sum::<u64>(),
                    "frames_forwarded":costs.iter().map(|c| c.frames_forwarded).sum::<u64>(),
                    "reconstructed_packets":costs.iter().map(|c| c.repaired_delta).sum::<u64>(),
                    "process":distribution(costs.iter().map(|c| c.process_ns)),
                    "forward":distribution(costs.iter().map(|c| c.forward_ns)),
                    "service":distribution(costs.iter().map(|c| c.service_ns)),
                    "service_per_frame":distribution(per_frame.into_iter()),
                    "frame_handoff_after_next_boundary":distribution(handoff_lags.into_iter()),
                    "frame_completing_service":distribution(complete.map(|c| c.service_ns)),
                    "noncompleting_service":distribution(other.map(|c| c.service_ns)),
                    "reconstruction_service":distribution(repair.map(|c| c.service_ns)),
                    "traversal_service":distribution(reference.iter().map(|c| c.service_ns)),
                    "backlog":backlog_json(&actual), "traversal_backlog":backlog_json(&overhead),
                    "pressure":pressure,
                    "host_arch":std::env::consts::ARCH, "host_os":std::env::consts::OS,
                    "build_profile":"release", "scope":"process_datagram -> forward_receive_event only"});
                report["decision"] = json!({
                    "extra_peak_waiting_packets_vs_traversal":actual.waiting_packets.saturating_sub(overhead.waiting_packets),
                    "extra_peak_waiting_datagram_bytes_vs_traversal":actual.waiting_bytes.saturating_sub(overhead.waiting_bytes),
                    "one_frame_waiting_packet_threshold":frame_volume,
                    "one_frame_waiting_datagram_byte_threshold":frame_volume*DATAGRAM_BYTES,
                });
                report["assert_enforced_zero_failure_counters"] = json!({
                    "authentication_failures":0, "unexpected_drops":0, "consumer_full":0,
                    "basis":"validity assertions must pass before this replay emits a report; not independently sampled counters"
                });
                let slowest_samples = slowest_indices(costs.iter().map(|cost| cost.service_ns)).into_iter()
                    .map(|index| {
                        let cost = costs[index];
                        let mut identity = packet_identity_json(packets[index], index);
                        identity["timing"] = json!({"replay_ordinal":cost.replay_ordinal,
                            "wall_elapsed_us":cost.elapsed_ns as f64/1_000.0,
                            "process_us":cost.process_ns as f64/1_000.0,
                            "forward_us":cost.forward_ns as f64/1_000.0,
                            "service_us":cost.service_ns as f64/1_000.0,
                            "frames_forwarded":cost.frames_forwarded,"repaired_packets":cost.repaired_delta});
                        identity
                    }).collect::<Vec<_>>();
                let unprotect_slowest = slowest_indices(unprotect.iter().map(|cost| cost.unprotect_ns)).into_iter()
                    .map(|index| {
                        let mut identity = packet_identity_json(packets[index], index);
                        identity["timing"] = json!({"wall_elapsed_us":unprotect[index].elapsed_ns as f64/1_000.0,
                            "unprotect_us":unprotect[index].unprotect_ns as f64/1_000.0});
                        identity
                    }).collect::<Vec<_>>();
                report["localization"] = json!({"replay_ordinal":costs[0].replay_ordinal,
                "slowest_samples":slowest_samples,
                "service_over_1ms_count":costs.iter().filter(|cost| cost.service_ns > 1_000_000).count(),
                "independent_unprotect":{
                    "distribution":distribution(unprotect.iter().map(|cost| cost.unprotect_ns)),
                    "authenticated_packets":unprotect.len(),"assert_enforced_authentication_failures":0,
                    "slowest_samples":unprotect_slowest,
                    "over_1ms_count":unprotect.iter().filter(|cost| cost.unprotect_ns > 1_000_000).count(),
                    "scope":"separate fresh receiver; identical full packet order and30warmup; independent elapsed clock; no subtraction or direct subphase identity"
                }});
                report["omitted_coverage"] = json!([
                    "fixed-size frames/keyframes: NAL type changes but larger IDR bursts not exercised",
                    "single-threaded feedback/admission: no contention with consumer/feedback threads",
                    "hardware crypto backend/ISA equivalence between M3 aarch64 and NucBox x86_64 not established",
                    "synthetic1ms burst:174x1200B is1.6704Gbps UDP payload, not a measured arrival distribution",
                    "socket/STUN/worker scheduling/decoder/Qt/scanout and time-correlated target queue/drop/service/scheduler state not exercised"
                ]);
                println!("RECEIVE_REPLAY_JSON {report}");
            }
            dispositions.push(pressure_repetitions >= 2);
        }
    }
    println!(
        "RECEIVE_REPLAY_DISPOSITION {}",
        if dispositions.contains(&true) {
            "LOCAL_SERVICE_PRESSURE_REPRODUCED"
        } else {
            "NOT_REPRODUCED_ON_LOCAL_HOST"
        }
    );
}
