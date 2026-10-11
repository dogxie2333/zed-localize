"""Opt-in real-editor display regression; only isolated processes are stopped."""
import argparse
import json
import os
from pathlib import Path
import signal
import socket
import subprocess
import tempfile
import threading
import time

p = argparse.ArgumentParser()
p.add_argument('--binary', type=Path, required=True)
p.add_argument('--zed', type=Path, required=True)
p.add_argument('--resources', type=Path, required=True)
p.add_argument('--workdir', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
args = p.parse_args()
if os.geteuid() == 0:
    p.error('Run as a normal desktop user, not root')
args.workdir.mkdir(parents=True, exist_ok=True)
results = []
for mode, unavailable, stalled, expected in (
    ('auto', False, False, 'wayland'),
    ('wayland', False, False, 'wayland'),
    ('x11', False, False, 'x11'),
    ('auto', True, False, 'x11'),
    ('auto', False, True, 'x11'),
    ('wayland', True, False, None),
):
    profile = Path(tempfile.mkdtemp(prefix='display-', dir=args.workdir))
    (profile/'config').mkdir()
    (profile/'config/settings.json').write_text('{"auto_update":false,"telemetry":{"diagnostics":false,"metrics":false}}')
    env = os.environ.copy()
    env['ZED_STATELESS'] = '1'
    for key in ('XDG_CONFIG_HOME', 'XDG_DATA_HOME', 'XDG_CACHE_HOME', 'XDG_STATE_HOME'):
        env[key] = str(profile/key)
    stop_server = threading.Event()
    listener = None
    worker = None
    if unavailable:
        env['WAYLAND_DISPLAY'] = str(profile/'not-a-compositor')
    if stalled:
        listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        listener.bind(str(profile/'stalled-compositor'))
        listener.listen(1)
        listener.settimeout(10)
        env['WAYLAND_DISPLAY'] = str(profile/'stalled-compositor')
        def stall():
            try:
                connection, _ = listener.accept()
                with connection:
                    stop_server.wait(15)
            except OSError:
                pass
        worker = threading.Thread(target=stall)
        worker.start()
    command = [str(args.binary), '--resources', str(args.resources), '--zed', str(args.zed), '--display-server', mode, 'launch', '--', '--user-data-dir', str(profile)]
    pid = None
    item = {'mode': mode, 'missing_compositor': unavailable, 'stalled_compositor': stalled, 'expected': expected}
    try:
        start = time.monotonic()
        with (profile/'stdout').open('w') as out, (profile/'stderr').open('w') as err:
            completed = subprocess.run(command, env=env, stdout=out, stderr=err, timeout=10)
        item['launcher_seconds'] = time.monotonic()-start
        item['returncode'] = completed.returncode
        item['stderr'] = (profile/'stderr').read_text(errors='replace')[-3000:]
        if expected is None:
            assert completed.returncode != 0, 'Explicit Wayland unexpectedly fell back'
            assert not (profile/'stdout').read_text().strip(), 'Unexpected editor spawn'
        else:
            assert completed.returncode == 0, item['stderr']
            item['launch'] = json.loads((profile/'stdout').read_text())
            pid = item['launch']['pid']
            assert item['launch']['display_server'] == expected, item
            process = Path('/proc')/str(pid)
            time.sleep(4)
            assert (process/'exe').resolve() == args.zed.resolve(), 'Editor exited or executable changed'
            assert str(profile).encode() in (process/'cmdline').read_bytes().split(b'\0'), 'Editor is not owned by this test'
            item['alive_after_launcher_exit'] = True
            item['graphics_logs'] = [line for log in profile.rglob('Zed.log') for line in log.read_text(errors='replace').splitlines() if 'Using GPU:' in line]
        if unavailable or stalled:
            assert item['launcher_seconds'] < 8, 'Readiness deadline was not bounded'
        item['passed'] = True
    except Exception as error:
        item['passed'] = False
        item['error'] = str(error)
    finally:
        if pid is not None:
            process = Path('/proc')/str(pid)
            try:
                if (process/'exe').resolve() == args.zed.resolve() and str(profile).encode() in (process/'cmdline').read_bytes().split(b'\0'):
                    os.kill(pid, signal.SIGTERM)
            except (OSError, ProcessLookupError):
                pass
        stop_server.set()
        if listener is not None:
            listener.close()
            worker.join(timeout=11)
        results.append(item)
        args.output.write_text(json.dumps(results, indent=2))
        print(json.dumps(item), flush=True)
if not all(item['passed'] for item in results):
    raise SystemExit(1)
