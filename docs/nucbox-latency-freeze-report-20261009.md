# NucBox latency and freeze investigation — assessment report (2026-10-08 → 2026-10-09)

Branch `nucbox-wayland-hdr`, commits `6472cae7` … `ddbca67e`. Source of truth for raw numbers:
`prompt-exports/optimize-nucbox-latency-freezes-runs.md` (append-only ledger) and the Oracle
exports beside it. Raw logs, captures and credentials stay on the NucBox (private `.runtime/`).

## 1. Bottom line

- **No latency improvement and no freeze fix was shipped or demonstrated.** Zero product
  optimization iterations passed their gate.
- What was delivered: working NucBox deploy/automation flow, a verified baseline characterization,
  reproduction of two distinct failure modes, privacy-correct diagnostics, and two opt-in
  discriminators (receive timing — rejected on cost; Vulkan frame timing — admitted and run).
- The freeze root cause is **unresolved**. The strongest lead for cadence is the synchronous
  Vulkan snapshot fence wait on the decoder worker (see §5), but it is not shown to cause the
  observed gaps.

## 2. Timeline of work

| Phase | Work | Outcome |
|---|---|---|
| Setup | Skill mirror + AGENTS.md NucBox section (`6472cae7`); Herdr/foreground-Cua guidance (uncommitted until now) | Done; box fast-forwarded and clean |
| A1/A2 baseline | 3×3 plan, one build-once, live Ori stream | A1 valid; **A2 INVALID** (~16 s media stall, auto-terminated); repeatability incomplete |
| D1 | Retained A2 freeze adjudication | **STAGE_ONLY**: no independent discriminator for cloud-stop vs local |
| B1 | Short baseline | **Incomplete** — Cua Play action timed out (libei/EIS input not ready) |
| Cua repair | Restored Wayland input | Automation working again |
| c10d37d0 / b66a0ea9 | RTSPS control-exit + session lifecycle diagnostics (Claude on box), `session.remote.list` error logging | Built (c10); b66 unbuilt |
| Claude trial | Idle stream, watcher | ~609 s RTSPS reset → video stops → media timeout → recovery exhausted → CloudMatch `finished` (~625 s). Inactivity plausible, **not proven**; server never answered control keepalives |
| B1R | Nine live windows, c10 artifact | Descriptive baseline (§3); **168 video-socket drops** in one window; gaps up to 304 ms / >512 ms |
| R1 / R1b | Test-only authenticated receive replay (18-case matrix) | Valid negative: no local service-pressure mechanism localized; **no fix earned** |
| T1–T5 (H0–H3) | Opt-in bounded receive-timing diagnostics, 5 hardening rounds | All failed the declared admission gate (p99 perturbation); never run live |
| T6 | Ordinary compiled-OUT build, one 180 s window | Valid, no freeze; freeze verdict INCONCLUSIVE |
| T7 | Opt-in paired Vulkan frame timing (`ddbca67e`), one 180 s window | Valid; HOST_WAIT_INVESTIGATION (§5) |

## 3. Measured results

Fresh-frame interval (Qt-observed upper bounds; **not** input-to-photon latency):

| Run | p95 / p99 / max (ms) | Late >25 ms |
|---|---|---|
| H0 historical (unmatched) | 21 / 25 / 36–41 | 0.77–0.83 % |
| A1 (3×60 s) | 21 / 24–25 / 41–56 | 0.67–0.98 % |
| B1R pooled (9×60 s, c10) | 22 / 26 / overflow (>512) | 1.070 % (range 0.99–1.21 % per attempt) |
| T6 ordinary, 180 s | 21 / 24 / 43 | 0.718 % |
| T7 Vulkan-timing, 180 s | source-interval p95≤21, p99≤25, max 293 | 0.854 % |

Stable throughout: 2560×1080, 60 FPS, H.265 10-bit 4:2:0, 100 Mbps, display 3440×1440@100, ping
~57–67 ms, reported loss 0 %, decoder residence ~8 ms. Occasional 190–304 ms gaps recovered on their
own. The 100 Hz collector is descriptive, not legacy Doctor acceptance.

