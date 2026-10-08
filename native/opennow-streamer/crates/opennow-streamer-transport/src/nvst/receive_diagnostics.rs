//! Diagnostic-only, one-shot receive-owner observation. No protocol state is owned here.
//! Completion bins are NOT utilization bins: a span can cross several bins.
//! Phase stamps are wall-only. CPU is sampled at bracketed bin opportunities,
//! not per packet; it cannot assign CPU/non-CPU time to an individual phase/stall.
//! Worker-wide wall minus CPU is non-CPU elapsed time, NOT scheduler/runqueue cause.
//! The 180-s logical cutoff is checked by this owner, not an independent watchdog:
//! blocked owners can export late or never. Cutoff/partial records are not full coverage.
// Portable tests share the accumulator, but do not compile Linux export/sampling.
#![cfg_attr(
    not(all(target_os = "linux", feature = "receive-diagnostics")),
    allow(dead_code)
)]
use std::time::Instant;

pub(super) const BIN_NS: u64 = 100_000_000;
const BIN_COUNT: usize = 1_800;
const LONG_CAP: usize = 1_024;
const LONG_NS: u64 = 1_000_000;

#[derive(Clone, Copy)]
pub(super) enum Phase {
    Receive,
    Preprocess,
    Process,
    Forward,
    Between,
    Service,
}
const PHASES: [&str; 6] = [
    "receive",
    "preprocess",
    "process",
    "forward",
    "between",
    "service",
];

#[derive(Clone, Copy)]
pub(super) struct Stamp {
    pub wall: u64,
    cpu: Option<u64>,
}

#[derive(Clone, Copy, Default)]
struct Stats {
    count: u64,
    wall: u64,
    cpu: u64,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    cpu_covered_wall: u64,
    cpu_missing: u64,
    cpu_invalid: u64,
    max: u64,
}
impl Stats {
    fn add(&mut self, start: Stamp, end: Stamp) -> u64 {
        let wall = end.wall.saturating_sub(start.wall);
        self.count += 1;
        self.wall += wall;
        self.max = self.max.max(wall);
        match (start.cpu, end.cpu) {
            (Some(a), Some(b)) if b >= a && b - a <= wall => {
                self.cpu += b - a;
                #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
                {
                    self.cpu_covered_wall += wall;
                }
            }
            (Some(_), Some(_)) => self.cpu_invalid += 1,
            _ => self.cpu_missing += 1,
        }
        wall
    }
}
#[derive(Clone, Default)]
struct Bin {
    phases: [Stats; 6],
    raw: u64,
    bytes: u64,
    idle: u64,
    errors: u64,
    accepted: u64,
    assembled: u64,
    repaired: u64,
    handled_stun: u64,
    invalid_stun: u64,
    wrong_source: u64,
    counter_discontinuities: u64,
    socket: Option<SocketSample>,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    cpu_sample: Option<CpuSample>,
}
#[derive(Clone, Copy)]
struct SocketSample {
    before: u64,
    after: u64,
    memory: [u32; 9],
}
#[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
#[derive(Clone, Copy)]
struct CpuSample {
    before: u64,
    cpu: Option<u64>,
    after: u64,
}
struct LongSpan {
    phase: usize,
    start: Stamp,
    end: Stamp,
}

