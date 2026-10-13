#!/usr/bin/env python3
"""Bounded metadata-only host/Flint observation. No configuration writes."""

import argparse
import datetime
import json
import math
import os
from pathlib import Path
import queue
import re
import subprocess
import threading
import time


UDP_KEYS = ("InErrors", "RcvbufErrors", "SndbufErrors", "InCsumErrors", "NoPorts", "MemErrors")
INTERFACES = ("pppoe-wan", "ifb4pppoe-wan")


def parse_udp(text):
    lines = [line.split()[1:] for line in text.splitlines() if line.startswith("Udp:")]
    if len(lines) != 2 or len(lines[0]) != len(lines[1]):
        return {}
    values = dict(zip(lines[0], lines[1]))
    return {key: int(values[key]) for key in UDP_KEYS if values.get(key, "").isdigit()}


def parse_pressure(text):
    result = {}
    for line in text.splitlines():
        fields = line.split()
        if not fields or fields[0] not in ("some", "full"):
            continue
        values = {}
        for field in fields[1:]:
            key, separator, value = field.partition("=")
            if separator and key in ("avg10", "avg60", "avg300", "total"):
                try:
                    number = int(value) if key == "total" else float(value)
                    if math.isfinite(number) and number >= 0:
                        values[key] = number
                except ValueError:
                    pass
        result[fields[0]] = values
    return result


def parse_meminfo(text):
    result = {}
    for key in ("MemAvailable", "SwapFree"):
        match = re.search(rf"^{key}:\s+(\d+)\s+kB$", text, re.M)
        if match:
            result[key + "Bytes"] = int(match[1]) * 1024
    return result


def parse_ping(text):
    match = re.search(r"time[=<]([0-9.]+)\s*ms", text)
    return float(match[1]) if match else None


def parse_router(text):
    result = {"interfaces": {}, "qdisc": {}}
    active = None
    for line in text.splitlines():
        fields = line.split()
        if fields and fields[0] == "cpu" and len(fields) >= 9:
            if all(value.isdigit() for value in fields[1:9]):
                ticks = list(map(int, fields[1:9]))
                result["cpuTotalTicks"] = sum(ticks)
                result["cpuIdleTicks"] = ticks[3] + ticks[4]
        elif len(fields) == 6 and fields[0] == "link" and fields[1] in INTERFACES:
            if all(value.isdigit() for value in fields[2:]):
                result["interfaces"][fields[1]] = dict(zip(
                    ("rxBytes", "txBytes", "rxDropped", "txDropped"), map(int, fields[2:])))
        elif len(fields) == 2 and fields[0] == "qdisc" and fields[1] in INTERFACES:
            active = fields[1]
        elif active:
            match = re.search(r"\(dropped (\d+), overlimits (\d+) requeues (\d+)\)", line)
            if match:
                result["qdisc"].setdefault(active, {"dropped": 0, "overlimits": 0, "requeues": 0})
                for key, value in zip(("dropped", "overlimits", "requeues"), match.groups()):
                    result["qdisc"][active][key] += int(value)
    return result


def counter_delta(before, after):
    return {key: value - before[key] if value >= before[key] else None
            for key, value in after.items() if key in before}


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="milliseconds")


def read_host():
    result = {}
    for field, path, parser in (
        ("udp", "/proc/net/snmp", parse_udp),
        ("memoryPressure", "/proc/pressure/memory", parse_pressure),
        ("memory", "/proc/meminfo", parse_meminfo),
    ):
        try:
            result[field] = parser(Path(path).read_text())
        except OSError:
            result[field] = None
    return result


def router_command(duration):
    count = math.ceil(duration / 5)
    return f"""i=0
while [ "$i" -lt {count} ]; do
  printf 'begin\\n'
  awk '/^cpu / {{print; exit}}' /proc/stat
  for dev in pppoe-wan ifb4pppoe-wan; do
    if [ -d "/sys/class/net/$dev" ]; then
      printf 'link %s' "$dev"
      for counter in rx_bytes tx_bytes rx_dropped tx_dropped; do
        printf ' %s' "$(cat "/sys/class/net/$dev/statistics/$counter")"
      done
      printf '\\nqdisc %s\\n' "$dev"
      tc -s qdisc show dev "$dev" 2>/dev/null
    fi
  done
  printf 'end\\n'
  i=$((i + 1))
  if [ "$i" -lt {count} ]; then sleep 5; fi
done"""


