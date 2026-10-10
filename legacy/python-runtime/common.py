"""Local-only configuration. Never serialize credentials or user document text."""
from pathlib import Path
import hashlib
import json
import os
import tempfile

ROOT = Path(os.environ.get('ZED_ZH_RUNTIME_ROOT',str(Path(__file__).resolve().parents[1])))
LOCAL = Path(os.environ.get('ZED_ZH_STATE_DIR',str(ROOT/'local')))
CONFIG = LOCAL / 'settings.json'

def backend_command(script,*arguments):
    backend=ROOT/'backend/zed-zh-runtime.exe'
    interpreter=backend if backend.is_file() else ROOT/'.venv/Scripts/python.exe'
    return [str(interpreter),str(script),*map(str,arguments)]

def launcher():
    return Path(os.environ.get('ZED_ZH_LAUNCHER',str(LOCAL/'bin/zed-zh-launcher.exe')))

def write_json(path, value):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.NamedTemporaryFile(mode='w', encoding='utf-8', dir=path.parent, delete=False) as file:
        json.dump(value, file, ensure_ascii=False, indent=2)
        temporary = file.name
    os.replace(temporary, path)

def settings():
    defaults = {'zed_exe': r'D:\Zed\Zed.exe', 'watch_enabled': True, 'startup_mode': 'manual', 'replace_shortcuts': True, 'provider': 'deepseek',
                'api_base': 'https://api.deepseek.com/v1', 'model': 'deepseek-flash', 'auto_translate': False}
    if CONFIG.exists():
        saved=json.loads(CONFIG.read_text(encoding='utf-8-sig'))
        defaults.update(saved)
        if 'startup_mode' not in saved and os.name=='nt':
            import winreg
            try:
                with winreg.OpenKey(winreg.HKEY_CURRENT_USER,r'Software\Microsoft\Windows\CurrentVersion\Run') as key:
                    winreg.QueryValueEx(key,'ZedZhShortcutGuard')
                defaults['startup_mode']='auto'
            except OSError:pass
    return defaults

def credential_id(provider, endpoint):
    return provider + ':' + hashlib.sha256(endpoint.encode()).hexdigest()[:24]

def overlay_config(exe, profile=None):
    import pefile
    import json5
    with Path(exe).open('rb') as file:
        digest = hashlib.file_digest(file, 'sha256').hexdigest()
    candidates = json.loads((ROOT / 'data/ui-adapters.json').read_text(encoding='utf-8'))['adapters']
    cache = LOCAL / 'verified-adapters.json'
    if cache.exists():
        candidates += json.loads(cache.read_text(encoding='utf-8'))['adapters']
    adapter = next((item for item in candidates if item.get('verified') and item['sha256'] == digest), None)
    if adapter is None:
        return None, digest
    catalog = json.loads((ROOT / 'assets/catalog.json').read_text(encoding='utf-8'))
    messages = {item['source']: item['translation'] for item in catalog['messages']
                if item['source'] != item['translation']
                and not any(c in item['source'] + item['translation'] for c in '{}\n\r\t`*_[]<>')
                and len(item['translation'].encode('utf-16-le')) <= len(item['source'].encode('utf-16-le'))}
    fonts = list(adapter['default_ui_fonts'])
    config_file = Path(profile) / 'config/settings.json' if profile else Path(os.environ.get('APPDATA', '')) / 'Zed/settings.json'
    if config_file.is_file():
        try:
            custom = json5.loads(config_file.read_text(encoding='utf-8')).get('ui_font_family')
            if isinstance(custom, str):
                fonts.append(custom)
        except Exception:
            pass
    image = pefile.PE(str(exe), fast_load=True)
    result = {'messages': messages, 'ui_fonts': fonts, 'ui_caller_rva': adapter['ui_caller_rva'],
              'image_size': image.OPTIONAL_HEADER.SizeOfImage, 'protected_names': []}
    image.close()
    return result, digest
