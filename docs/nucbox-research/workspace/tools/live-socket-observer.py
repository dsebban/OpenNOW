#!/usr/bin/env python3
"""Attribute UDP queue/drop metadata to sockets owned by one process."""

import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import re
import time


def process_start_ticks(text):
    fields = text.rsplit(") ", 1)[1].split()
    return int(fields[19])


def parse_udp_table(text, owned_inodes, family):
    result = {}
    for line in text.splitlines()[1:]:
        fields = line.split()
        if len(fields) < 13:
            continue
        try:
            inode = int(fields[9])
            if inode not in owned_inodes:
                continue
            result[inode] = {
                "family": family,
                "localPort": int(fields[1].rsplit(":", 1)[1], 16),
                "rxQueueBytes": int(fields[4].split(":")[1], 16),
                "socketDrops": int(fields[-1]),
            }
        except (ValueError, IndexError):
            continue
    return result


def owned_sockets(directory):
    result = set()
    failures = 0
    with os.scandir(directory) as entries:
        for index, entry in enumerate(entries):
            if index >= 4096:
                return result, failures + 1
            try:
                match = re.fullmatch(r"socket:\[(\d+)\]", os.readlink(entry.path))
                if match:
                    result.add(int(match[1]))
            except FileNotFoundError:
                continue
            except OSError:
                failures += 1
    return result, failures


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat(timespec="milliseconds")


def sample_process(pid, start_ticks):
    proc = Path("/proc") / str(pid)
    if process_start_ticks((proc / "stat").read_text()) != start_ticks:
        raise ProcessLookupError("process identity changed")
    before, errors_before = owned_sockets(proc / "fd")
    tables = {}
    for name, family in (("udp", 4), ("udp6", 6)):
        tables.update(parse_udp_table((proc / "net" / name).read_text(), before, family))
    after, errors_after = owned_sockets(proc / "fd")
    if process_start_ticks((proc / "stat").read_text()) != start_ticks:
        raise ProcessLookupError("process identity changed")
    return {inode: value for inode, value in tables.items() if inode in after}, errors_before + errors_after


def drop_change(previous, current):
    if previous is None:
        return None
    return current - previous if current >= previous else None


def write_private(path, value):
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | getattr(os, "O_NOFOLLOW", 0), 0o600)
    with os.fdopen(descriptor, "w") as file:
        json.dump(value, file, separators=(",", ":"), allow_nan=False)
        file.write("\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--pid", type=int, required=True)
    parser.add_argument("--duration", type=int, required=True, help="1 to 3600 seconds")
    parser.add_argument("--output", type=Path, required=True, help="New private JSON file")
    args = parser.parse_args()
    if args.pid <= 0 or not 1 <= args.duration <= 3600:
        parser.error("positive pid and duration 1 to 3600 required")
    if args.output.exists() or args.output.is_symlink():
        parser.error("output must be a new file")
    os.umask(0o077)
    try:
        start_ticks = process_start_ticks((Path("/proc") / str(args.pid) / "stat").read_text())
    except (OSError, ValueError, IndexError):
        parser.error("target process is unavailable")
    salt = os.urandom(16)
    identities = {}
    previous = {}
    totals = {}
    start = time.monotonic()
    deadline = start + args.duration
    result = {"schema": 1, "pid": args.pid, "startUtc": utc(), "requestedSeconds": args.duration,
              "samples": [], "complete": True, "invalidReason": None}
    try:
        for tick in range(args.duration):
            delay = start + tick - time.monotonic()
            if delay > 0:
                time.sleep(delay)
            if time.monotonic() >= deadline:
                break
            rows, errors = sample_process(args.pid, start_ticks)
            if len(rows) > 256 or len(identities) + len(set(rows) - set(identities)) > 4096:
                result["complete"] = False
                result["invalidReason"] = "socket_capacity_exceeded"
                break
            sample = {"utc": utc(), "elapsedSeconds": time.monotonic() - start,
                      "ownershipReadErrors": errors, "valid": errors == 0, "sockets": []}
            for inode, row in rows.items():
                if inode not in identities:
                    identities[inode] = hashlib.blake2b(str(inode).encode(), key=salt, digest_size=8).hexdigest()
                socket_id = identities[inode]
                prior = previous.get(inode) if errors == 0 else None
                change = drop_change(prior, row["socketDrops"])
                reset = prior is not None and change is None
                sample["sockets"].append({"socketId": socket_id, **row,
                                          "dropDelta": change, "counterReset": reset})
                aggregate = totals.setdefault(socket_id, {"family": row["family"], "localPort": row["localPort"],
                    "observedDropDelta": 0, "maxRxQueueBytes": 0, "samples": 0, "counterResets": 0,
                    "firstObservedDrops": row["socketDrops"], "firstUtc": sample["utc"]})
                aggregate["samples"] += 1
                aggregate["lastUtc"] = sample["utc"]
                aggregate["lastObservedDrops"] = row["socketDrops"]
                aggregate["maxRxQueueBytes"] = max(aggregate["maxRxQueueBytes"], row["rxQueueBytes"])
                aggregate["counterResets"] += reset
                if change is not None:
                    aggregate["observedDropDelta"] += change
            previous = {inode: row["socketDrops"] for inode, row in rows.items()} if errors == 0 else {}
            result["samples"].append(sample)
    except KeyboardInterrupt:
        result["complete"] = False
        result["invalidReason"] = "interrupted"
    except ProcessLookupError:
        result["complete"] = False
        result["invalidReason"] = "process_identity_changed"
    except (OSError, ValueError, IndexError):
        result["complete"] = False
        result["invalidReason"] = "process_read_unavailable"
    result["endUtc"] = utc()
    result["elapsedSeconds"] = time.monotonic() - start
    result["socketTotals"] = [{"socketId": socket_id, **value} for socket_id, value in totals.items()]
    result["limits"] = [
        "UDP namespace tables are filtered by the exact process fd socket inodes before and after the table read.",
        "Socket IDs are salted within this run. No local/remote IP addresses or payloads are stored.",
        "First samples are cumulative counters, not drops attributable to this observation interval.",
        "Missing or newly opened sockets have null interval delta. Short-lived sockets between samples can be missed.",
        "A socket first observed after an ownership gap starts a new measured interval. PID reuse aborts observation.",
        "Observed drops are process-socket receive drops; absent sender data and downstream decode/presentation loss are different measurements.",
    ]
    write_private(args.output, result)
    print(json.dumps({"samples": len(result["samples"]), "socketCount": len(totals),
                      "complete": result["complete"], "invalidReason": result["invalidReason"]}))


if __name__ == "__main__":
    main()
