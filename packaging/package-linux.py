"""Build Ubuntu package and portable archive from a public-resource whitelist."""
import argparse,json,shutil,subprocess,tarfile,hashlib,re
from pathlib import Path

p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);p.add_argument('--rust-licenses',type=Path,required=True);p.add_argument('--version',default='0.3.1-alpha.2');args=p.parse_args()
if not re.fullmatch(r'\d+\.\d+\.\d+(?:-[a-z]+\.\d+)?',args.version):p.error('Invalid release version')
deb_version=args.version.replace('-','~')
root=Path(__file__).resolve().parents[1];dist=root/'dist';dist.mkdir(exist_ok=True)
stage=Path('/tmp/zed-localize-build')/('package-'+args.version);stage.mkdir(parents=True,exist_ok=True)
payload=stage/'opt/zed-localize';payload.mkdir(parents=True,exist_ok=True)
shutil.copy2(args.binary,payload/'zed-localize-linux');(payload/'zed-localize-linux').chmod(0o755)
for name in ('README.md','README.en.md','README.zh-TW.md','LICENSE','ACKNOWLEDGEMENTS.md','THIRD_PARTY_NOTICES.md'):
    shutil.copy2(root/name,payload/name)
for folder in ('assets/locales','docs'):
    shutil.copytree(root/folder,payload/folder,dirs_exist_ok=True)
(payload/'licenses').mkdir(exist_ok=True)
for name in ('GPL-3.0.txt','zed-i18n-MIT.txt'):shutil.copy2(root/'licenses'/name,payload/'licenses'/name)
shutil.copytree(args.rust_licenses,payload/'licenses/rust-dependencies',dirs_exist_ok=True)
(payload/'assets').mkdir(exist_ok=True);shutil.copy2(root/'assets/LICENSE.md',payload/'assets/LICENSE.md')
(payload/'data').mkdir(exist_ok=True);shutil.copy2(root/'data/locales.json',payload/'data/locales.json')
metadata={'repository':'dogxie2333/zed-localize','commit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),'release':args.version,'platform':'Ubuntu 24.04 x86_64','backend':'experimental-ptrace-equal-byte-ui','source_license':'MIT','community_resources':'GPL-3.0'}
(payload/'SOURCE.json').write_text(json.dumps(metadata,indent=2)+'\n')
control=stage/'DEBIAN';control.mkdir(exist_ok=True)
(control/'control').write_text('Package: zed-localize\nVersion: '+deb_version+'\nArchitecture: amd64\nMaintainer: Zed Localize contributors <noreply@github.com>\nDepends: libc6 (>= 2.39)\nRecommends: fonts-noto-cjk\nSection: editors\nPriority: optional\nDescription: Experimental independent localization companion for official Zed\n')
desktop=stage/'usr/share/applications';desktop.mkdir(parents=True,exist_ok=True)
(desktop/'zed-localize.desktop').write_text('[Desktop Entry]\nType=Application\nName=Zed Localize\nComment=Experimental localization launcher for official Zed\nExec=/opt/zed-localize/zed-localize-linux launch\nIcon=zed-localize\nTerminal=false\nCategories=Development;TextEditor;\n')
icons=stage/'usr/share/icons/hicolor/scalable/apps';icons.mkdir(parents=True,exist_ok=True)
shutil.copy2(root/'web/public/mark.svg',icons/'zed-localize.svg')
deb=dist/('zed-localize_'+args.version+'_amd64.deb')
subprocess.run(['dpkg-deb','--root-owner-group','--build',str(stage),str(deb)],check=True)
archive=dist/'zed-localize-ubuntu24-x86_64.tar.gz'
with tarfile.open(archive,'w:gz') as tar:tar.add(payload,arcname='zed-localize')
checks=dist/'SHA256SUMS-linux.txt';checks.write_text(''.join(hashlib.file_digest(file.open('rb'),'sha256').hexdigest()+'  '+file.name+'\n' for file in (deb,archive)))
print(json.dumps({'deb':str(deb),'portable':str(archive),'resources_only':True,'python_runtime':False}))