## 4. Failure modes established

1. **Terminal freeze (reproduced twice: A2, idle trial).** Video counters stop while Qt keeps
   running; ~15 s later media timeout, one recovery attempt, then session end. Local worker was
   alive (pings advanced). Cloud inactivity timeout is plausible (idle trial had no input) but
   unproven; no allocation-correlated server cause was captured. Intermediate-hop pings do not
   prove peer-path health.
2. **Transient stutters (up to 300 ms+).** B1R T3 showed **+168 kernel UDP drops on the video
   socket** (rcvbuf 425984 B), same window as >500 ms gaps. Mechanism (burst vs. slow service) not
   localized; replay found no local pressure. No rmem/network change was made.
3. **Control channel silence.** Server sends nothing on RTSPS control after PLAY and never answers
   keepalives (pings sent, pongs 0). Present in healthy sessions, so not itself a freeze signal.

## 5. Latest evidence (T7) and the one remaining candidate

Output-producing decode spans (10 496 paired outputs, 175 s): send→receive 0.30 ms, conversion
7.97 ms, of which host fence wait **7.87 ms (95.2 %)**. This is host-wait inside
`VulkanCopyPool::copy_frame` (`video/vulkan_copy.rs`), including unfinished GPU decode, so it is
not proven to be avoidable work. The 293 ms gap in the same window is **not** attributable to it
(max successful pair 28 ms). Candidate: separate source-dependency wait from copy completion before
considering any asynchronous completion handoff. Not implemented; gain unproven.

## 6. Diagnostics and engineering delivered

- RTSPS control-exit reasons, session state/error lifecycle logging, `session.remote.list` errors;
  peer text removed from lifecycle logs (privacy fix).
- Opt-in receive diagnostics (defaults OFF, bounded, compiled-out in ordinary builds) — **not
  deployable**: H0–H3 each failed p99 perturbation (best: median cost 4.16 % pass, p99 +9.6 % fail
  vs. 5 % limit).
- Opt-in `OPENNOW_VULKAN_FRAME_TIMING=1` paired timing (default OFF, never persisted).
- Test-only authenticated receive replay harness; 306–311 Linux tests passing at last check.
- Verified Cua-on-Wayland deployment/driving procedure, clean-quit flow, per-run isolation.
- Unresolved: prior full-suite ALSA failure remains FAIL; b66a0ea9 never built on the box.

## 7. Assessment of process

Honest read: the investigation spent most effort on measurement scaffolding and admission gates
for instrumentation that could not meet its own cost budget, rather than on a fix. The gates did
prevent false claims (several invalid windows retained, no cherry-picking, no gain asserted).
Cost: ~two days of NucBox time, zero shipped improvement.

## 8. Recommended next steps (in order)

1. Capture a **second** terminal freeze with the already-built c10 lifecycle logging plus the
   ordinary build, with periodic gameplay input vs. a matched no-input run, to test the
   inactivity-timeout hypothesis cheaply (no new instrumentation).
2. For the video-socket drops: test a larger receive buffer (`rmem`) or a dedicated high-priority
   drain thread as an A/B — needs explicit approval for the rmem change.
3. Only then revisit the Vulkan fence wait (T7 candidate), distinguishing dependency wait from
   copy completion.
4. Retire the receive-timing diagnostic line (H0–H3) unless a cheaper design is found.

## 9. Evidence index

Ledger: `prompt-exports/optimize-nucbox-latency-freezes-runs.md`. Oracle plans/reviews:
`prompt-exports/oracle-*.md` (16 files). Historical baseline: `docs/nucbox-latency-baseline-20261008.md`.
Private on box: `~/dev/gfn-client-research/.runtime/{vulkan-frame-live-XTt5DLKx,ordinary-validation-ywkyorcw,verification-rtsps-AE3DjG57,…}`.