def router_reader(stream, output, stopped):
    block = []
    for line in stream:
        if stopped.is_set():
            return
        if line.strip() == "begin":
            block = []
        elif line.strip() == "end":
            sample = {"utc": utc(), **parse_router("".join(block))}
            try:
                output.put_nowait(sample)
            except queue.Full:
                return
        elif len(block) < 256 and len(line) <= 4096:
            block.append(line)


def private_output(path, value):
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0)
    descriptor = os.open(path, flags, 0o600)
    with os.fdopen(descriptor, "w") as file:
        json.dump(value, file, separators=(",", ":"), allow_nan=False)
        file.write("\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--duration", type=int, required=True, help="1 to 3600 seconds")
    parser.add_argument("--output", type=Path, required=True, help="New private JSON file")
    parser.add_argument("--flint", action="store_true", help="One read-only SSH session, sampled every 5s")
    args = parser.parse_args()
    if not 1 <= args.duration <= 3600:
        parser.error("duration must be 1 to 3600 seconds")
    if args.output.exists() or args.output.is_symlink():
        parser.error("output must be a new file")
    os.umask(0o077)
    router_queue = queue.Queue(maxsize=64)
    stopped = threading.Event()
    router = None
    if args.flint:
        router = subprocess.Popen([
            "ssh", "-T", "-i", "/home/gamer/.ssh/flint_ed25519",
            "-o", "BatchMode=yes", "-o", "StrictHostKeyChecking=yes",
            "-o", "ConnectTimeout=3", "-o", "ServerAliveInterval=5",
            "-o", "ServerAliveCountMax=1", "root@192.168.8.1",
            router_command(args.duration),
        ], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
        threading.Thread(target=router_reader, args=(router.stdout, router_queue, stopped), daemon=True).start()
    start = time.monotonic()
    deadline = start + args.duration
    result = {"schema": 1, "startUtc": utc(), "requestedSeconds": args.duration,
              "host": [], "router": [], "interrupted": False}
    try:
        for tick in range(args.duration):
            wait = start + tick - time.monotonic()
            if wait > 0:
                time.sleep(wait)
            if time.monotonic() >= deadline:
                break
            sample = {"utc": utc(), "elapsedSeconds": time.monotonic() - start, **read_host()}
            try:
                ping = subprocess.run(["ping", "-n", "-c", "1", "-W", "1", "192.168.8.1"],
                                      capture_output=True, text=True, timeout=min(1.05, deadline - time.monotonic()),
                                      env={**os.environ, "LC_ALL": "C"})
                sample["gatewayRttMs"] = parse_ping(ping.stdout)
                sample["gatewayReply"] = sample["gatewayRttMs"] is not None
                sample["gatewayProbeValid"] = ping.returncode in (0, 1)
            except (OSError, subprocess.TimeoutExpired):
                sample["gatewayReply"] = False
                sample["gatewayRttMs"] = None
                sample["gatewayProbeValid"] = False
            result["host"].append(sample)
            while not router_queue.empty():
                result["router"].append(router_queue.get_nowait())
    except KeyboardInterrupt:
        result["interrupted"] = True
    finally:
        stopped.set()
        if router:
            router.terminate()
            try:
                router.wait(timeout=2)
            except subprocess.TimeoutExpired:
                router.kill()
                router.wait(timeout=2)
        while not router_queue.empty():
            result["router"].append(router_queue.get_nowait())
    result["endUtc"] = utc()
    result["elapsedSeconds"] = time.monotonic() - start
    hosts = result["host"]
    result["gatewayReplies"] = sum(sample["gatewayReply"] for sample in hosts)
    result["gatewayProbes"] = len(hosts)
    result["gatewayValidProbes"] = sum(sample["gatewayProbeValid"] for sample in hosts)
    result["udpDelta"] = counter_delta(hosts[0].get("udp") or {}, hosts[-1].get("udp") or {}) if hosts else {}
    result["routerRequested"] = args.flint
    result["routerSamples"] = len(result["router"])
    previous = None
    for sample in result["router"]:
        if previous and "cpuTotalTicks" in previous and "cpuTotalTicks" in sample:
            total = sample["cpuTotalTicks"] - previous["cpuTotalTicks"]
            idle = sample["cpuIdleTicks"] - previous["cpuIdleTicks"]
            sample["cpuPercent"] = 100 * (total - idle) / total if total > 0 and 0 <= idle <= total else None
        previous = sample
    private_output(args.output, result)
    print(json.dumps({"hostSamples": len(hosts), "routerSamples": result["routerSamples"],
                      "gatewayReplies": result["gatewayReplies"], "elapsedSeconds": result["elapsedSeconds"]}))


if __name__ == "__main__":
    main()
