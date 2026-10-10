"""Append a ZIP overlay to the native EXE; avoid massive LLVM byte-array IR."""
from pathlib import Path
import hashlib
import json
import shutil
import os
import time
import subprocess

root=Path(__file__).resolve().parents[1]
build=Path(os.environ.get('ZED_ZH_BUILD_DIR',root.parent/'.build-cache/zed-zh-addon/package'))
payload=build/'portable-payload.zip'
target=Path(json.loads(subprocess.check_output(['cargo','metadata','--no-deps','--format-version','1'],cwd=root,text=True,encoding='utf-8'))['target_directory'])
output=root/'dist/Zed-Localize.exe'
output.parent.mkdir(exist_ok=True)
temporary=output.with_name(f'Zed-Localize.{os.getpid()}.building.exe')
shutil.copyfile(target/'release/zed-zh-launcher.exe',temporary)
digest=hashlib.sha256()
with payload.open('rb') as source,temporary.open('ab') as target:
    for block in iter(lambda:source.read(1024*1024),b''):
        digest.update(block);target.write(block)
    target.write(payload.stat().st_size.to_bytes(8,'little'))
    target.write(digest.hexdigest().encode('ascii'))
    target.write(b'ZED_ZH_PAYLOAD_V1')
for attempt in range(120):
    try:
        os.replace(temporary,output)
        break
    except PermissionError:
        if attempt==119:raise
        time.sleep(0.25)
print(json.dumps({'exe':str(output),'mib':round(output.stat().st_size/1048576,2),'payload_sha256':digest.hexdigest()}))
