#!/usr/bin/env python3
"""Analyze OPENNOW_FRAME_STAGE_LOG output (frame-stage-log lines in launch.log).

Per stage (enqueue/decoded/publish):
  - consecutive wall-clock interval histogram (the burst signature)
  - PTS interval histogram (sender cadence sanity check)
Joined by pts_us:
  - per-frame enqueue->decoded latency
  - per-frame decoded->publish latency
  - burst pairing: correlation between short and long intervals

Usage: stage-log-analysis.py [launch.log] [--since SECONDS]
"""
import json
import re
import statistics
import sys
from collections import defaultdict

LINE = re.compile(r"frame-stage-log stage=(\w+) pts_us=(\d+) wall_us=(\d+)")


def pct(values, p):
    if not values:
        return float("nan")
    vs = sorted(values)
    i = min(len(vs) - 1, max(0, round((p / 100) * (len(vs) - 1))))
    return vs[i]


def hist_ms(intervals_us):
    bins = [0, 5, 8, 11, 14, 16, 18, 20, 22, 25, 30, 40, 60, 100, 250, 1000]
    counts = [0] * len(bins)
    for iv in intervals_us:
        ms = iv / 1000
        for i, b in enumerate(bins):
            if ms <= b:
                counts[i] += 1
                break
        else:
            counts[-1] += 1
    total = len(intervals_us)
    out = []
    prev = 0
    for b, c in zip(bins, counts):
        if c:
            out.append(f"({prev},{b}]ms:{c}({100*c/total:.1f}%)")
        prev = b
    return " ".join(out)


def main():
    path = sys.argv[1] if len(sys.argv) > 1 else ".runtime/logs/launch.log"
    since = None
    if "--since" in sys.argv:
        since = float(sys.argv[sys.argv.index("--since") + 1])

    stages = defaultdict(list)  # stage -> [(pts_us, wall_us)]
    for line in open(path, errors="replace"):
        m = LINE.search(line)
        if not m:
            continue
        stage, pts, wall = m.group(1), int(m.group(2)), int(m.group(3))
        stages[stage].append((pts, wall))

    for stage in stages:
        stages[stage].sort(key=lambda t: t[1])

    summary = {}
    for stage in ("enqueue", "decoded", "publish"):
        rows = stages.get(stage, [])
        if len(rows) < 3:
            print(f"stage={stage}: only {len(rows)} samples")
            continue
        if since:
            t_end = rows[-1][1]
            rows = [r for r in rows if r[1] >= t_end - since * 1e6]
        walls = [w for _, w in rows]
        ptsv = [p for p, _ in rows]
        iv = [b - a for a, b in zip(walls, walls[1:])]
        piv = [b - a for a, b in zip(ptsv, ptsv[1:])]
        late25 = sum(1 for x in iv if x > 25_000)
        early14 = sum(1 for x in iv if x < 14_000)
        print(f"\n=== stage={stage} n={len(rows)} window={(walls[-1]-walls[0])/1e6:.1f}s")
        print(f"  wall intervals: p50={pct(iv,50)/1000:.2f}ms p95={pct(iv,95)/1000:.2f}ms "
              f"p99={pct(iv,99)/1000:.2f}ms max={max(iv)/1000:.1f}ms")
        print(f"  pts intervals : p50={pct(piv,50)/1000:.2f}ms p95={pct(piv,95)/1000:.2f}ms "
              f"max={max(piv)/1000:.1f}ms")
        print(f"  >25ms: {late25} ({100*late25/len(iv):.2f}%)   <14ms: {early14} ({100*early14/len(iv):.1f}%)")
        print(f"  hist: {hist_ms(iv)}")
        summary[stage] = {"n": len(rows), "iv_p50_ms": pct(iv, 50) / 1000,
                          "iv_p95_ms": pct(iv, 95) / 1000, "iv_max_ms": max(iv) / 1000,
                          "late25_pct": 100 * late25 / len(iv)}

    # per-frame pipeline latencies joined by PTS
    by_pts = {}
    for stage, rows in stages.items():
        for pts, wall in rows:
            by_pts.setdefault(pts, {})[stage] = wall
    joined = [(p, s) for p, s in by_pts.items() if "enqueue" in s and "decoded" in s]
    if joined:
        d1 = sorted(s["decoded"] - s["enqueue"] for _, s in joined)
        print(f"\nenqueue->decoded: n={len(d1)} p50={pct(d1,50)/1000:.2f}ms "
              f"p95={pct(d1,95)/1000:.2f}ms max={max(d1)/1000:.1f}ms")
    j2 = [(p, s) for p, s in by_pts.items() if "decoded" in s and "publish" in s]
    if j2:
        d2 = sorted(s["publish"] - s["decoded"] for _, s in j2)
        print(f"decoded->publish: n={len(d2)} p50={pct(d2,50)/1000:.2f}ms "
              f"p95={pct(d2,95)/1000:.2f}ms max={max(d2)/1000:.1f}ms")
    j3 = [(p, s) for p, s in by_pts.items() if "enqueue" in s and "publish" in s]
    if j3:
        d3 = sorted(s["publish"] - s["enqueue"] for _, s in j3)
        print(f"enqueue->publish: n={len(d3)} p50={pct(d3,50)/1000:.2f}ms "
              f"p95={pct(d3,95)/1000:.2f}ms max={max(d3)/1000:.1f}ms")

    # burst pairing at publish: for each >25ms interval, was the previous <14ms? and what does the PTS gap say?
    pub = stages.get("publish")
    if pub and len(pub) > 3:
        rows = [r for r in pub if not since or r[1] >= pub[-1][1] - since * 1e6]
        walls = [w for _, w in rows]
        iv = [b - a for a, b in zip(walls, walls[1:])]
        pairs = sum(1 for i in range(1, len(iv)) if iv[i] > 25_000 and iv[i - 1] < 14_000)
        lates = sum(1 for x in iv if x > 25_000)
        print(f"\npublish late intervals preceded by <14ms pair: {pairs}/{lates}")

    print("\n" + json.dumps({"stages": summary}, indent=1))


if __name__ == "__main__":
    main()
