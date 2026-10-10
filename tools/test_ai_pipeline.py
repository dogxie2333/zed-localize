"""Local HTTP mock plus the real community validator; never calls a provider."""
import argparse
import json
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
import os
import subprocess
import tempfile
import threading

p = argparse.ArgumentParser()
p.add_argument("--binary", type=Path, required=True)
p.add_argument("--jobs", type=Path, required=True)
p.add_argument("--toolkit", type=Path, required=True)
p.add_argument("--python", type=Path, default=Path("python"))
p.add_argument("--output", type=Path, default=Path("reports/ai-integration.json"))
args = p.parse_args()
jobs = json.loads(args.jobs.read_text(encoding="utf-8"))
assert len(jobs["batches"]) == 1 and len(jobs["batches"][0]["entries"]) == 1
source = jobs["batches"][0]["entries"][0]["source"]
translation = json.loads((args.toolkit / "translations/zh-CN.json").read_text(encoding="utf-8"))[source]
state = {"calls": 0, "requests": [], "answer": {source: translation}}
fake_key = "mock-credential-never-a-real-key"

class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass
    def do_POST(self):
        assert self.path == "/v1/chat/completions"
        payload = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        state["calls"] += 1
        state["requests"].append(payload)
        body = json.dumps({"id": "chatcmpl-mock", "object": "chat.completion", "created": 0,
            "model": "mock-model", "choices": [{"index": 0, "message": {"role": "assistant",
                "content": json.dumps(state["answer"], ensure_ascii=False)}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 2048, "completion_tokens": 30, "total_tokens": 2078,
                      "prompt_tokens_details": {"cached_tokens": 1024}}}, ensure_ascii=False).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

server = HTTPServer(("127.0.0.1", 0), Handler)
threading.Thread(target=server.serve_forever, daemon=True).start()
try:
    with tempfile.TemporaryDirectory(prefix="zed-zh-ai-test-") as directory:
        root = Path(directory)
        config = root / "config.toml"
        config.write_text(f'api_base="http://127.0.0.1:{server.server_port}/v1"\nmodel="mock-model"\nallow_local_http=true\nprovider_prompt_cache_key=true\ntimeout_seconds=10\n', encoding="utf-8")
        job_file = root / "jobs.json"
        job_file.write_text(json.dumps(jobs, ensure_ascii=False), encoding="utf-8")
        cache = root / "cache.redb"
        report_file = root / "report.json"
        def run(key=True, success=True):
            environment = dict(os.environ)
            environment.pop("ZED_I18N_API_KEY", None)
            if key:
                environment["ZED_I18N_API_KEY"] = fake_key
            process = subprocess.run([str(args.binary.resolve()), "translate", "--jobs", str(job_file), "--config", str(config),
                "--cache", str(cache), "--toolkit", str(args.toolkit.resolve()), "--python", str(args.python),
                "--bridge", str(Path("tools/community_bridge.py").resolve()), "--output", str(report_file)],
                env=environment, capture_output=True, text=True, encoding="utf-8", timeout=30)
            assert fake_key not in process.stdout + process.stderr
            if success:
                assert process.returncode == 0, process.stderr
                return json.loads(report_file.read_text(encoding="utf-8"))
            assert process.returncode != 0, "Invalid response was accepted"
            return None

        first = run()
        assert first["batch_requests"] == 1 and first["local_cache_hits"] == 0
        assert first["provider_cached_tokens"] == 1024
        payload = state["requests"][0]
        assert "GPUI" in payload["messages"][0]["content"]
        assert "MenuItem::action" in payload["messages"][1]["content"]
        assert payload["prompt_cache_key"].startswith("zed-zh:")
        second = run(key=False)
        assert second["batch_requests"] == 0 and second["local_cache_hits"] == 1
        assert state["calls"] == 1
        jobs["source_commit"] = "fixture-version-shift-only"
        entry = jobs["batches"][0]["entries"][0]
        entry["occurrences"][0]["line"] += 100
        entry["occurrences"][0]["start_byte"] += 100
        entry["code_context"] = "\n".join(str(int(line.split(':', 1)[0]) + 100) + ':' + line.split(':', 1)[1] for line in entry["code_context"].splitlines())
        job_file.write_text(json.dumps(jobs, ensure_ascii=False), encoding="utf-8")
        shifted = run(key=False)
        assert shifted["local_cache_hits"] == 1 and state["calls"] == 1
        entry["code_context"] += '\n MenuItem::action("Open Project Settings File", DifferentContext)'
        job_file.write_text(json.dumps(jobs, ensure_ascii=False), encoding="utf-8")
        changed = run()
        assert changed["batch_requests"] == 1 and state["calls"] == 2
        assert payload["messages"][0]["content"] == state["requests"][1]["messages"][0]["content"]

        # The actual upstream validator must reject a missing placeholder.
        jobs["batches"][0]["entries"] = [{"source": "Failed to save {path}", "kind": "error_prompt",
            "call": "fixture", "code_context": "Label::new(error_message)", "occurrences": []}]
        job_file.write_text(json.dumps(jobs), encoding="utf-8")
        state["answer"] = {"Failed to save {path}": "保存失败"}
        run(success=False)
        state["answer"] = {"Failed to save {path}": "保存 {path} 失败"}
        recovered = run()
        assert recovered["batch_requests"] == 1 and recovered["local_cache_hits"] == 0
        jobs["batches"][0]["entries"][0]["code_context"] += " new-context"
        job_file.write_text(json.dumps(jobs), encoding="utf-8")
        state["answer"] = {"Failed to save {path}": "保存 {path} 失败 " + fake_key}
        run(success=False)
        assert fake_key.encode() not in cache.read_bytes()
        report = {"status": "passed", "external_provider_called": False, "community_validator_used": True,
            "initial_batch_requests": 1, "repeated_requests_without_key": 0,
            "line_and_version_change_cache_hit": True, "semantic_context_change_invalidated": True,
            "stable_system_prefix": True, "placeholder_rejection_and_recovery": True,
            "provider_secret_echo_rejected": True, "runtime_compatibility_auto_approved": False}
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(report, indent=2), encoding="utf-8")
        print(json.dumps(report, indent=2))
finally:
    server.shutdown()
    server.server_close()