pub(super) struct Capture {
    origin: Instant,
    bins: Vec<Bin>,
    longs: Vec<LongSpan>,
    omitted: u64,
    receive_long_excluded: u64,
    first_authenticated_ns: Option<u64>,
    last_authenticated_ns: Option<u64>,
    first_assembled_ns: Option<u64>,
    last_assembled_ns: Option<u64>,
    last_service: Option<Stamp>,
    previous_counts: [u64; 6],
    used: usize,
    socket_errors: u64,
    cutoff_spans: u64,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    monotonic_anchor: Option<[u64; 3]>,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    final_socket: Option<SocketSample>,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    final_cpu_sample: Option<CpuSample>,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    cpu_sample_errors: u64,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    output: Option<std::fs::File>,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    pid: u32,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    tid: i64,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    role: String,
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    realtime_ns: Option<u64>,
}
impl Capture {
    pub(super) fn new() -> Self {
        Self {
            origin: Instant::now(),
            bins: vec![Bin::default(); BIN_COUNT],
            longs: Vec::with_capacity(LONG_CAP),
            omitted: 0,
            receive_long_excluded: 0,
            first_authenticated_ns: None,
            last_authenticated_ns: None,
            first_assembled_ns: None,
            last_assembled_ns: None,
            last_service: None,
            previous_counts: [0; 6],
            used: 0,
            socket_errors: 0,
            cutoff_spans: 0,
            #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
            monotonic_anchor: None,
            #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
            final_socket: None,
            #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
            final_cpu_sample: None,
            #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
            cpu_sample_errors: 0,
            #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
            output: None,
            #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
            pid: std::process::id(),
            #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
            tid: unsafe { libc::syscall(libc::SYS_gettid) },
            #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
            role: std::thread::current()
                .name()
                .unwrap_or("unknown")
                .to_owned(),
            #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
            realtime_ns: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()
                .and_then(|duration| duration.as_nanos().try_into().ok()),
        }
    }
    #[cfg(test)]
    pub(super) fn assert_fixture(&self, packets: u64, frames: u64, repaired: u64) {
        let sum = |field: fn(&Bin) -> u64| self.bins.iter().map(field).sum::<u64>();
        assert_eq!(sum(|b| b.raw), packets);
        assert_eq!(sum(|b| b.accepted), packets);
        assert_eq!(sum(|b| b.assembled), frames);
        assert_eq!(sum(|b| b.repaired), repaired);
        assert_eq!(
            sum(|b| b.errors
                + b.idle
                + b.handled_stun
                + b.invalid_stun
                + b.wrong_source
                + b.counter_discontinuities),
            0
        );
        for phase in [
            Phase::Receive,
            Phase::Preprocess,
            Phase::Process,
            Phase::Forward,
            Phase::Service,
        ] {
            assert_eq!(
                self.bins
                    .iter()
                    .map(|b| b.phases[phase as usize].count)
                    .sum::<u64>(),
                packets
            );
        }
        // Missing per-span CPU is intentional; actual sampling errors are separate.
        for stats in self.bins.iter().flat_map(|b| b.phases.iter()) {
            assert_eq!(stats.cpu_missing, stats.count);
            assert_eq!(stats.cpu, 0);
            assert_eq!(stats.cpu_invalid, 0);
        }
        assert_eq!(self.omitted, 0);
        assert_eq!(self.cutoff_spans, 0);
        assert_eq!(self.socket_errors, 0);
        #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
        {
            assert_eq!(self.cpu_sample_errors, 0);
            let samples = self.bins.iter().filter(|b| b.socket.is_some()).count();
            assert!(samples > 0, "fixture must exercise actual Linux samples");
            assert_eq!(
                samples,
                self.bins.iter().filter(|b| b.cpu_sample.is_some()).count()
            );
            for bin in self.bins.iter().filter(|b| b.socket.is_some()) {
                let socket = bin.socket.unwrap();
                assert!(socket.memory[1] > 0 && socket.memory[1] != u32::MAX);
                assert!(socket.after >= socket.before);
                let cpu = bin.cpu_sample.unwrap();
                assert!(cpu.cpu.is_some() && cpu.after >= cpu.before);
            }
        }
        println!(
            "RECEIVE_DIAGNOSTIC_FIXTURE_VALIDATION packets={packets} frames={frames} repaired={repaired} used_bins={} tails={} omissions={} cutoff={} sampling_errors={} span_cpu=unsampled",
            self.used,
            self.longs.len(),
            self.omitted,
            self.cutoff_spans,
            self.socket_errors
        );
    }
    pub(super) fn stamp(&self) -> Stamp {
        // Per-packet CPU-clock syscalls materially perturb receive service.
        // Preserve every wall boundary; CPU coverage is only per-bin brackets.
        Stamp {
            wall: self
                .origin
                .elapsed()
                .as_nanos()
                .try_into()
                .unwrap_or(u64::MAX),
            cpu: None,
        }
    }
    fn bin(&mut self, end: Stamp) -> Option<&mut Bin> {
        let index = (end.wall / BIN_NS) as usize;
        if index >= BIN_COUNT {
            self.cutoff_spans += 1;
            return None;
        }
        self.used = self.used.max(index + 1);
        Some(&mut self.bins[index])
    }
    pub(super) fn span(&mut self, phase: Phase, start: Stamp, end: Stamp) {
        let Some(bin) = self.bin(end) else {
            return;
        };
        let phase = phase as usize;
        if bin.phases[phase].add(start, end) >= LONG_NS {
            if phase == Phase::Receive as usize {
                // Normal blocking receive waits must not evict rare inline-work tails.
                // Their aggregate count/max/CPU/missing accounting above is unchanged.
                self.receive_long_excluded += 1;
                return;
            }
            if self.longs.len() < LONG_CAP {
                self.longs.push(LongSpan { phase, start, end });
            } else {
                self.omitted += 1;
            }
        }
    }
    pub(super) fn before_receive(&mut self) -> Stamp {
        let stamp = self.stamp();
        if let Some(previous) = self.last_service.take() {
            self.span(Phase::Between, previous, stamp);
        }
        stamp
    }
    pub(super) fn result(&mut self, end: Stamp, bytes: Option<usize>, idle: bool) {
        if let Some(bin) = self.bin(end) {
            if let Some(bytes) = bytes {
                bin.raw += 1;
                bin.bytes += bytes as u64;
            } else if idle {
                bin.idle += 1;
            } else {
                bin.errors += 1;
            }
        }
    }
    pub(super) fn service_end(&mut self, end: Stamp) {
        self.last_service = Some(end);
    }
    // Cumulative receiver counters remain separate from socket successes.
    pub(super) fn counters(&mut self, end: Stamp, counts: [u64; 6]) {
        let previous = self.previous_counts;
        self.previous_counts = counts;
        // The timer starts with the worker, not with a verified scene. These
        // processing-completion endpoints bound authentication/assembly evidence,
        // not continuous decode/presentation or scene-valid coverage.
        if end.wall < BIN_NS * BIN_COUNT as u64 {
            if counts[0] > previous[0] {
                self.first_authenticated_ns.get_or_insert(end.wall);
                self.last_authenticated_ns = Some(end.wall);
            }
            if counts[1] > previous[1] {
                self.first_assembled_ns.get_or_insert(end.wall);
                self.last_assembled_ns = Some(end.wall);
            }
        }
        if let Some(bin) = self.bin(end) {
            let mut deltas = [0; 6];
            for i in 0..6 {
                if let Some(delta) = counts[i].checked_sub(previous[i]) {
                    deltas[i] = delta;
                } else {
                    bin.counter_discontinuities += 1;
                }
            }
            bin.accepted += deltas[0];
            bin.assembled += deltas[1];
            bin.repaired += deltas[2];
            bin.handled_stun += deltas[3];
            bin.invalid_stun += deltas[4];
            bin.wrong_source += deltas[5];
        }
    }
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    pub(super) fn start() -> Option<Self> {
        use std::os::unix::fs::OpenOptionsExt;
        use std::sync::atomic::{AtomicBool, Ordering};
        static CLAIMED: AtomicBool = AtomicBool::new(false);
        let path =
            std::path::PathBuf::from(std::env::var_os("OPENNOW_NVST_RECEIVE_DIAGNOSTICS_FILE")?);
        if !path.is_absolute() {
            opennow_streamer_protocol::log::log_async(
                "WARN",
                "receive-diag",
                "status=non-absolute-output",
            );
            return None;
        }
        if CLAIMED.swap(true, Ordering::Relaxed) {
            opennow_streamer_protocol::log::log_async(
                "INFO",
                "receive-diag",
                "status=already-claimed",
            );
            return None;
        }
        // Exclusive, private output; never overwrite a previous capture or follow a symlink.
        let output = match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
        {
            Ok(file) => file,
            Err(_) => {
                opennow_streamer_protocol::log::log_async(
                    "WARN",
                    "receive-diag",
                    "status=output-unavailable",
                );
                return None;
            }
        };
        let mut capture = Self::new();
        capture.output = Some(output);
        let before = capture.stamp().wall;
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut ts) } == 0 {
            capture.monotonic_anchor = Some([
                before,
                ts.tv_sec as u64 * 1_000_000_000 + ts.tv_nsec as u64,
                capture.stamp().wall,
            ]);
        }
        opennow_streamer_protocol::log::log_async(
            "INFO",
            "receive-diag",
            &format!(
                "schema=1 status=started pid={} tid={} role={} duration_ms=180000 bin_ms=100 timer_origin=worker_setup media_coverage=unverified span_cpu=unsampled",
                capture.pid, capture.tid, capture.role
            ),
        );
        Some(capture)
    }
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    fn sample_cpu(&mut self) -> CpuSample {
        let before = self.stamp().wall;
        let cpu = thread_cpu();
        let after = self.stamp().wall;
        if cpu.is_none() {
            self.cpu_sample_errors += 1;
        }
        CpuSample { before, cpu, after }
    }
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    pub(super) fn checkpoint(owner: &mut Option<Self>, socket: &std::net::UdpSocket) {
        use std::os::fd::AsRawFd;
        let Some(capture) = owner.as_mut() else {
            return;
        };
        // Only wall time is needed for socket brackets. Do not issue a CPU-clock
        // syscall for every checkpoint when the 100-ms sample is not due.
        let before = capture
            .origin
            .elapsed()
            .as_nanos()
            .try_into()
            .unwrap_or(u64::MAX);
        let complete = before >= BIN_NS * BIN_COUNT as u64;
        let index = (before / BIN_NS).min(BIN_COUNT as u64 - 1) as usize;
        if !complete && capture.bins[index].socket.is_some() {
            return;
        }
        let cpu_sample = capture.sample_cpu();
        let socket_before = capture.stamp().wall;
        let mut memory = [0u32; 9];
        let mut length = std::mem::size_of_val(&memory) as libc::socklen_t;
        // Linux SO_MEMINFO is a read-only UAPI query, not an extra socket reader.
        let result = unsafe {
            libc::getsockopt(
                socket.as_raw_fd(),
                libc::SOL_SOCKET,
                libc::SO_MEMINFO,
                memory.as_mut_ptr().cast(),
                &mut length,
            )
        };
        let after = capture
            .origin
            .elapsed()
            .as_nanos()
            .try_into()
            .unwrap_or(u64::MAX);
        // Failed samples are not retried per packet; retain an unavailable marker.
        if result != 0 || length as usize != std::mem::size_of_val(&memory) {
            capture.socket_errors += 1;
            memory = [u32::MAX; 9];
        }
        let sample = SocketSample {
            before: socket_before,
            after,
            memory,
        };
        if complete {
            capture.final_socket = Some(sample);
            capture.final_cpu_sample = Some(cpu_sample);
            if let Some(capture) = owner.take() {
                capture.export(true);
            }
        } else {
            capture.bins[index].socket = Some(sample);
            capture.bins[index].cpu_sample = Some(cpu_sample);
            capture.used = capture.used.max(index + 1);
        }
    }
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    fn export(mut self, complete: bool) {
        let Some(output) = self.output.take() else {
            return;
        };
        if self.final_cpu_sample.is_none() {
            self.final_cpu_sample = Some(self.sample_cpu());
        }
        let elapsed = self.stamp().wall;
        let preallocated_bytes = std::mem::size_of::<Self>()
            + self.bins.capacity() * std::mem::size_of::<Bin>()
            + self.longs.capacity() * std::mem::size_of::<LongSpan>();
        // Transfer bounded buffers once; no exporter owns a socket, receiver or feedback lock.
        let result = std::thread::Builder::new().name("nvst-diag-export".into()).spawn(move || {
            use std::io::Write;
            let mut output = std::io::BufWriter::new(output);
            let bins: Vec<_> = self.bins[..self.used].iter().enumerate().map(|(index, bin)| {
                let phases: Vec<_> = bin.phases.iter().map(|s| serde_json::json!([s.count,s.wall,s.cpu,s.cpu_covered_wall,s.cpu_missing,s.cpu_invalid,s.max])).collect();
                let socket = bin.socket.map(|s| serde_json::json!([s.before,s.after,s.memory]));
                let cpu_sample = bin.cpu_sample.map(|s| serde_json::json!([s.before,s.cpu,s.after]));
                serde_json::json!({"index":index,"phases":phases,"raw":bin.raw,"bytes":bin.bytes,"idle":bin.idle,"errors":bin.errors,"accepted_authenticated":bin.accepted,"assembled":bin.assembled,"repaired":bin.repaired,"stun_ok":bin.handled_stun,"stun_invalid":bin.invalid_stun,"wrong_source":bin.wrong_source,"counter_discontinuities":bin.counter_discontinuities,"socket":socket,"cpu_sample":cpu_sample})
            }).collect();
            let longs: Vec<_> = self.longs.iter().map(|s| serde_json::json!([s.phase,s.start.wall,s.end.wall,s.start.cpu,s.end.cpu])).collect();
            let mut value = serde_json::json!({"schema":1,"complete":complete,"preallocated_bytes":preallocated_bytes,"elapsed_ns":elapsed,"pid":self.pid,"tid":self.tid,"role":self.role,"realtime_origin_ns":self.realtime_ns,"clock":"Instant-relative-monotonic; realtime anchor approximate only","span_cpu":"unsampled; phase cpu_missing counts intentional absence of per-span CPU coverage","bin_ns":BIN_NS,"phase_names":PHASES,"phase_fields":["count","wall_ns","thread_cpu_ns","cpu_covered_wall_ns","cpu_missing","cpu_invalid","max_wall_ns"],"socket_fields":["before_ns","after_ns","SO_MEMINFO_u32"],"socket_memory_fields":["rmem_alloc","rcvbuf","wmem_alloc","sndbuf","fwd_alloc","wmem_queued","optmem","backlog","drops"],"long_fields":["phase","start_ns","end_ns","start_cpu_ns","end_cpu_ns"],"long_omitted":self.omitted,"cutoff_spans":self.cutoff_spans,"monotonic_anchor":self.monotonic_anchor,"final_socket":self.final_socket.map(|s|serde_json::json!([s.before,s.after,s.memory])),"socket_errors":self.socket_errors,"bins":bins,"long_spans":longs,"limits":"complete means timer reached, not full coverage. Partial exit can omit terminal in-flight phase/counters. Completion bins are not utilization bins. Delivered receive timing only; no kernel arrivals or scheduler trace. Socket drops are interval brackets; memory is not payload bytes. Non-CPU elapsed is not runqueue or blocking attribution."});
            value["cpu_sampling"] = serde_json::json!({"policy":"one bracketed thread-CPU snapshot at the first worker opportunity per 100-ms bin, plus final snapshot","fields":["before_ns","thread_cpu_ns","after_ns"],"errors":self.cpu_sample_errors,"final":self.final_cpu_sample.map(|s|serde_json::json!([s.before,s.cpu,s.after])),"limits":"Worker-wide snapshot differences include receive, inline work, housekeeping and diagnostics. No per-span CPU or blocking/preemption/runqueue attribution; delayed worker means delayed snapshots."});
            value["media_coverage"] = serde_json::json!({"timer_origin":"worker_setup","first_authenticated_ns":self.first_authenticated_ns,"last_authenticated_ns":self.last_authenticated_ns,"first_assembled_ns":self.first_assembled_ns,"last_assembled_ns":self.last_assembled_ns,"limits":"Processing-completion endpoints only, not continuous media/decode/presentation or verified scene coverage. Startup is included; scene validation after timer start does not earn 180 seconds of valid scene."});
            value["long_policy"] = serde_json::json!({"threshold_ns":LONG_NS,"capacity":LONG_CAP,"receive_long_excluded":self.receive_long_excluded,"limits":"Receive has aggregate accounting only, never interval-tail records. Other phases share the capped tail pool; no full receive interval trace."});
            let ok = serde_json::to_writer(&mut output, &value).is_ok() && output.flush().is_ok();
            opennow_streamer_protocol::log::log_async(if ok { "INFO" } else { "WARN" }, "receive-diag", if ok { "status=exported" } else { "status=export-failed" });
        });
        if result.is_err() {
            opennow_streamer_protocol::log::log_async(
                "WARN",
                "receive-diag",
                "status=export-thread-failed",
            );
        }
    }
}
#[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
impl Drop for Capture {
    fn drop(&mut self) {
        // Move buffers out on early worker return; export itself takes the file so
        // the moved value's Drop cannot export recursively.
        if self.output.is_some() {
            let mut partial = Self::new();
            std::mem::swap(self, &mut partial);
            partial.export(false);
        }
    }
}
#[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
fn thread_cpu() -> Option<u64> {
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    {
        let mut ts = libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        };
        if unsafe { libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut ts) } == 0 {
            return Some((ts.tv_sec as u64) * 1_000_000_000 + ts.tv_nsec as u64);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn stamp(wall: u64, cpu: Option<u64>) -> Stamp {
        Stamp { wall, cpu }
    }
    #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
    #[test]
    fn linux_socket_cpu_and_partial_export_smoke() {
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
        let path = std::env::temp_dir().join(format!(
            "opennow-receive-diag-test-{}-{}-{}",
            std::process::id(),
            unsafe { libc::syscall(libc::SYS_gettid) },
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        let mut capture = Capture::new();
        capture.output = Some(file);
        let mut owner = Some(capture);
        Capture::checkpoint(&mut owner, &socket);
        let capture = owner.as_mut().unwrap();
        assert_eq!(capture.socket_errors, 0);
        assert!(capture.bins.iter().find_map(|b| b.socket).unwrap().memory[1] > 0);
        let initial_cpu = capture
            .bins
            .iter()
            .find_map(|b| b.cpu_sample)
            .unwrap()
            .cpu
            .unwrap();
        let start = capture.stamp();
        let mut value = 1u64;
        for _ in 0..100_000 {
            value = std::hint::black_box(value.wrapping_mul(3));
        }
        std::hint::black_box(value);
        let end = capture.stamp();
        capture.span(Phase::Process, start, end);
        std::thread::sleep(std::time::Duration::from_millis(100));
        Capture::checkpoint(&mut owner, &socket);
        let capture = owner.as_ref().unwrap();
        let final_cpu = capture
            .bins
            .iter()
            .filter_map(|b| b.cpu_sample)
            .last()
            .unwrap()
            .cpu
            .unwrap();
        assert!(final_cpu > initial_cpu);
        assert_eq!(capture.cpu_sample_errors, 0);
        let latest_index = capture
            .bins
            .iter()
            .rposition(|bin| bin.cpu_sample.is_some())
            .unwrap();
        let latest_cpu = capture.bins[latest_index].cpu_sample.unwrap();
        let latest_socket = capture.bins[latest_index].socket.unwrap();
        drop(owner); // Early worker exit must export partial, without retaining the socket.
        let deadline = Instant::now() + std::time::Duration::from_secs(5);
        let value = loop {
            if let Ok(bytes) = std::fs::read(&path) {
                if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                    break value;
                }
            }
            assert!(Instant::now() < deadline, "export did not complete");
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        assert_eq!(value["complete"], false);
        assert_eq!(value["socket_errors"], 0);
        let latest_bin = &value["bins"][latest_index];
        assert_eq!(latest_bin["index"], latest_index);
        assert_eq!(
            latest_bin["cpu_sample"],
            serde_json::json!([latest_cpu.before, latest_cpu.cpu, latest_cpu.after])
        );
        assert_eq!(
            latest_bin["socket"],
            serde_json::json!([
                latest_socket.before,
                latest_socket.after,
                latest_socket.memory
            ])
        );
        assert_eq!(value["cpu_sampling"]["errors"], 0);
        assert!(value["cpu_sampling"]["final"][1].as_u64().is_some());
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        // Owned disposable fixture only; keep it for inspection rather than deleting it.
        println!("partial_export_fixture={}", path.display());
    }

    #[test]
    fn cross_bin_spans_missing_cpu_and_bounded_tails_are_explicit() {
        let mut capture = Capture::new();
        for _ in 0..LONG_CAP + 3 {
            capture.span(
                Phase::Process,
                stamp(BIN_NS - 1, Some(0)),
                stamp(BIN_NS + LONG_NS, Some(10)),
            );
        }
        assert_eq!(capture.bins[0].phases[2].count, 0);
        assert_eq!(capture.bins[1].phases[2].count, (LONG_CAP + 3) as u64);
        assert_eq!(capture.longs.len(), LONG_CAP);
        assert_eq!(capture.omitted, 3);
        #[cfg(all(target_os = "linux", feature = "receive-diagnostics"))]
        assert_eq!(
            capture.bins[1].phases[2].cpu_covered_wall,
            (LONG_CAP as u64 + 3) * (LONG_NS + 1)
        );
        println!(
            "portable_preallocated_bytes={}",
            std::mem::size_of::<Capture>()
                + BIN_COUNT * std::mem::size_of::<Bin>()
                + LONG_CAP * std::mem::size_of::<LongSpan>()
        );
        capture.span(Phase::Forward, stamp(0, None), stamp(10, None));
        capture.span(Phase::Forward, stamp(0, Some(0)), stamp(10, Some(20)));
        assert_eq!(capture.bins[0].phases[3].cpu_missing, 1);
        assert_eq!(capture.bins[0].phases[3].cpu_invalid, 1);
        capture.span(
            Phase::Process,
            stamp(0, None),
            stamp(BIN_NS * BIN_COUNT as u64, None),
        );
        assert_eq!(capture.used, 2);
    }
    #[test]
    fn normal_receive_waits_do_not_hide_later_service_tails() {
        let mut capture = Capture::new();
        let wait_ns = 10_000_000;
        for i in 0..LONG_CAP + 32 {
            let start = i as u64 * wait_ns;
            capture.span(
                Phase::Receive,
                stamp(start, Some(0)),
                stamp(start + wait_ns, Some(1_000)),
            );
        }
        let start = (LONG_CAP + 32) as u64 * wait_ns;
        capture.span(
            Phase::Receive,
            stamp(start, None),
            stamp(start + wait_ns, None),
        );
        capture.span(
            Phase::Process,
            stamp(start + wait_ns, None),
            stamp(start + 2 * wait_ns, None),
        );
        capture.span(
            Phase::Forward,
            stamp(start + 2 * wait_ns, None),
            stamp(start + 3 * wait_ns, None),
        );
        let receive_count: u64 = capture.bins.iter().map(|b| b.phases[0].count).sum();
        let receive_cpu: u64 = capture.bins.iter().map(|b| b.phases[0].cpu).sum();
        let receive_missing: u64 = capture.bins.iter().map(|b| b.phases[0].cpu_missing).sum();
        let receive_max = capture.bins.iter().map(|b| b.phases[0].max).max().unwrap();
        assert_eq!(receive_count, LONG_CAP as u64 + 33);
        assert_eq!(receive_cpu, (LONG_CAP as u64 + 32) * 1_000);
        assert_eq!(receive_missing, 1);
        assert_eq!(receive_max, wait_ns);
        assert_eq!(capture.longs.len(), 2);
        assert_eq!(capture.longs[0].phase, Phase::Process as usize);
        assert_eq!(capture.longs[1].phase, Phase::Forward as usize);
        assert_eq!(capture.receive_long_excluded, LONG_CAP as u64 + 33);
        assert_eq!(capture.omitted, 0);
    }

    #[test]
    fn raw_accepted_frames_and_counter_resets_remain_distinct() {
        let mut capture = Capture::new();
        let end = stamp(1, None);
        capture.result(end, Some(1200), false);
        capture.result(end, Some(48), false);
        capture.result(end, None, true);
        capture.result(end, None, false);
        capture.counters(end, [1, 0, 0, 1, 0, 0]);
        capture.counters(stamp(10, None), [2, 1, 1, 1, 0, 0]);
        capture.counters(stamp(20, None), [2, 1, 1, 1, 0, 0]);
        capture.counters(stamp(30, None), [0; 6]);
        let bin = &capture.bins[0];
        assert_eq!((bin.raw, bin.bytes, bin.idle, bin.errors), (2, 1248, 1, 1));
        assert_eq!(
            (bin.accepted, bin.assembled, bin.repaired, bin.handled_stun),
            (2, 1, 1, 1)
        );
        assert_eq!(bin.counter_discontinuities, 4);
        assert_eq!(
            (
                capture.first_authenticated_ns,
                capture.last_authenticated_ns
            ),
            (Some(1), Some(10))
        );
        assert_eq!(
            (capture.first_assembled_ns, capture.last_assembled_ns),
            (Some(10), Some(10))
        );
    }
}
