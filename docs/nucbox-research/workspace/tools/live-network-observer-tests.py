#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("observer", Path(__file__).with_name("live-network-observer.py"))
observer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(observer)


class ParserTests(unittest.TestCase):
    def test_udp_allowlist_and_counter_reset(self):
        text = "Udp: InDatagrams NoPorts InErrors OutDatagrams RcvbufErrors SndbufErrors InCsumErrors MemErrors\nUdp: 15 2 3 7 4 5 6 0\n"
        self.assertEqual(observer.parse_udp(text)["RcvbufErrors"], 4)
        self.assertNotIn("InDatagrams", observer.parse_udp(text))
        self.assertEqual(observer.counter_delta({"x": 4}, {"x": 2}), {"x": None})
        self.assertEqual(observer.parse_udp("Udp: InErrors\n"), {})

    def test_pressure_and_memory(self):
        self.assertEqual(observer.parse_pressure("some avg10=1.25 avg60=0.20 avg300=0.00 total=99\nfull avg10=nan total=0\n"),
                         {"some": {"avg10": 1.25, "avg60": .2, "avg300": 0., "total": 99}, "full": {"total": 0}})
        self.assertEqual(observer.parse_meminfo("MemAvailable: 123 kB\nSwapFree: 456 kB\nSecret: 5 kB\n"),
                         {"MemAvailableBytes": 123 * 1024, "SwapFreeBytes": 456 * 1024})

    def test_ping(self):
        self.assertEqual(observer.parse_ping("64 bytes time=1.72 ms"), 1.72)
        self.assertIsNone(observer.parse_ping("timeout"))

    def test_router_numeric_filter(self):
        sample = observer.parse_router("cpu 1 2 3 40 5 6 7 8 90 91\nlink pppoe-wan 100 200 3 4\nqdisc pppoe-wan\n Sent 50 bytes 4 pkt (dropped 7, overlimits 8 requeues 9)\nlink wlan0 999 999 999 999\nsecret arbitrary text\n")
        self.assertEqual(sample["cpuTotalTicks"], 72)
        self.assertEqual(sample["cpuIdleTicks"], 45)
        self.assertEqual(sample["qdisc"]["pppoe-wan"]["dropped"], 7)
        self.assertEqual(set(sample["interfaces"]), {"pppoe-wan"})
        self.assertNotIn("secret", sample)

    def test_remote_command_is_bounded_and_read_only(self):
        command = observer.router_command(12)
        self.assertIn('"$i" -lt 3', command)
        self.assertIn("sleep 5", command)
        self.assertNotIn("uci", command)
        self.assertNotIn("lan1", command)


if __name__ == "__main__":
    unittest.main()
