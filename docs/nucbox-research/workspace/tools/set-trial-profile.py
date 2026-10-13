#!/usr/bin/env python3
"""Set a reproducible trial through OpenNOW's authenticated settings API.

Run only with the GUI closed. The original profile is retained privately.
"""
import argparse, json, os, pathlib
from probe_regions import Core

root=pathlib.Path(__file__).resolve().parents[1]
parser=argparse.ArgumentParser()
parser.add_argument('region', help='Exact name from authenticated region discovery')
parser.add_argument('--restore', action='store_true')
parser.add_argument('--codec',choices=['h264','h265','auto'],default='h265')
parser.add_argument('--color',choices=['8bit_420','10bit_420'],default='10bit_420')
args=parser.parse_args()
os.umask(0o077)
directory=root/'.runtime/benchmarks'
core=Core(directory/'bin/opennow-core',root/'.runtime/config/OpenNOW',directory/'core-private.log')
keys=['region','resolution','fps','colorQuality','codec','nativeVideoBackend','decoderPreference']
try:
    core.request('core.hello',{'protocolVersion':5,'shell':'qt','shellVersion':'1.0.2'})
    if core.request('auth.session.get').get('session') is None:
        raise RuntimeError('Normal saved-session restore required')
    discovery=core.request('network.regions.list')
    saved=directory/'original-settings.json'
    settings=core.request('settings.get')['settings']
    if not saved.exists():saved.write_text(json.dumps({k:settings[k] for k in keys},indent=2)+'\n')
    if args.restore:desired=json.loads(saved.read_text())
    else:
        candidates=[r for r in discovery['regions'] if r['name']==args.region]
        if len(candidates)!=1:raise RuntimeError('Region must match one discovered endpoint')
        desired={'region':candidates[0]['url'],'resolution':'1920x1080','fps':60,'codec':args.codec,
                 'colorQuality':args.color,'nativeVideoBackend':'auto','decoderPreference':'auto'}
    for key in keys:
        params={'key':key,'value':desired[key]}
        if key=='region':params['providerIdpId']=discovery['providerIdpId']
        core.request('settings.set',params)
    actual=core.request('settings.get')['settings']
    if any(actual[k]!=desired[k] for k in keys):raise RuntimeError('Settings readback mismatch')
    print('Trial profile verified: '+(args.region if not args.restore else 'original profile'))
    print(json.dumps({k:actual[k] for k in keys if k!='region'}))
finally:core.close()
