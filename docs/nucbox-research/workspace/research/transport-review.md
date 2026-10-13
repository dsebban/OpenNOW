# OpenNOW transport review

Reviewed source at `bee18c118dbc89f42319436dcdb172d5b9e15e0c`, read-only. The repository root AGENTS.md applies; no nested AGENTS.md was found. No build, test, client launch, account access, or packet-payload inspection was performed.

Source paths below are relative to `vendor/OpenNOW/native/opennow-streamer/crates/`. `T` means `opennow-streamer-transport/src`, `C` means `opennow-streamer-core/src`. References name the starting line of the relevant implementation.

## What is reusable

The native transport is a credible starting point for the fork. It already has the two-socket layout seen in the official-client capture, SRTP authentication and replay handling, GS frame identifiers, Reed-Solomon repair, bounded media admission, private Mjolnir NACK-v2, and explicit reference recovery. A new socket/protocol implementation would discard substantial working code before proving any latency advantage.

`T/nvst.rs:5140` owns a reserved pair of UDP sockets and the bundle's `str0m::Rtc`. Video uses the lower port and ICE/DTLS the adjacent port. `C/lib.rs:873` starts the bundle worker, then `C/lib.rs:900` starts the dedicated Mjolnir worker using the reserved video socket. These workers share feedback state and the encoded-media consumer. The dedicated video worker performs NATT hole punching, raw SRTP receive/authentication, FEC/reordering and access-unit assembly. The bundle worker owns ICE, DTLS, SCTP, audio and control/input. Blocking decode is downstream.

The official capture supplied for this review agrees with that split: GS extension `0x4753` video on client UDP 49005/server 5004, STUN/DTLS on client UDP 49006, and TLS records on TCP 322. TCP headers alone do not prove the encrypted application's full RTSPS exchange. This is compatibility evidence for the architecture, not proof that this OpenNOW revision works against this seat.

`T/nvst.rs:5675` explicitly separates raw Mjolnir video from the DTLS bundle. Video uses session encryption material rather than assuming the bundle's DTLS SRTP keys. Profile parsing is explicit, with the legacy handoff defaulting to AEAD AES-256-GCM with an 8-byte tag at `T/nvst.rs:1517`. SRTP key derivation/authentication is at `T/nvst.rs:2364`; the replay window is 2,048 packets at `T/nvst.rs:213`. Preserve authentication before media processing and keep secrets out of diagnostics.

GS parsing at `T/nvst.rs:2992` retains sender frame ID, stream-packet index, SOF/EOF and multi-FEC coordinates. That is necessary for meaningful end-to-end loss and latency accounting. Port pairing and NATT behavior should also be retained: `T/nvst.rs:7440` uses authenticated version-6 STUN and deliberately avoids prepending the legacy raw PING.

## Recovery behavior in the current source

FEC operates before encoded-frame assembly. `T/nvst.rs:3456` implements NVIDIA-compatible GF arithmetic; `T/nvst.rs:3880` buffers FEC blocks in order. Complete healthy blocks drain immediately. Enough parity repairs missing data locally. Failed repair falls back to NACK. A later block proves that an incomplete head is stalled; only its first missing data run is requested while successor blocks remain buffered. Retention ends at 52 ms, a 2,048-packet span, or 128 pending FEC blocks. After exhaustion, reference continuity is invalidated and a keyframe is requested. A missing complete block follows the same bounded wait. With only one incomplete block, the code waits for a successor rather than immediately declaring loss.

The 52 ms threshold is checked by packet-driven `push` calls. It is not a guaranteed timer firing at exactly 52 ms when the socket becomes idle. General authenticated-media idle handling is separate, `T/nvst.rs:4256`, and transitions to `RecoveryRequired`; explicit recovery is required before normal receive resumes.

`T/nvst.rs:2839` selects private ServerControl `0x0317` NACK-v2 for Mjolnir on `control_channel_partially_reliable`. The bundle-only route uses Generic RTCP NACK on the RTCP SCTP channel. `T/nvst_control.rs:38` encodes version, stream ordinal and base-sequence/64-bit bitmap records, with at most 64 requested sequence numbers per command. Failed channel admission restores the attempt. A diagnostic reporting a NACK send proves local admission, not server receipt or retransmission.

`T/nvst.rs:1207` limits Mjolnir sends to `clamp(floor(52 ms / RTT), 1, 3)` including the first send. Retries wait RTT plus 4 ms; pending ranges expire after 52 ms. The control worker checks recovery every 4 ms at `T/nvst.rs:6846`. Fresh RTT preference is ICE pair, then video STUN, then bundle STUN, with five-second freshness and a 30 ms retry-only fallback at `T/nvst.rs:711`. This uses a current sample, not a median or tail estimator.

