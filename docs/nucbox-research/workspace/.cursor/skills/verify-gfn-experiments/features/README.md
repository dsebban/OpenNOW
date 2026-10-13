# GFN verification feature map

| Feature | File | Evidence |
| --- | --- | --- |
| Saved-account launch, resume, game input, statistics, close | [Playback](playback.md) | Fresh GUI action/result screenshots and process ownership |
| Real stream measurement and candidate acceptance | [Live measurement](live-measurement.md) | Private trial JSON, matched control comparison, decision |
| Network/socket observation and reversible network experiments | [Network comparison](network-comparison.md) | Time-aligned numeric observations and restored-state proof |
| Fixed-scene descriptive latency baseline | [Latency baseline](latency-baseline.md) | Repeated windows, typed percentile semantics, private JSON/markdown and retained preflight failures |

Read the relevant file before driving its feature. Mark each sub-feature `pass`, `fail`, or `skipped` in the proof, with an artifact path or concrete reason. A healthy launch does not cover measurement; a valid measurement does not imply an accepted improvement. A blocked mutation remains a recorded attempt and supplies no performance result.

For a skill smoke check, one real feature is sufficient. For an implementation claim, cover all affected entry points listed in the relevant feature file. Preserve proof after cleanup and keep private artifacts outside Git.
