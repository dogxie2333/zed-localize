"""Verify forced launcher termination cannot orphan its suspended Zed child."""
import argparse
import ctypes
from ctypes import wintypes
import json
from pathlib import Path
import subprocess
import tempfile

p = argparse.ArgumentParser()
p.add_argument("--probe", type=Path, required=True)
p.add_argument("--exe", type=Path, required=True)
args = p.parse_args()
kernel = ctypes.WinDLL("kernel32", use_last_error=True)
kernel.OpenProcess.argtypes = [wintypes.DWORD, wintypes.BOOL, wintypes.DWORD]
kernel.OpenProcess.restype = wintypes.HANDLE
kernel.WaitForSingleObject.argtypes = [wintypes.HANDLE, wintypes.DWORD]
kernel.WaitForSingleObject.restype = wintypes.DWORD
kernel.CloseHandle.argtypes = [wintypes.HANDLE]
with tempfile.TemporaryDirectory(prefix="zed-startup-guard-") as profile:
    launcher = subprocess.Popen([str(args.probe.resolve()), str(args.exe.resolve()), profile],
                                stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                text=True, encoding="utf-8")
    handle = None
    child = None
    try:
        line = launcher.stdout.readline().strip()
        if not line.isdecimal():
            raise RuntimeError("Guard fixture failed: " + launcher.stderr.read())
        child = int(line)
        handle = kernel.OpenProcess(0x00100000, False, child)  # SYNCHRONIZE
        assert handle, "Cannot observe the owned child"
        launcher.kill()
        launcher.wait(timeout=10)
        assert kernel.WaitForSingleObject(handle, 10000) == 0, "Paused child survived the launcher"
        result = {"status": "passed", "owned_child": child, "forced_launcher_exit_cleanup": True}
        Path("reports").mkdir(exist_ok=True)
        Path("reports/startup-guard.json").write_text(json.dumps(result, indent=2), encoding="utf-8")
        print(json.dumps(result, indent=2))
    finally:
        if launcher.poll() is None:
            launcher.kill()
            launcher.wait(timeout=10)
        if handle:
            kernel.CloseHandle(handle)
