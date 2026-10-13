#!/usr/bin/env python3
"""Inspect classic PCAP headers only. Never emit captured payloads or keys."""
import argparse, collections, json, socket, statistics, struct

def spread(values):
    values = sorted(values)
    return {"median": statistics.median(values), "p95": values[round(.95*(len(values)-1))], "max": values[-1]} if values else None

def packets(path):
    with open(path, "rb") as f:
        header = f.read(24)
        formats = {b'\xd4\xc3\xb2\xa1': ('<', 1e6), b'\xa1\xb2\xc3\xd4': ('>', 1e6), b'\x4d\x3c\xb2\xa1': ('<', 1e9), b'\xa1\xb2\x3c\x4d': ('>', 1e9)}
        if len(header) != 24 or header[:4] not in formats:
            raise ValueError("Expected classic PCAP")
        endian, scale = formats[header[:4]]
        link = struct.unpack(endian+'I', header[20:24])[0]
        if link != 1:
            raise ValueError(f"Expected Ethernet capture, got link type {link}")
        while record := f.read(16):
            if len(record) != 16: raise ValueError("Truncated record header")
            sec, frac, size, original = struct.unpack(endian+'IIII', record)
            if size > 262144: raise ValueError("Oversized record")
            packet = f.read(size)
            if len(packet) != size: raise ValueError("Truncated packet")
            yield sec+frac/scale, packet, original

