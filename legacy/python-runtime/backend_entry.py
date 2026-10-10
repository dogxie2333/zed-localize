"""PyInstaller runtime entry: route bundled public scripts, import UI lazily."""
from pathlib import Path
import os
import runpy
import sys

root=Path(os.environ.get('ZED_ZH_RUNTIME_ROOT',str(Path(sys.executable).resolve().parent.parent)))
os.environ['ZED_ZH_RUNTIME_ROOT']=str(root)
sys.path.insert(0,str(root/'runtime'))
sys.path.insert(0,str(root/'tools'))
if len(sys.argv)<2:raise SystemExit('Use the single-file launcher.')
script=Path(sys.argv[1]).resolve()
allowed={'runtime/ai_settings.py','runtime/update_compat.py','runtime/select_folder.py',
    'tools/update.py','tools/community_bridge.py','tools/test_settings_ui.py'}
try:relative=script.relative_to(root.resolve()).as_posix()
except ValueError:raise SystemExit('Unsupported runtime entry')
if relative not in allowed:raise SystemExit('Unsupported runtime entry')
sys.argv=sys.argv[1:]
try:
    runpy.run_path(str(script),run_name='__main__')
except Exception as error:
    import json,time
    state=Path(os.environ.get('ZED_ZH_STATE_DIR',str(root/'local')))
    state.mkdir(parents=True,exist_ok=True)
    (state/'backend-error.json').write_text(json.dumps({'script':relative,'error_type':type(error).__name__,'time':time.time()}),encoding='utf-8')
    raise
