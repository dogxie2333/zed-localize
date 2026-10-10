"""Offline maintenance orchestrator, using upstream extraction/context tools."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import os
import re
import subprocess
import sys
import tomllib
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "tools"))
from community_bridge import PIN, load_toolkit, write
from repo_bridge import checkout,revision as repo_revision
STATE_ROOT=Path(os.environ['ZED_ZH_STATE_DIR']) if os.environ.get('ZED_ZH_STATE_DIR') else ROOT
CACHE_ROOT=STATE_ROOT/'.cache'

def github(path):
    request = urllib.request.Request("https://api.github.com/" + path, headers={"User-Agent": "zed-zh-addon-maintenance"})
    # GitHub's job token is only an HTTP header; it is never serialized.
    token = os.environ.get("GH_TOKEN") or os.environ.get("GITHUB_TOKEN")
    if token:
        request.add_header("Authorization", "Bearer " + token)
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.load(response)

def run(*args, cwd=None):
    subprocess.run(list(map(str, args)), cwd=cwd, check=True, stdout=subprocess.DEVNULL)

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--check-only", action="store_true")
    parser.add_argument("--force", action="store_true")
    parser.add_argument("--maintainer", type=Path)
    parser.add_argument("--config", type=Path)
    parser.add_argument("--cache", type=Path, default=CACHE_ROOT/'translations.redb')
    parser.add_argument("--toolkit", type=Path, help="Reuse a clean existing community checkout at the detected commit")
    parser.add_argument("--source-root", type=Path, help="Reuse a clean existing official checkout at the detected tag")
    args = parser.parse_args()
    release = github("repos/zed-industries/zed/releases/latest")
    tag = release["tag_name"]
    if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
        raise ValueError("Unexpected official stable release tag")
    community_commit = github("repos/LI-NA/zed-i18n/commits/master")["sha"]
    if not re.fullmatch(r"[0-9a-f]{40}", community_commit):
        raise ValueError("Unexpected community commit")
    state_path = STATE_ROOT/'upstream-state.json' if STATE_ROOT!=ROOT else ROOT/'data/upstream-state.json'
    state = json.loads((state_path if state_path.exists() else ROOT/'data/upstream-state.json').read_text(encoding="utf-8"))
    changed = state.get("latest_tag") != tag or state.get("community_commit") != community_commit
    api_ready = bool(os.environ.get("ZED_I18N_API_KEY") and os.environ.get("ZED_I18N_MODEL"))
    work_needed = changed or (state.get("translation_status") == "pending-api" and api_ready)
    if args.check_only:
        print(json.dumps({"latest_tag": tag, "community_commit": community_commit, "changed": changed,
                          "work_needed": work_needed, "installed_program_changed": False}))
        return
    if not work_needed and not args.force:
        print("No new upstream text; no AI request")
        return
    toolkit = args.toolkit.resolve() if args.toolkit else CACHE_ROOT/'community-tools'/PIN
    if args.toolkit:
        load_toolkit(toolkit)
    else:
        checkout("https://github.com/LI-NA/zed-i18n.git", PIN, toolkit)
    data_root = toolkit if community_commit == PIN else CACHE_ROOT/'community-data'/community_commit
    if data_root != toolkit:
        checkout("https://github.com/LI-NA/zed-i18n.git", community_commit, data_root)
    source = args.source_root.resolve() if args.source_root else CACHE_ROOT/'official'/tag
    if args.source_root:
        source_commit = repo_revision(source)
        if github("repos/zed-industries/zed/commits/" + tag)["sha"] != source_commit:
            raise ValueError("Existing official checkout is not the detected release tag")
    else:
        source_commit = checkout("https://github.com/zed-industries/zed.git", tag, source)
    # This is a real public GitHub commit, not merely a local repository with a
    # claimed public origin. Never use user projects as translation context.
    if github("repos/zed-industries/zed/commits/" + source_commit)["sha"] != source_commit:
        raise ValueError("Source commit is not confirmed public")
    load_toolkit(toolkit)
    project = tomllib.loads((data_root / "config/project.toml").read_text(encoding="utf-8"))
    prepared = CACHE_ROOT/'jobs'/tag
    prepared.mkdir(parents=True, exist_ok=True)
    manifest_arg = []
    if project.get("zed_commit") != source_commit:
        from tools.zed_i18n.extract import extract_repository
        catalog, manifest = extract_repository(source, toolkit / "tools/zed_i18n/runtime_overlay")
        old = json.loads((data_root / "manifest/ui-strings.json").read_text(encoding="utf-8"))
        for key, metadata in manifest.items():
            if key in old:
                metadata["status"] = old[key].get("status", "needs_review")
        write(prepared / "fresh-manifest.json", manifest)
        manifest_arg = ["--manifest", prepared / "fresh-manifest.json"]
    run(sys.executable, ROOT / "tools/community_bridge.py", "prepare", "--toolkit", toolkit,
        "--source-root", source, "--out", prepared, "--data-root", data_root, *manifest_arg)
    jobs = json.loads((prepared / "jobs.json").read_text(encoding="utf-8"))
    # Public extraction/context only. Native end-user settings consume this
    # reviewed job envelope; provider credentials and replies never enter it.
    if STATE_ROOT == ROOT:
        write(ROOT / 'data/translation-jobs.json', jobs)
        write(ROOT / 'data/translation-jobs/zh-CN.json', jobs)
        from export_locales import LANGUAGES
        for locale in LANGUAGES:
            if locale == 'zh-CN': continue
            locale_out = prepared / locale
            run(sys.executable, ROOT / 'tools/community_bridge.py', 'prepare',
                '--toolkit', toolkit, '--source-root', source, '--out', locale_out,
                '--data-root', data_root, '--locale', locale, *manifest_arg)
            write(ROOT / f'data/translation-jobs/{locale}.json', json.loads((locale_out/'jobs.json').read_text(encoding='utf-8')))
    missing = sum(len(batch["entries"]) for batch in jobs["batches"])
    translated = {"translations": {}, "status": "draft", "batch_requests": 0, "local_cache_hits": 0,
                  "review_required": jobs["review_required"], "runtime_compatibility_approved": False}
    status = "community-complete" if not missing else "pending-api"
    if missing and args.maintainer and args.config:
        output = prepared / "translated-draft.json"
        run(args.maintainer.resolve(), "translate", "--jobs", prepared / "jobs.json", "--config", args.config.resolve(),
            "--cache", args.cache.resolve(), "--toolkit", toolkit, "--python", sys.executable,
            "--bridge", ROOT / "tools/community_bridge.py", "--output", output)
        translated = json.loads(output.read_text(encoding="utf-8"))
        status = "draft-ready" if not translated["unresolved_count"] else "needs-review"
    elif missing:
        print("New text prepared; API/model not configured, leaving a draft for review")
    draft = {"latest_tag": tag, "source_commit": source_commit, "community_commit": community_commit,
             "new_text_count": missing, "translation_status": status, **translated}
    # No jobs, raw model responses, API configuration or source paths enter the
    # public draft. Only validated translations and review metadata are exported.
    write((STATE_ROOT/'drafts' if STATE_ROOT!=ROOT else ROOT/'data/drafts') / f"{tag}.json", draft)
    write(state_path, {"latest_tag": tag, "community_commit": community_commit,
                      "translation_status": status, "runtime_compatibility_approved": False})
    print(json.dumps({"latest_tag": tag, "new_text_count": missing, "translation_status": status,
                      "batch_requests": translated["batch_requests"], "local_cache_hits": translated["local_cache_hits"]}))

if __name__ == "__main__":
    main()
