"""Real installed-Zed integration test. Only owns an isolated spawned PID."""
import argparse
import json
from pathlib import Path
import subprocess
import time

p = argparse.ArgumentParser()
p.add_argument("--binary", type=Path, required=True)
p.add_argument("--exe", type=Path, required=True)
p.add_argument("--python-reference", type=Path)
p.add_argument("--output", type=Path, default=Path("reports/native-integration.json"))
args = p.parse_args()

def run(*arguments):
    result = subprocess.run([str(args.binary.resolve()), *arguments, "--exe", str(args.exe.resolve())],
                            capture_output=True, text=True, encoding="utf-8", timeout=30)
    if result.returncode:
        raise RuntimeError(result.stderr)
    return json.loads(result.stdout)

plan = run("plan")
assert plan["supported"] and plan["candidate_count"] > 0
parity = None
if args.python_reference:
    reference = json.loads(args.python_reference.read_text(encoding="utf-8"))
    fields = ["source", "translation", "rva", "before", "after", "padding_bytes"]
    parity = [{k: e[k] for k in fields} for e in plan["entries"]] == [{k: e[k] for k in fields} for e in reference["entries"]]
    assert parity, "Rust candidates differ from the verified Python prototype"
memory = run("test-memory")
assert memory["count"] == memory["restored"] == plan["candidate_count"]
assert memory["disk_unchanged"] and memory["injected_failure_rolled_back"]
launched = run("launch", "--experimental", "--isolated")
assert launched["mode"] == "experimental-launch" and launched["resident_launcher"] is False
pid = launched["pid"]
try:
    time.sleep(4)
    restored = run("restore", "--pid", str(pid))
    assert restored["verified"] and restored["restored"] == plan["candidate_count"]
    report = {"status": "passed", "candidate_count": plan["candidate_count"], "python_parity": parity,
              "memory": memory, "launcher_exited": True, "restore_after_exit": restored,
              "plan_seconds": plan["tool_elapsed_seconds"], "launch_seconds": launched["tool_elapsed_seconds"],
              "isolated": True, "visual_acceptance": False}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
finally:
    # This is the PID created by this test, after identity-checked restoration.
    subprocess.run(["taskkill", "/PID", str(pid), "/F"], capture_output=True, timeout=10)
