#!/usr/bin/env python3
"""Use OpenNOW's core and normal saved-session restore; emit no credential data."""
import argparse, datetime, json, os, pathlib, queue, statistics, subprocess, threading, time
from urllib.parse import urlsplit, urlunsplit

class Core:
    def __init__(self, executable, data_dir, diagnostics):
        self.responses=queue.Queue(); self.serial=0
        env=os.environ.copy()
        env.setdefault('DBUS_SESSION_BUS_ADDRESS',f'unix:path=/run/user/{os.getuid()}/bus')
        env.setdefault('XDG_RUNTIME_DIR',f'/run/user/{os.getuid()}')
        self.stderr=open(diagnostics,'a',encoding='utf8')
        self.process=subprocess.Popen([str(executable),'--data-dir',str(data_dir)],stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=self.stderr,text=True,env=env)
        def read():
            for line in self.process.stdout:
                if len(line)>1024*1024: self.responses.put(None); return
                try: value=json.loads(line)
                except json.JSONDecodeError: continue
                if value.get('type')=='response': self.responses.put(value)
            self.responses.put(None)
        threading.Thread(target=read,daemon=True).start()
    def request(self, method, params=None, timeout=60):
        self.serial+=1; request_id=f'benchmark-{self.serial}'
        self.process.stdin.write(json.dumps({'type':'request','id':request_id,'method':method,'params':params or {}})+'\n');self.process.stdin.flush()
        deadline=time.monotonic()+timeout
        while True:
            value=self.responses.get(timeout=max(.01,deadline-time.monotonic()))
            if value is None: raise RuntimeError('Core exited before the response')
            if value.get('id')!=request_id: continue
            if not value.get('ok'): raise RuntimeError('Core rejected '+method+'; see private diagnostics')
            return value['result']
    def close(self):
        if self.process.poll() is None:
            self.process.stdin.close()
            try:self.process.wait(timeout=10)
            except subprocess.TimeoutExpired:
                self.process.terminate()
                try:self.process.wait(timeout=5)
                except subprocess.TimeoutExpired:self.process.kill();self.process.wait()
        self.stderr.close()

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--rounds',type=int,default=5)
    parser.add_argument('--root',type=pathlib.Path,default=pathlib.Path(__file__).resolve().parents[1])
    args=parser.parse_args();root=args.root.resolve();os.umask(0o077)
    if not 1<=args.rounds<=20:parser.error('rounds must be 1..20')
    directory=root/'.runtime/benchmarks';directory.mkdir(parents=True,exist_ok=True)
    core=Core(directory/'bin/opennow-core',root/'.runtime/config/OpenNOW',directory/'core-private.log')
    try:
        core.request('core.hello',{'protocolVersion':5,'shell':'qt','shellVersion':'1.0.2'})
        session=core.request('auth.session.get')
        if session.get('session') is None:raise RuntimeError('Saved session needs normal interactive restoration')
        del session
        discovery=core.request('network.regions.list')
        regions=[]
        for item in discovery.get('regions',[]):
            parsed=urlsplit(item.get('url',''))
            if not parsed.hostname or parsed.scheme not in ('https','http'):continue
            if not parsed.hostname.endswith('.nvidiagrid.net'):raise RuntimeError('Unexpected provider endpoint; review discovery privately')
            port=f':{parsed.port}' if parsed.port else ''
            regions.append({'name':str(item.get('name','Unknown'))[:80],'url':urlunsplit((parsed.scheme,parsed.hostname+port,'/','',''))})
        if not regions:raise RuntimeError('No regional endpoints returned')
        rows={r['url']:{**r,'round_means_ms':[],'failed_rounds':0} for r in regions}
        print(f'Saved session restored; checking {len(regions)} discovered regions.',flush=True)
        for repeat in range(args.rounds):
            for start in range(0,len(regions),32):
                result=core.request('network.regions.ping',{'regions':regions[start:start+32]})
                for item in result.get('results',[]):
                    if not isinstance(item,dict) or item.get('url') not in rows:continue
                    row=rows[item['url']];value=item.get('pingMs')
                    if isinstance(value,(float,int)) and not isinstance(value,bool):row['round_means_ms'].append(value)
                    else:row['failed_rounds']+=1
            print(f'Completed round {repeat+1}/{args.rounds}',flush=True);time.sleep(.5)
        for row in rows.values():
            values=row['round_means_ms'];row['median_ms']=statistics.median(values) if values else None
            row['min_ms']=min(values) if values else None;row['max_ms']=max(values) if values else None
        ordered=sorted(rows.values(),key=lambda r:r['median_ms'] if r['median_ms'] is not None else float('inf'))
        report={'captured_utc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'authenticated_saved_session':True,'source':'OpenNOW network.regions.list + network.regions.ping','method':'One warmup plus three TCP connections per region per round; median of round means. Not UDP stream latency, jitter, loss or decoder performance.','rounds':args.rounds,'regions':ordered}
        (directory/'region-probes.json').write_text(json.dumps(report,indent=2)+'\n')
        for row in ordered:print(f"{row['name']}: median={row['median_ms']} ms, range={row['min_ms']}..{row['max_ms']}, failures={row['failed_rounds']}")
    finally:core.close()

if __name__=='__main__':main()
