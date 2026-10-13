# Ori stall localization, 12 October 2026

A new saved-account Ori allocation ran for more than 35 minutes without unexpected termination. A 30 minute monitored interval had 1801 Qt samples,107574 fresh-source swaps,59.76 presented FPS,3396 intervals above 25 ms, seven intervals above 100 ms, P99 upper 31 ms and maximum histogram upper 338 ms. No runtime-stop, presentation-gating or PTS-discontinuity samples occurred. Stalls remain unresolved.

The stream used Germany, HEVC 10-bit HDR,2560×1080 at 60 source FPS. The display stayed 3440×1440 at 100 Hz, bt2100 and desktop mode. Internet remained direct. No connectivity configuration changed. Rejected pacing and deferred-copy experiments were disabled. This new cloud allocation is a separate comparison block.

The application-context Anti-AFK shortcut fix was built and exercised with the stream menu open. The native input queue count increased by two at each four-minute timer boundary. Seven scheduled pulses occurred during the monitored interval. Queue acceptance is not independent remote-delivery proof; sustained playback supports the inactivity interpretation of earlier terminations. No wire keepalive behavior changed.

An armed 180 second receive capture assembled 10799 frames. Its corrected first counter bin contained six frames. Processing max 0.189 ms, forwarding max 0.033 ms and receiver service max 1.614 ms argue against receive-owner service starvation in that interval. Two 100 ms bins contained no assembled frame:

| Capture bin | Authenticated packets | Subsequent burst | Qt gap upper bound | Estimated overlapping gateway RTT |
| --- | ---: | --- | ---: | --- |
|621|3|12 frames in next 100 ms|131 ms|19.5 ms|
|791|116|16 frames two bins later|338 ms|88.0/91.7 ms|

The thirty-minute socket observer saw no drops on either owned UDP socket. The two fifteen-minute gateway observers saw one unanswered ping in the first interval and none in the second; maxima were 106 ms and 72.8 ms. Ping timing is measured in userspace and cannot establish kernel arrival timing or causality.

Code inspection found a potential amplification mechanism: incomplete FEC heads hold successors, and the 52 ms deadline restarts when the first missing shard changes. This capture does not prove that renewal caused a real stall. The existing expiration path invalidates the picture and requests keyframe recovery before feeding successors; preserve that contract.

A bounded diagnostic for blocked-head episodes is now implemented behind the existing armed receive-capture feature. It records deadline restarts, ready successors and observed ages without changing dequeue behavior. Focused and full transport checks passed; live deployment and verification are next. No performance candidate is accepted.

Evidence is private under `.runtime/stall-goal-20261011/`. Exact build hashes and process identities are in Doctor records; raw logs were archived only after normal GUI quit and verified Qt/core exit. All observers completed. The goal remains active.

## Follow-up receive and FEC capture

The rebuilt diagnostic passed 17 focused tests and all 321 transport crate tests, with two preexisting tests ignored. Its real 180-second capture produced both private JSON files and completed normal GUI cleanup afterward. No FEC head episode, omission, or unmatched finish was recorded. All 10,801 frames assembled without an empty 100 ms bin, while Qt presented 10,068 fresh sources and recorded three intervals above 100 ms, including histogram overflow.

Native decode counters advanced 9,634 outputs over 170.04 seconds while assembly advanced 10,203. Decode timing epochs changed from 0 to 1 to 2, so this is recovery evidence rather than a stable comparison. Recognized events included two video drops, one keyframe request, and 93 decoded mailbox replacements. Mailbox replacement cannot explain the assembled-to-decoded deficit. Encoded backpressure can gate deltas until an IDR and reopen the decoder, amplifying a slow frame. The next run uses the existing Vulkan frame-timing diagnostic to measure mean codec, conversion and fence-wait service times. No deadline-policy change is justified by this capture.

## GPU service and recovery admission

The existing frame-timing diagnostic measured 34,304 successful paired outputs across 134 fixed-scene batches. Mean codec time was 0.280 ms, conversion 10.306 ms and nested fence wait 10.212 ms. Codec plus conversion averaged 10.586 ms, below the 16.667 ms budget. Fence waiting includes decode dependencies, GPU scheduling and CPU wakeup, not only copying. Measurements exclude compressed queue residence, decoder reopening and failed conversions; they do not identify the whole recovery delay. Two decoder identities occurred in this selected interval.

A concrete admission bug is corrected in source: when an IDR replaces stale encoded work, native submission now reports Queued, so embedded feedback accepts it rather than asking for another keyframe. An IDR rejected because control commands fill the queue remains a drop and requests recovery. The nonembedded path likewise no longer infers retention from the keyframe flag. Four focused overflow checks passed.

Full Linux checks initially exposed an unchanged audio fixture failure: its custom ALSA configuration omitted ctl.hw needed for card-name enumeration. The fixture now supplies that control definition without adding hardware playback PCMs. The isolated audio check and complete default Linux crate suite pass. No host audio configuration changed.

The rebuilt client also includes opt-in reopening timing under OPENNOW_VULKAN_FRAME_TIMING. It measures total clear/open/replace time and separately codec-open time, including destruction of the previous decoder. Real playback validation of the admission fix and reopening measurements remain pending. No matched latency improvement is claimed.

## Recovery admission qualification

