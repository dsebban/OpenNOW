#!/usr/bin/env python3
import argparse, os, subprocess
from pathlib import Path
root=Path('/home/gamer/dev/gfn-client-research')
parser=argparse.ArgumentParser(description='Launch the custom native Wayland client.')
parser.add_argument('--activate-existing', action='store_true', help='Use native single-instance activation when the workspace Qt client is already running.')
parser.add_argument('--foreground', action='store_true', help='Keep the native process attached to a caller such as Steam.')
args=parser.parse_args()
active=set()
for entry in Path('/proc').iterdir():
 try:
  exe=(entry/'exe').resolve()
  if str(exe).startswith(str(root)) and exe.name in ('opennow-qt','opennow-core'):
   active.add(exe.name)
 except OSError: pass
if active and not (args.activate_existing and 'opennow-qt' in active):
 raise SystemExit('Workspace Qt/core still active; close normally before launch.')
env=os.environ.copy()
result=subprocess.run(['systemctl','--user','show-environment'],text=True,capture_output=True,check=True)
for line in result.stdout.splitlines():
 key,_,value=line.partition('=')
 if key in ('DISPLAY','WAYLAND_DISPLAY','XAUTHORITY','XDG_SESSION_TYPE','XDG_CURRENT_DESKTOP','DBUS_SESSION_BUS_ADDRESS','XDG_RUNTIME_DIR'): env[key]=value
for key in ('OPENNOW_LIVE_TELEMETRY','OPENNOW_EMBEDDED_PUBLISH_POLL_MS','OPENNOW_CONTINUOUS_VIDEO_UPDATE','OPENNOW_NATIVE_VIDEO_BACKEND','OPENNOW_EMBEDDED_READY_WAKE','QSG_RENDER_LOOP','QT_QPA_UPDATE_IDLE_TIME'): env.pop(key,None)
if env.get('XDG_SESSION_TYPE')!='wayland': raise SystemExit('Current session is not Wayland')
env['QT_QPA_PLATFORM']='wayland'
if args.foreground:
 os.execve(str(root/'tools/run-native.sh'), [str(root/'tools/run-native.sh')], env)
proc=subprocess.Popen([str(root/'tools/run-native.sh')],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,start_new_session=True)
print('Requested native activation via PID' if active else 'Launched native Wayland client PID',proc.pid)
