"""Temporary local observer; never reads arbitrary user document contents."""
import json
from pathlib import Path
import sys
import frida

root = Path(__file__).resolve().parents[1]
profile = root / '.cache/renderer-profile'
(profile / 'config').mkdir(parents=True,exist_ok=True)
(profile/'config/settings.json').write_text('{"auto_update":false,"telemetry":{"metrics":false,"diagnostics":false}}',encoding='utf-8')
device = frida.get_local_device()
pid = device.spawn([r'D:\Zed\Zed.exe','--user-data-dir',str(profile)],stdio='pipe',cwd=r'D:\Zed')
session = None
try:
    session = device.attach(pid)
    script = session.create_script((root/'tools/renderer_probe.js').read_text(encoding='utf-8'))
    script.on('message',lambda message,data: print(json.dumps(message),flush=True) if message['type']=='error' else None)
    script.load()
    catalog=json.loads((root/'assets/catalog.json').read_text(encoding='utf-8'))
    script.exports_sync.start([message['source'] for message in catalog['messages']])
    device.resume(pid)
    print(json.dumps({'pid':pid,'profile':str(profile),'observer_only':True}),flush=True)
    for command in sys.stdin:
        if command.strip()=='stop': break
        result=script.exports_sync.samples()
        (root/'reports').mkdir(exist_ok=True)
        (root/'reports/render-traces.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
        print(json.dumps({'sampled_sources':len(result['samples']),'hooks':result['hooks']}),flush=True)
finally:
    if session:
        session.detach()
    try: device.kill(pid)
    except frida.ProcessNotFoundError: pass