The corrected client completed a 30-minute monitored interval without disconnecting: 59.91 presented FPS, 2,548 intervals above 25 ms, five gaps above 100 ms, maximum histogram upper bound 288 ms and P99 upper 29 ms. Anti-AFK input queue pulses remained spaced by at most 240.001 seconds. This is qualification on a new allocation, not a matched performance comparison.

The 272 ms gap coincided with an NVST gap increment, reference invalidation and an IDR request. Later recovery changed the decode epoch. A queue-triggered decoder reopen took 1.775 ms total, with 54 microseconds opening the codec, so codec reopening itself does not explain the long pause. The archived native log covers the monitored interval and records 30 compressed-video drops and three keyframe requests. Qt totals cover the full interval.

A source review found that the blocked-head probe misses wholly absent FEC blocks. Those enter a separate 52 ms waiting branch before PacketGap. Zero blocked-head episodes therefore does not exclude FEC gap waiting. The next diagnostic extension classifies this branch and records effective incoming FEC shard metadata without changing recovery timing.

Both gateway intervals received every transmitted probe; maxima were 38 ms and 87.6 ms. Socket observation completed successfully. These observations do not establish where packets were delayed. Normal GUI quit was verified against both exact process IDs before logs were archived. Stalls remain unresolved; the goal stays active.

## Long-window gap classification

The opt-in probe now labels incomplete FEC heads, wholly missing FEC blocks and RTP sequence gaps. It retains bounded episode records and a 32-tuple incoming FEC histogram. Missing-block frame coordinates identify the first available successor rather than the absent block. No NACK, recovery or release deadline changed.

An optional integral capture duration from 180 through 1800 seconds keeps the default three-minute window and bounds allocation at 18,000 bins. Startup and output metadata report actual duration. Invalid values reject only diagnostics. Narrow transport compilation with the receive-diagnostics feature passed; deployment and a real thirty-minute capture are next.

## Thirty-minute classified capture

The long capture completed with 18,000 bins and 108,000 assembled frames. The monitored interval averaged 59.90 presented FPS, with six gaps above 100 ms, maximum upper bound 276 ms and P99 upper 29 ms. Native telemetry retained a stable decode epoch, one keyframe request and 12 compressed-video drops. Ori stayed connected with seven four-minute activity pulses. Both socket observations completed; both gateway intervals answered every transmitted probe.

All six large presentation gaps were near elevated gateway RTTs, ranging from 57.6 to 128 ms. This is correlation, not proof of packet-arrival timing. Receive service maximum was 7.09 ms, processing 1.11 ms and forwarding 0.37 ms, below the visible gap sizes. Two 100 ms bins had no assembled frames.

The new probe recorded one incomplete FEC head, no whole missing blocks or RTP gaps, and no omissions. Its contiguous missing data run covered nine shards. It expired after an observed 59.8 ms with ten ready successors and twelve buffered blocks. There were no deadline restarts. This event overlaps the 276 ms presentation gap, so timer renewal is not its mechanism.

Incoming FEC normally supplied two parity shards for 19–40 data shards, with sender repair percentages of 5–10. Larger rare blocks had three or five parity shards. Source ANNOUNCE defaults therefore do not establish effective live parity.

The next decision is whether the fixed 52 ms FEC recovery deadline leaves enough time for retransmission at approximately 56 ms stream RTT. A focused Surf review is pending; no deadline or FEC negotiation change is enabled. The goal remains active.

The classified run was closed normally after all observers completed; exact Qt/core exits and archive files were verified. Fourteen focused diagnostic checks passed. The completed Surf review recommends a default-off, admission-gated recovery window capped at 69 ms, with queued/admitted NACK and matching post-NACK packet timing. This is an experiment, not an accepted policy. It preserves NACK encoding, retry/tracking policy and reference invalidation on expiry. Implementation and matched same-cloud comparison are pending.

## RTT deadline candidate rejected

The deployed candidate passed 332 transport tests with two preexisting ignores and the default-feature compile check. Real GUI playback verified HEVC 10-bit HDR, Germany, 2560×1080p60 source and the preserved 3440×1440/100 Hz desktop. Activity pulses remained on a four-minute cadence. Mode application was verified through source receipts.

The first baseline window completed 30 minutes at 59.929 presented FPS, 12 intervals above 100 ms, maximum upper 238 ms and P99 upper 26 ms. No eligible FEC head occurred. The first candidate window completed approximately 30 minutes at 59.916 FPS, eight intervals above 100 ms, maximum upper 317 ms and P99 upper 28 ms. Its two eligible heads expired.

During the second candidate window, the total reached six eligible heads, all expired with no matching requested-sequence observation before expiry. Four actually used the extra deadline window. This is negative episode evidence: added waiting did not complete recovery. It does not establish that no retransmission arrived after expiry, since arrival watches close then.

The candidate was intentionally rejected before the second window finished. Fixed52 restoration was verified with applied revision 5 and no active gap. The controller then terminated on the intentional external mode change. The partial ABBA set remains incomplete and supports no aggregate causal performance claim; failed startup and partial windows were retained. Normal GUI quit and both exact process exits were verified before logs were archived.

A separate read-only source review ruled out a missed-flush ordering bug: NACK admission precedes output draining, and the subsequent receive wait is capped at 1 ms. Admission still does not prove encrypted UDP delivery. The next app-only direction under assessment is earlier existing keyframe feedback after confirmed FEC loss, while preserving reference validity and the original reorder deadline. No new candidate is enabled.
