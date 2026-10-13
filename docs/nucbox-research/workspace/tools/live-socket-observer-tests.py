#!/usr/bin/env python3
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("observer", Path(__file__).with_name("live-socket-observer.py"))
observer = importlib.util.module_from_spec(spec)
spec.loader.exec_module(observer)


class SocketParserTests(unittest.TestCase):
    def test_namespace_rows_are_filtered_by_owned_inode(self):
        fixture = "sl local_address rem_address st tx_queue:rx_queue tr:tm retr uid timeout inode ref ptr drops\n 1: 0100007F:BF6D 00000000:0000 07 00000000:00000A00 00:00000000 00000000 1000 0 123 2 00000000 9\n 2: 0100007F:9999 00000000:0000 07 00000000:00000000 00:00000000 00000000 0 0 456 2 00000000 999\n"
        rows = observer.parse_udp_table(fixture, {123}, 4)
        self.assertEqual(rows, {123: {"family": 4, "localPort": 49005, "rxQueueBytes": 2560, "socketDrops": 9}})
        self.assertNotIn("address", str(rows))

    def test_ipv6_and_malformed_rows(self):
        fixture = "header\n 1: 00000000000000000000000001000000:BF6E 00000000000000000000000000000000:0000 07 00000000:00000010 00:00000000 00000000 1000 0 789 2 00000000 0\n malformed\n"
        self.assertEqual(observer.parse_udp_table(fixture, {789}, 6)[789]["localPort"], 49006)
        self.assertEqual(observer.parse_udp_table(fixture, set(), 6), {})

    def test_drop_intervals_do_not_count_initial_or_reset_counters(self):
        self.assertIsNone(observer.drop_change(None, 100))
        self.assertEqual(observer.drop_change(100, 105), 5)
        self.assertIsNone(observer.drop_change(105, 0))

    def test_start_ticks_survive_spaces_and_parentheses_in_comm(self):
        fields = ['S'] + ['0'] * 18 + ['123456'] + ['0'] * 5
        self.assertEqual(observer.process_start_ticks('77 (app (name)) ' + ' '.join(fields)), 123456)


if __name__ == "__main__":
    unittest.main()
