"""Package unchanged license texts discovered via Cargo metadata."""
import argparse
import json
from pathlib import Path
import shutil
import subprocess

p = argparse.ArgumentParser()
p.add_argument("--output", type=Path, required=True)
p.add_argument("--toolchain")
p.add_argument("--target", help="Collect only the selected target's default-feature dependencies")
args = p.parse_args()
command = ["cargo"] + (["+" + args.toolchain] if args.toolchain else [])
scope = ["--filter-platform", args.target] if args.target else ["--all-features"]
metadata = json.loads(subprocess.check_output(command + ["metadata", "--locked", "--format-version", "1"] + scope, text=True, encoding="utf-8"))
args.output.mkdir(parents=True, exist_ok=True)
notices = []
for package in metadata["packages"]:
    if package["source"] is None:
        continue
    source = Path(package["manifest_path"]).parent
    destination = args.output / (package["name"] + "-" + package["version"])
    files = sorted({path for pattern in ["LICENSE*", "COPYING*", "NOTICE*"] for path in source.glob(pattern) if path.is_file()})
    if files:
        destination.mkdir(parents=True, exist_ok=True)
        for path in files:
            shutil.copy2(path, destination / path.name)
    notices.append({"name": package["name"], "version": package["version"], "license": package["license"],
                    "repository": package["repository"], "files": [path.name for path in files]})
(args.output / "DEPENDENCIES.json").write_text(json.dumps(notices, indent=2), encoding="utf-8")
print("Collected dependency notices:", len(notices))
