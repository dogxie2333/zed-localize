"""Create a release from an explicit public-file whitelist, never user state."""
from pathlib import Path
import hashlib,json,subprocess,zipfile
r=Path(__file__).resolve().parents[1];dist=r/'dist';dist.mkdir(exist_ok=True)
exe=dist/'Zed-Localize.exe';assert exe.is_file()
files={'Zed-Localize.exe':exe,'README.md':r/'README.md','README.en.md':r/'README.en.md','README.zh-TW.md':r/'README.zh-TW.md','LICENSE':r/'LICENSE','ACKNOWLEDGEMENTS.md':r/'ACKNOWLEDGEMENTS.md','THIRD_PARTY_NOTICES.md':r/'THIRD_PARTY_NOTICES.md'}
for file in (r/'licenses').rglob('*'):
    if file.is_file():files[file.relative_to(r).as_posix()]=file
files['examples/settings.json']=r/'examples/settings.json'
archive=dist/'zed-localize-windows-x64.zip'
with zipfile.ZipFile(archive,'w',compression=zipfile.ZIP_DEFLATED,compresslevel=9,strict_timestamps=False) as z:
    for name,path in sorted(files.items()):z.write(path,name)
    z.writestr('SOURCE.json',json.dumps({'repository':'dogxie2333/zed-localize','commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=r,text=True).strip(),'source_license':'MIT','community_resources':'GPL-3.0','python_runtime':False},indent=2))
def digest(path):
    with path.open('rb') as file:return hashlib.file_digest(file,'sha256').hexdigest()
(dist/'SHA256SUMS.txt').write_text(''.join(f'{digest(p)}  {p.name}\n' for p in (exe,archive)),encoding='utf-8')
print(json.dumps({'exe_mib':round(exe.stat().st_size/1048576,2),'package_mib':round(archive.stat().st_size/1048576,2),'files':len(files),'user_state_packaged':False}))
