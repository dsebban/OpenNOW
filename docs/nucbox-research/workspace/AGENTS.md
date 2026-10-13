# GFN client research

Use `.cursor/skills/verify-gfn-experiments/SKILL.md` before validating or
benchmarking custom OpenNOW changes on this NucBox. The same skill is
discoverable at `/home/gamer/.codex/skills/verify-gfn-experiments/`.

The machine configuration and incident history live in
`/home/gamer/.codex/AGENTS.md`. Read its current observations before host or
Flint changes. The client source has additional instructions in
`vendor/OpenNOW/AGENTS.md`.

For resuming GFN or changing its display, stream, or network profile, read
[the user-accepted working profile](references/gfn-working-profile.md) first.
Preserve it during ordinary playback; it records the successful setup,
persistence limits, and the user's permanent SQM-off instruction.

For Flint 2 custom-firmware builds, features, or GFN network-stability research,
read [references/flint2-custom-firmware.md](references/flint2-custom-firmware.md).
The firmware reference is [pesa1234/MT6000_cust_build](https://github.com/pesa1234/MT6000_cust_build);
match published build metadata to the installed router before using its recipes.

This workspace is the NucBox itself: agents run locally on ChimeraOS and
`vendor/OpenNOW` is the working client checkout (fork `dsebban/OpenNOW`).
Edit, build, and test directly here — there is no separate remote target.

Only one operator may drive the shared GUI and saved-account session.
Independent agents may review source or work in isolated checkouts. Keep
builds and tests outside scored stream intervals. Credentials, raw captures,
and session logs stay out of Git; private evidence belongs in `.runtime/`.

The experiment skill defines measurement validity, matching controls,
rejection rules, cleanup, and the feature map. Update that source when a
verified entry point or measurement contract changes instead of duplicating
the procedure here.
