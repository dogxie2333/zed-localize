"""Kill a test worker while its own cmd.exe is suspended; Job cleans it."""
from pathlib import Path
import subprocess
import sys
import psutil

ROOT=Path(__file__).resolve().parents[1]
if '--worker' in sys.argv:
    sys.path.insert(0,str(ROOT/'runtime'))
    from windows_job import PendingLaunchGuard
    child=subprocess.Popen(['cmd.exe','/c','exit'],creationflags=0x00000004,
        stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    guard=PendingLaunchGuard(child.pid)
    print(child.pid,flush=True)
    sys.stdin.readline()
    guard.close()
else:
    worker=subprocess.Popen([sys.executable,__file__,'--worker'],stdin=subprocess.PIPE,stdout=subprocess.PIPE,text=True)
    child_pid=None
    try:
        child_pid=int(worker.stdout.readline())
        child=psutil.Process(child_pid)
        worker.kill();worker.wait(10)
        child.wait(10)
        print('{"passed":true,"own_suspended_child_cleaned_by_kernel":true}')
    finally:
        if worker.poll() is None:worker.kill()
        if child_pid:
            try:psutil.Process(child_pid).kill()
            except psutil.NoSuchProcess:pass
