#!/usr/bin/env python3
"""Aggregate allowlisted native telemetry after a recorded trial start."""
import argparse, collections, json, pathlib, re, statistics, time
root=pathlib.Path(__file__).resolve().parents[1]
p=argparse.ArgumentParser();p.add_argument('name');p.add_argument('--since-ms',type=int,required=True);a=p.parse_args()
if not re.fullmatch(r'[A-Za-z0-9_-]{1,60}',a.name):p.error('Use a simple trial label')
directory=root/'.runtime/benchmarks';samples=collections.defaultdict(list);drops=collections.Counter();events=collections.Counter();until_ms=int(time.time()*1000)
keys={'framesPerSecond','pingMs','jitterMs','packetLossPercent','bitrateMbps','decodeTimeMs','decoderResidenceMs',
      'decodeTimings.call.p50','decodeTimings.call.p95','decodeTimings.call.max',
      'decodeTimings.residence.p50','decodeTimings.residence.p95','decodeTimings.residence.max'}
for path in (root/'.runtime/config/OpenNOW/diagnostics').glob('native-streamer*.log'):
    for line in path.read_text(errors='replace').splitlines():
        m=re.match(r'^(\d+) ',line)
        if not m or not a.since_ms<=int(m[1])<=until_ms:continue
        fields=dict(re.findall(r'([A-Za-z][A-Za-z0-9_.]*)=([^ ]+)',line))
        if fields.get('event')=='queue-dropped' and fields.get('unit')=='frames':
            if re.fullmatch(r'[a-z0-9-]{1,40}',fields.get('media','')) and fields.get('count','').isdigit():
                drops[fields['media']]+=int(fields['count'])
        if fields.get('event') in {'keyframe-request','recovery-required'}:events[fields['event']]+=1
        if fields.get('type')!='telemetry':continue
        for k in keys:
            if re.fullmatch(r'-?\d+(\.\d+)?',fields.get(k,'')):samples[k].append(float(fields[k]))
report={'trial':a.name,'since_ms':a.since_ms,'until_ms':until_ms,'scope':'Native telemetry windows; not physical scanout or input-to-photon measurement',
        'queue_drops_by_source':dict(drops),'events':dict(events),'metrics':{}}
for k,v in samples.items():
    ordered=sorted(v);report['metrics'][k]={'samples':len(v),'median':statistics.median(v),'p95':ordered[min(len(v)-1,int(.95*len(v)))],'max':max(v)}
directory.mkdir(parents=True,exist_ok=True)
(directory/(a.name+'-summary.json')).write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report,indent=2))
