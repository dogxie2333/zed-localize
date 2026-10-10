"""Local UI proof, with public catalogue only and no runtime translation calls."""
import json
from pathlib import Path
import hashlib
import sys
import frida
import pefile

root=Path(__file__).resolve().parents[1]
exe=Path(r'D:\Zed\Zed.exe')
with exe.open('rb') as file:
    assert hashlib.file_digest(file,'sha256').hexdigest()=='d7afb758b045958838d843b95dd28c41554efd810c0f15012cf0069ff8046781'
catalog=json.loads((root/'assets/catalog.json').read_text(encoding='utf-8'))
messages={m['source']:m['translation'] for m in catalog['messages'] if m['source']!=m['translation']
          and not any(c in m['source']+m['translation'] for c in '{}\n\r\t`*_[]<>')
          and len(m['translation'].encode('utf-16-le'))<=len(m['source'].encode('utf-16-le'))}
profile=root/'.cache/ui-overlay-profile'
(profile/'config').mkdir(parents=True,exist_ok=True)
(profile/'config/settings.json').write_text('{"auto_update":false,"telemetry":{"metrics":false,"diagnostics":false},"buffer_font_family":"IBM Plex Sans"}',encoding='utf-8')
device=frida.get_local_device()
pid=device.spawn([str(exe),'--user-data-dir',str(profile)],cwd=str(exe.parent),stdio='pipe')
session=None
try:
    session=device.attach(pid)
    script=session.create_script((root/'tools/ui_overlay.js').read_text(encoding='utf-8'))
    script.on('message',lambda m,d:print(json.dumps(m),flush=True) if m['type']=='error' else None)
    script.load()
    report=script.exports_sync.configure({'messages':messages,'ui_fonts':['IBM Plex Sans'],
        'ui_caller_rva':'0x5c09ef0','image_size':pefile.PE(str(exe),fast_load=True).OPTIONAL_HEADER.SizeOfImage})
    device.resume(pid)
    print(json.dumps({'pid':pid,'dictionary':len(messages),'profile':str(profile),**report}),flush=True)
    for command in sys.stdin:
        if command.strip()=='stop':break
        stats=script.exports_sync.stats()
        (root/'reports/ui-overlay-observation.json').write_text(json.dumps(stats,ensure_ascii=False,indent=2),encoding='utf-8')
        print(json.dumps({'translated_distinct':len(stats['translated_keys']),'denied':stats['denied'],'protected_count':stats['protected_count']}),flush=True)
finally:
    if session:session.detach()
    try:device.kill(pid)
    except frida.ProcessNotFoundError:pass
