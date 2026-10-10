"""Event-driven Frida controller. No AI calls; child gating preserves native restart."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import json
from pathlib import Path
import os
import queue
import subprocess
import threading
import time
import uuid

import frida
import psutil
from filelock import FileLock, Timeout
from common import ROOT, LOCAL, CONFIG, overlay_config, settings, write_json

def submit(request):
    folder = LOCAL / 'requests'
    folder.mkdir(parents=True, exist_ok=True)
    write_json(folder / (uuid.uuid4().hex + '.json'), request)

class Supervisor:
    def __init__(self):
        self.device = frida.get_local_device()
        self.sessions = {}
        self.work = queue.Queue()
        self.pool = ThreadPoolExecutor(max_workers=2)
        self.guard = threading.Lock()
        self.config_cache = {}
        self.started = time.time()
        self.device.on('child-added', self.child_added)

    def log(self, event, **data):
        # Only engine status/PID/hash, never argv, document names or API data.
        LOCAL.mkdir(exist_ok=True)
        with (LOCAL / 'runtime-events.jsonl').open('a', encoding='utf-8') as file:
            file.write(json.dumps({'time': time.time(), 'event': event, **data}) + '\n')

    def child_added(self, child):
        self.work.put(('child', child))

    def attach(self, pid, gated=False):
        exe = Path(settings()['zed_exe']).resolve()
        process = psutil.Process(pid)
        actual = Path(process.exe()).resolve()
        is_helper = actual.name.lower() == 'auto_update_helper.exe' and actual.is_relative_to(exe.parent)
        if actual != exe and not is_helper:
            if gated:
                self.device.resume(pid)
            return
        if pid in self.sessions:
            if gated:
                self.device.resume(pid)
            return
        session = None
        try:
            session = self.device.attach(pid)
            session.on('detached', lambda *args: self.work.put(('detached', pid)))
            # Frida supplies child-process notifications and pause/resume.
            # Only this Zed family is gated; unrelated children resume at once.
            try:
                session.enable_child_gating()
                child_gating = True
            except Exception:
                child_gating = False
            script = None
            status = 'update-helper'
            if actual == exe:
                command = process.cmdline()
                profile = command[command.index('--user-data-dir') + 1] if '--user-data-dir' in command else None
                stat = exe.stat()
                cache_key = (str(exe), stat.st_size, stat.st_mtime_ns, profile)
                if cache_key not in self.config_cache:
                    self.config_cache[cache_key] = overlay_config(exe, profile)
                config, digest = self.config_cache[cache_key]
                if config is not None:
                    config = dict(config)
                    config['attach_existing'] = not gated
                    config['protected_names'] = [Path(value).name for value in command[1:] if Path(value).is_file()]
                    script = session.create_script((ROOT / 'tools/ui_overlay.js').read_text(encoding='utf-8'))
                    script.on('message', lambda message, data: self.log('script-error', pid=pid) if message['type'] == 'error' else None)
                    script.load()
                    configured = script.exports_sync.configure(config)
                    status = 'chinese'
                    self.log('configured', pid=pid, sha256=digest, dictionary_count=configured['messages'], child_gating=child_gating)
                else:
                    status = 'unsupported-english'
                    self.log('unsupported-version', pid=pid, sha256=digest)
            with self.guard:
                self.sessions[pid] = (session, script, status)
            if gated:
                self.device.resume(pid)
            self.started = time.time()
        except Exception as error:
            self.log('attach-failed', pid=pid, error_type=type(error).__name__)
            if session:
                try: session.detach()
                except Exception: pass
            if gated:
                try: self.device.resume(pid)
                except Exception: pass

    def safe_attach(self, pid, gated=False):
        try:
            self.attach(pid, gated)
        except Exception as error:
            self.log('child-dispatch-failed', pid=pid, error_type=type(error).__name__)
        finally:
            if gated:
                try: self.device.resume(pid)
                except Exception: pass

    def launch(self, request):
        current = settings()
        exe = Path(current['zed_exe'])
        arguments = list(request.get('arguments', []))
        if request.get('profile'):
            arguments = ['--user-data-dir', request['profile']] + arguments
        if request.get('english'):
            current['watch_enabled'] = False
            write_json(CONFIG, current)
            child = subprocess.Popen([str(exe), *arguments], cwd=exe.parent, stdin=subprocess.DEVNULL,
                                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            self.log('english-launch', pid=child.pid)
            return
        current['watch_enabled'] = True
        write_json(CONFIG, current)
        environment = dict(os.environ)
        environment.pop('ZED_I18N_API_KEY',None)
        environment.pop('ZED_STATELESS',None)
        if request.get('profile'): environment['ZED_STATELESS']='1'
        pid = self.device.spawn([str(exe), *arguments], cwd=str(exe.parent), stdio='pipe', envp=environment)
        from windows_job import PendingLaunchGuard
        pending=None
        try:
            pending=PendingLaunchGuard(pid)
            self.safe_attach(pid, gated=True)
            pending.release()
        except Exception:
            # Only the PID just created by this call may be killed on failure.
            try:self.device.kill(pid)
            except Exception:pass
            raise
        finally:
            if pending:pending.close()

    def status(self):
        result = []
        with self.guard:
            items = list(self.sessions.items())
        for pid, (session, script, mode) in items:
            summary = {'pid': pid, 'mode': mode}
            if script:
                try:
                    stats = script.exports_sync.stats()
                    summary.update(translated_distinct=len(stats['translated_keys']), denied=stats['denied'], protected_count=stats['protected_count'])
                except Exception:
                    pass
            result.append(summary)
        write_json(LOCAL / 'runtime-status.json', {'controller_pid': os.getpid(), 'sessions': result, 'updated': time.time()})

    def run(self):
        next_status = 0
        while True:
            for path in sorted((LOCAL / 'requests').glob('*.json')):
                try:
                    request = json.loads(path.read_text(encoding='utf-8'))
                    path.unlink()
                    if request.get('control')=='stop':
                        self.shutdown()
                        return
                    elif request.get('attach_pid'):
                        self.safe_attach(int(request['attach_pid']))
                    else:
                        self.launch(request)
                except Exception as error:
                    self.log('request-failed', error_type=type(error).__name__)
            try:
                event, value = self.work.get(timeout=0.5)
                if event == 'child':
                    # Immediate resume for unrelated subprocesses. Attaching to
                    # Zed/update helper happens outside Frida's callback thread.
                    self.pool.submit(self.safe_attach, value.pid, True)
                elif event == 'detached':
                    with self.guard: self.sessions.pop(value, None)
                    self.started = time.time()
            except queue.Empty:
                pass
            if time.time() >= next_status:
                self.status()
                next_status = time.time() + 5
            if not self.sessions and time.time() - self.started > 10 and not list((LOCAL / 'requests').glob('*.json')):
                break
        self.pool.shutdown(wait=True)

    def shutdown(self):
        # Release gates and detach scripts; never terminate the editor.
        self.pool.shutdown(wait=True)
        with self.guard:
            items=list(self.sessions.values())
        for session,_,_ in items:
            try:session.disable_child_gating()
            except Exception:pass
            try:session.detach()
            except Exception:pass
        while True:
            try:event,value=self.work.get_nowait()
            except queue.Empty:break
            if event=='child':
                try:self.device.resume(value.pid)
                except Exception:pass
        self.sessions.clear()
        self.status()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--profile')
    parser.add_argument('--attach', type=int)
    parser.add_argument('--english', action='store_true')
    args,arguments = parser.parse_known_args()
    request = {'profile': args.profile, 'english': args.english, 'arguments': arguments}
    if args.attach:
        request = {'attach_pid': args.attach}
    submit(request)
    LOCAL.mkdir(exist_ok=True)
    try:
        with FileLock(LOCAL / 'supervisor.lock', timeout=2):
            Supervisor().run()
    except Timeout:
        return  # Existing controller consumes this launch request.

if __name__ == '__main__':
    main()
