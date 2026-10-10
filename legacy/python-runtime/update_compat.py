"""Background maintenance: fetch reviewed adapter DATA, never arbitrary code."""
import json
from pathlib import Path
import time
import urllib.request
from common import ROOT, LOCAL, CONFIG, settings, write_json,backend_command

def automatic_translation():
    current = settings()
    if not current.get('auto_translate'):
        return
    # Heavy AI dependencies are imported only after explicit opt-in. No GUI.
    import keyring
    import os
    import subprocess
    from common import credential_id
    from filelock import FileLock, Timeout
    key = keyring.get_password('zed-zh-addon-ai', credential_id(current['provider'], current['api_base']))
    if not key:
        write_json(LOCAL/'ai-maintenance-status.json', {'status':'no-saved-credential','time':time.time()})
        return
    tool=LOCAL/'bin/zed-zh-maintain.exe'
    if not tool.is_file():tool=ROOT/'zed-zh-maintain.exe'
    if not tool.is_file(): return
    config=LOCAL/'ai-automatic.toml'
    content=f'api_base={json.dumps(current["api_base"])}\nmodel={json.dumps(current["model"])}\nuse_legacy_max_tokens=true\n'
    if current['provider'] in ('deepseek','glm','zai'):
        content+='[extra_body.thinking]\ntype="disabled"\n'
    try:
        with FileLock(LOCAL/'ai-maintenance.lock', timeout=0):
            config.write_text(content,encoding='utf-8')
            environment=dict(os.environ)
            environment.update(ZED_I18N_API_KEY=key,ZED_I18N_MODEL=current['model'])
            try:
                result=subprocess.run(backend_command(ROOT/'tools/update.py',
                    '--maintainer',str(tool),'--config',str(config)),cwd=ROOT,env=environment,
                    stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=600)
                write_json(LOCAL/'ai-maintenance-status.json',{'status':'completed' if result.returncode==0 else 'failed','time':time.time()})
            finally:
                environment.pop('ZED_I18N_API_KEY',None)
    except Timeout:
        return
    except Exception as error:
        write_json(LOCAL/'ai-maintenance-status.json',{'status':'failed','error_type':type(error).__name__,'time':time.time()})

def main():
    stamp=LOCAL/'compat-last-check.json'
    if stamp.exists() and time.time()-json.loads(stamp.read_text(encoding='utf-8'))['time']<3600:
        return
    url='https://raw.githubusercontent.com/dogxie2333/zed-zh-addon/main/data/ui-adapters.json'
    try:
        request=urllib.request.Request(url,headers={'User-Agent':'zed-zh-addon-local-maintenance'})
        with urllib.request.urlopen(request,timeout=20) as response:
            data=json.load(response)
        if data.get('schema')!=1: return
        for adapter in data['adapters']:
            if len(adapter['sha256'])!=64 or not all(c in '0123456789abcdef' for c in adapter['sha256']): return
            if int(adapter['ui_caller_rva'],16)<=0 or not isinstance(adapter.get('verified'),bool): return
        write_json(LOCAL/'verified-adapters.json',data)
        write_json(stamp,{'time':time.time(),'status':'checked','adapter_count':len(data['adapters'])})
    except Exception as error:
        write_json(stamp,{'time':time.time(),'status':'check-failed','error_type':type(error).__name__})
    automatic_translation()

if __name__=='__main__':
    import sys
    if '--auto-only' in sys.argv:automatic_translation()
    else:main()