At the supplied official capture's STUN median RTT of 56.53 ms and p95 of 57.14 ms, the policy allows one send. A retransmission requiring a comparable full RTT would arrive beyond the 52 ms retention threshold. This is a concrete policy mismatch worth investigating. It does not prove real failure: STUN RTT is not the exact NACK response time, capture has no observed sequence gaps, and no returned retransmission was correlated. The 147.06 ms maximum also argues against choosing a deadline from one lucky minimum.

Private Mjolnir recovery uses control IDR rather than a second RTCP PLI/RR path, `T/nvst.rs:2884` and `T/nvst.rs:6872`. Requests have a 250 ms cooldown at `T/nvst.rs:88`; reference invalidation is local at `T/nvst.rs:4557`. Successful FEC or timely NACK prevents that more expensive recovery.

Older `docs/streamer-comparison/video.md:65` and `:107` describe 150 ms/generic-NACK behavior and are stale for this revision. `docs/mjolnir-nack-v2.md:55` explicitly leaves official mode-dependent 52/68 ms and initial-delay policies outside this implementation. The official client's exact active policy remains unverified here.

## Queue ownership and practical limits

The receive loop reads one datagram at a time and runs decryption/FEC/assembly on its worker, `T/nvst.rs:7380`. An 8 MiB requested socket receive buffer is configured at `T/nvst.rs:5358`; the actual Linux value and overflow rate require live observation. The video read timeout is 10 ms, but a blocking `recv_from` returns when data arrives, so this is not a fixed 10 ms per-packet latency. The bundle caps socket waits at 1 ms, `T/nvst.rs:7222`.

Encoded audio and video share an eight-entry synchronous channel in the normal engine-created path, `C/lib.rs:88` and `:813`. `T/lib.rs:81` uses `try_send`, so a slow consumer cannot block network receive. Full admission drops the incoming assembled frame. `T/nvst.rs:7691` marks a delivery discontinuity and retires undelivered feedback. It preserves the discontinuity until a frame reaches the consumer. The consumer forwards to the platform sink on its own thread, `C/lib.rs:2922`. Eight entries describe a capacity, not measured latency; both audio and video can consume them.

SCTP writes have a 128 KiB aggregate ceiling, half reserved for teardown, `T/nvst_budget.rs:1`. Normal input and recovery feedback share the remaining budget. The partial control stream is unordered with a 300 ms lifetime, `T/nvst_input.rs:90`. Thus a NACK can remain in SCTP longer than the video receiver's 52 ms useful recovery window. This is a potential stale-feedback cost, not evidence of observed head-of-line blocking. Documentation reports a different official default and does not justify blindly copying it.

Transport commands and receive events still use unbounded `mpsc::channel`, `T/nvst.rs:5737`; their downstream SCTP budget does not bound that upstream backlog.

## Ranked changes and validation

1. **Make recovery deadlines a measured session policy.** Instrument gap detection, NACK admission, socket send, returned authenticated sequence, FEC repair, expiry and keyframe arrival first. Then compare the existing 52 ms policy with a bounded 68 ms candidate and an RTT-informed candidate. Change tracking and dequeue together; preserve packet/block/memory limits. This can reduce recovery resets on the ~57 ms path, but increases exceptional gap latency. Use deterministic synthetic loss/reordering at 30/57/80 ms RTT and 0/4/12 ms jitter, followed by three repeated equal-length live trials per candidate. Report medians, p95/p99 frame age, recovered frames, IDR count, freeze duration and peak retained bytes. Do not increase the healthy-path buffer.

2. **Bound and measure commands before SCTP admission.** Replace the unbounded command backlog with explicit admission and motion coalescing while preserving key/button ordering, releases, text reservations and shutdown priority. Record queue age and depth, NACK admission failures and SCTP buffered bytes. Validate with a paused/slow bundle worker and an input flood; require bounded memory, prompt releases/stop, no old motion replay, and recovery feedback remaining serviceable. This addresses an actual ownership gap without changing the negotiated channel dialect.

The supplied 40,000-packet official capture had zero kernel capture drops, 24,364 video packets, 2,743 frame IDs, 5,628 FEC packets and no observed sequence gaps. It does not demonstrate a current transport fault or FEC-repair success. Keep existing FEC and receive architecture initially. Batching receive, reducing FEC, or changing reliability profiles needs profiling or loss evidence before implementation.