def analyze(path):
    flows = {}
    stun_pending = {}; stun_rtts = []
    stun_by_initiator = collections.defaultdict(list)
    count = fragments = truncated = 0
    first = last = None
    for now, packet, original in packets(path):
        count += 1; truncated += len(packet) < original
        first = now if first is None else first; last = now
        if len(packet) < 34: continue
        ethertype = struct.unpack('!H', packet[12:14])[0]; offset = 14
        while ethertype in (0x8100, 0x88a8) and len(packet) >= offset+4:
            ethertype = struct.unpack('!H', packet[offset+2:offset+4])[0]; offset += 4
        if ethertype != 0x0800 or len(packet) < offset+20: continue
        ip = packet[offset:]; ihl = (ip[0]&15)*4
        if ip[0]>>4 != 4 or ihl < 20 or len(ip) < ihl+8: continue
        if struct.unpack('!H', ip[6:8])[0]&0x3fff:
            fragments += 1; continue
        proto = ip[9]
        if proto not in (6,17): continue
        transport = ip[ihl:]; source_port, dest_port = struct.unpack('!HH',transport[:4])
        key = (socket.inet_ntoa(ip[12:16]),source_port,socket.inet_ntoa(ip[16:20]),dest_port,proto)
        flow = flows.setdefault(key, {"packets":0,"wire_bytes":0,"times":[],"sizes":collections.Counter(),"protocol_candidates":collections.Counter(),"gs_frames":set(),"gs_fec_packets":0,"rtp_sequences":collections.defaultdict(list)})
        flow['packets'] += 1; flow['wire_bytes'] += original; flow['times'].append(now); flow['sizes'][original] += 1
        if proto == 6:
            header_size = (transport[12]>>4)*4 if len(transport)>=20 else 0
            body = transport[header_size:] if header_size>=20 else b''
            if body.startswith((b'RTSP/',b'OPTIONS ',b'SETUP ',b'ANNOUNCE ',b'PLAY ',b'TEARDOWN ')):
                flow['protocol_candidates']['RTSP_start_line'] += 1
            elif len(body)>=5 and 20<=body[0]<=23 and body[1:3] in (b'\x03\x01',b'\x03\x02',b'\x03\x03'):
                flow['protocol_candidates']['TLS_record_header'] += 1
            continue
        udp_length = struct.unpack('!H',transport[4:6])[0]
        body = transport[8:udp_length]
        if len(body)>=20 and body[4:8] == b'\x21\x12\xa4\x42':
            flow['protocol_candidates']['STUN'] += 1
            message_type = struct.unpack('!H',body[:2])[0]
            transaction = body[8:20]
            if message_type == 0x0001:
                stun_pending.setdefault((transaction,*key[:4]),now)
            elif message_type == 0x0101:
                sent = stun_pending.pop((transaction,key[2],key[3],key[0],key[1]),None)
                if sent is not None and 0<=now-sent<=5:
                    rtt=(now-sent)*1000
                    stun_rtts.append(rtt)
                    stun_by_initiator[key[2]].append(rtt)
            continue
        if len(body)>=13 and 20<=body[0]<=25 and body[1:3] in (b'\xfe\xfd',b'\xfe\xff',b'\xfe\xfc'):
            flow['protocol_candidates']['DTLS'] += 1; continue
        if len(body)<12 or body[0]>>6 != 2: continue
        pos = 12+4*(body[0]&15)
        if not body[0]&0x10 or len(body)<pos+4: continue
        profile, words = struct.unpack('!HH',body[pos:pos+4]); pos += 4
        if profile != 0x4753 or words*4 < 16 or len(body)<pos+words*4: continue
        extension = body[pos:pos+16]
        frame = struct.unpack('<I',extension[4:8])[0]
        fec = struct.unpack('<I',extension[12:16])[0]
        flow['protocol_candidates']['RTP_GS_0x4753'] += 1
        flow['gs_frames'].add(frame)
        flow['gs_fec_packets'] += bool((fec>>4)&255 and (fec>>12)&1023 >= (fec>>22)&1023)
        ssrc = struct.unpack('!I',body[8:12])[0]; seq = struct.unpack('!H',body[2:4])[0]
        flow['rtp_sequences'][ssrc].append(seq)
    results=[]
    for key, f in flows.items():
        gaps=[]
        for sequences in f['rtp_sequences'].values():
            extended=[]; highest=sequences[0]
            for seq in sequences:
                candidate=seq+round((highest-seq)/65536)*65536
                highest=max(highest,candidate); extended.append(candidate)
            unique=set(extended)
            gaps.append({"observed":len(extended),"duplicates":len(extended)-len(unique),"unseen_slots_in_captured_range":max(unique)-min(unique)+1-len(unique)})
        results.append({"source":f'{key[0]}:{key[1]}',"destination":f'{key[2]}:{key[3]}',"transport":'UDP' if key[4]==17 else 'TCP',"packets":f['packets'],"wire_bytes":f['wire_bytes'],"packet_sizes":f['sizes'].most_common(5),"interarrival_ms":spread([(b-a)*1000 for a,b in zip(f['times'],f['times'][1:])]),"protocol_candidates":dict(f['protocol_candidates']),"gs_unique_frame_ids":len(f['gs_frames']),"gs_fec_packets":f['gs_fec_packets'],"rtp_sequence_observations":gaps})
    return {"capture_point":"Flint phy1-ap0, before Wi-Fi delivery to the client","packets":count,"truncated":truncated,"ipv4_fragments":fragments,"duration_seconds":last-first if first is not None else 0,"stun_binding_rtt_ms":spread(stun_rtts),"stun_matched_transactions":len(stun_rtts),"stun_by_initiator":{ip:{"samples":len(v),"rtt_ms":spread(v)} for ip,v in stun_by_initiator.items()},"limitations":["Header identification does not authenticate or decrypt SRTP.","Unseen sequence slots are not a measurement of rendered frame loss.","Capture timing at the router excludes later client Wi-Fi, decode, and presentation delays.","STUN timings may include retransmissions; server-initiated request/response intervals at this LAN capture point measure the local client leg."],"flows":sorted(results,key=lambda x:x['wire_bytes'],reverse=True)}

if __name__ == '__main__':
    parser=argparse.ArgumentParser(); parser.add_argument('pcap'); args=parser.parse_args()
    print(json.dumps(analyze(args.pcap),indent=2))
