# NucBox research workspace

`workspace/` preserves the outer `gfn-client-research` repository's tools,
instructions, references and reports alongside this OpenNOW experiment branch.
`manifest.json` records the source commit and each exported file's SHA-256.
Private runtime logs, captures, credentials, toolchains and binaries are excluded.

The scripts expect the original workspace layout. To restore it, copy the
contents of `workspace/` into a separate research directory, then place an
OpenNOW checkout of this branch at `vendor/OpenNOW` beneath that directory.
Follow its README and machine constraints before building or operating GFN.
Machine-specific skills and account state are not supplied by this snapshot.

To refresh the snapshot from the outer workspace, run
`python3 tools/export-research-workspace.py`. It copies only Git-selected files
and verifies their hashes. Review both repositories before committing.

The working-profile record and latest playback analysis are under
`workspace/references/gfn-working-profile.md` and
`workspace/research/last-night-playback-20261013.md`. Experimental pacing,
FEC deadlines and preemptive keyframe code are retained for reproducibility;
they are not accepted performance fixes and remain disabled in ordinary playback.
