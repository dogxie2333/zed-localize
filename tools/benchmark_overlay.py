"""Compare real DirectWrite + Frida paths in an owned isolated Zed copy.

This measures a fixed public-label microbenchmark, not typing/scrolling frames.
"""
import json,os,subprocess,time,shutil,hashlib
from pathlib import Path
import frida,pefile

root=Path(__file__).resolve().parents[1]
folder=root.parent/'.build-cache/zed-zh-addon/overlay-benchmark'
folder.mkdir(parents=True,exist_ok=True)
exe=folder/'Zed.exe'
original=Path('D:/Zed/Zed.exe')
digest=hashlib.file_digest(original.open('rb'),'sha256').hexdigest()
adapter=next(item for item in json.loads((root/'data/ui-adapters.json').read_text(encoding='utf-8'))['adapters'] if item['sha256']==digest and item['verified'])
if not exe.exists() or hashlib.file_digest(exe.open('rb'),'sha256').hexdigest()!=digest:shutil.copy2(original,exe)
profile=folder/'profile';(profile/'config').mkdir(parents=True,exist_ok=True)
(profile/'config/settings.json').write_text('{"auto_update":false,"telemetry":{"metrics":false,"diagnostics":false}}',encoding='utf-8')
config={'messages':json.loads((root/'assets/locales/zh-CN.json').read_text(encoding='utf-8'))['messages'],'locale':'zh-CN','catalogs':{},'public_sources':json.loads((root/'assets/public-ui-keys.json').read_text(encoding='utf-8')),'ui_fonts':adapter['default_ui_fonts'],'ui_caller_rva':adapter['ui_caller_rva'],'image_size':pefile.PE(str(exe),fast_load=True).OPTIONAL_HEADER.SizeOfImage,'attach_existing':True,'protected_names':[]}
device=frida.get_local_device()
reports={}
for name,source in [('baseline',root/'reports/ui-overlay-baseline.js'),('optimized',root/'tools/ui_overlay.js')]:
    process=subprocess.Popen([str(exe),'--user-data-dir',str(profile)],cwd=original.parent,env={**os.environ,'ZED_STATELESS':'1'},stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    session=None;script=None
    try:
        print(json.dumps({'case':name,'phase':'starting'}),flush=True)
        time.sleep(2)
        for attempt in range(40):
            try:session=device.attach(process.pid);break
            except frida.ProcessNotFoundError:
                if process.poll() is not None:raise RuntimeError('Isolated Zed exited')
                time.sleep(.1)
        body=source.read_text(encoding='utf-8')
        assert body.count('Thread.backtrace(this.context,Backtracer.ACCURATE)')==1
        body=body.replace('Thread.backtrace(this.context,Backtracer.ACCURATE)','benchmarkBacktrace(this.context,Backtracer.ACCURATE,this.threadId)')
        script=session.create_script(body+'\n'+(root/'tools/benchmark_overlay.js').read_text(encoding='utf-8'))
        script.on('message',lambda message,data: print(json.dumps(message,ensure_ascii=True),flush=True))
        script.load();script.exports_sync.configure(config)
        print(json.dumps({'case':name,'phase':'configured'}),flush=True)
        for attempt in range(40):
            try:reports[name]=script.exports_sync.benchmark();break
            except frida.RPCException as error:
                if 'dwrite.dll' not in str(error):raise
                time.sleep(.1)
        else:raise RuntimeError('DirectWrite did not load')
        script.exports_sync.disable()
    finally:
        if script:
            try:script.unload()
            except frida.InvalidOperationError:pass
        if session:
            try:session.detach()
            except frida.InvalidOperationError:pass
        if process.poll() is None:process.terminate();process.wait(timeout=10)
reports['median_improvement_percent']=100*(1-reports['optimized']['median_ms']/reports['baseline']['median_ms'])
(root/'reports/overlay-benchmark.json').write_text(json.dumps(reports,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(reports,ensure_ascii=True))
