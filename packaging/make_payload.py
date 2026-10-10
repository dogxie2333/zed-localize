"""Pack native components and public resources only; no Python runtime."""
from pathlib import Path
import hashlib,json,os,zipfile,subprocess
root=Path(__file__).resolve().parents[1]
build=Path(os.environ.get('ZED_ZH_BUILD_DIR',root.parent/'.build-cache/zed-zh-addon/package'))
build.mkdir(parents=True,exist_ok=True)
target=Path(json.loads(subprocess.check_output(['cargo','metadata','--no-deps','--format-version','1'],cwd=root,text=True,encoding='utf-8'))['target_directory'])
files={'runtime/install-startup.ps1':root/'runtime/install-startup.ps1','zed-zh-settings.exe':target/'release/zed-zh-settings.exe','assets/catalog.json':root/'assets/catalog.json','assets/LICENSE.md':root/'assets/LICENSE.md'}
for name in ('ui_overlay.js','gadget_lifecycle.js','test_gadget_restart.js'):
    files['tools/'+name]=root/'tools'/name
for source in (root/'assets/locales').glob('*.json'):
    files[source.relative_to(root).as_posix()]=source
for folder in ('data','docs','licenses'):
    for source in (root/folder).rglob('*'):
        if source.is_file() and (folder=='licenses' or source.suffix in ('.json','.md','.txt')):files[source.relative_to(root).as_posix()]=source
for source in (root/'web/dist').rglob('*'):
    if source.is_file():files[source.relative_to(root).as_posix()]=source
for name in ('LICENSE','README.md','README.en.md','README.zh-TW.md','ACKNOWLEDGEMENTS.md','THIRD_PARTY_NOTICES.md'):files[name]=root/name
vendor=Path(os.environ.get('ZED_ZH_VENDOR_DIR',root/'local/vendor'))
for name in ('frida-gadget-17.23.3.dll','frida-gadget-origin.json'):files['vendor/'+name]=vendor/name
origin=json.loads((vendor/'frida-gadget-origin.json').read_text())
assert hashlib.sha256((vendor/'frida-gadget-17.23.3.dll').read_bytes()).hexdigest()==origin['sha256_dll']
payload=build/'portable-payload.zip'
with zipfile.ZipFile(payload,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9,strict_timestamps=False) as archive:
    for name,source in sorted(files.items()):archive.write(source,name)
    archive.writestr('build-version.json',json.dumps({'gadget':origin,'native_source':hashlib.sha256(b''.join(p.read_bytes() for p in sorted((root/'src').rglob('*.rs')))).hexdigest(),'python_runtime':False}))
with zipfile.ZipFile(payload) as archive:uncompressed=sum(p.file_size for p in archive.infolist());count=len(archive.infolist())
print(json.dumps({'compressed_mib':round(payload.stat().st_size/1048576,2),'uncompressed_mib':round(uncompressed/1048576,2),'files':count,'python_runtime':False,'payload':str(payload)}))
