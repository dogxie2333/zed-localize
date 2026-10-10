"""Real Windows restart test, kills only a dedicated empty test profile."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import psutil

ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT/'runtime'))
from common import overlay_config
from supervisor import Supervisor
import queue
profile=ROOT/'local/automated-restart-test'
(profile/'config').mkdir(parents=True,exist_ok=True)
(profile/'config/settings.json').write_text('{"auto_update":false,"telemetry":{"metrics":false,"diagnostics":false}}')
started=[]
controller=Supervisor()
try:
    with tempfile.TemporaryDirectory() as folder:
        fake=Path(folder)/'unknown.exe'
        fake.write_bytes(b'not-an-approved-zed-binary')
        assert overlay_config(fake)[0] is None
    for iteration in range(2):
        controller.launch({'profile':str(profile),'arguments':[]})
        target=None
        deadline=time.monotonic()+35
        while time.monotonic()<deadline:
            try:
                event,value=controller.work.get_nowait()
                if event=='child':controller.safe_attach(value.pid,True)
                if event=='detached':controller.sessions.pop(value,None)
            except queue.Empty:pass
            for pid, (session, script, mode) in list(controller.sessions.items()):
                try:
                    proc=psutil.Process(pid)
                    command=proc.cmdline()
                    stats=script.exports_sync.stats() if script else {}
                    if str(profile) in command and mode=='chinese' and stats.get('translated_keys'):
                        target=proc;summary={'translated_distinct':len(stats['translated_keys'])};break
                except psutil.Error:pass
            if target:break
            time.sleep(0.5)
        assert target is not None,'Our isolated Zed never rendered Chinese'
        started.append({'pid':target.pid,'translated_distinct':summary['translated_distinct']})
        assert str(profile) in target.cmdline()
        target.terminate()
        target.wait(10)
        time.sleep(1)
    report={'passed':True,'isolated_launches':started,'unknown_hash_falls_back':True,'existing_sessions_terminated':False}
    (ROOT/'reports').mkdir(exist_ok=True)
    (ROOT/'reports/runtime-restart.json').write_text(json.dumps(report,indent=2))
    print(json.dumps(report))
finally:
    controller.pool.shutdown(wait=True)
    for session,_,_ in list(controller.sessions.values()):
        try:session.detach()
        except Exception:pass
    for proc in psutil.process_iter(['name','cmdline']):
        try:
            if proc.info['name']=='Zed.exe' and str(profile) in (proc.info['cmdline'] or []):proc.terminate()
        except psutil.Error:pass
