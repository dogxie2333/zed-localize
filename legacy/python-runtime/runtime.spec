from pathlib import Path
import os
os.environ['GRADIO_ANALYTICS_ENABLED']='False'
os.environ['LITELLM_LOCAL_MODEL_COST_MAP']='True'
from PyInstaller.utils.hooks import collect_data_files,copy_metadata,collect_submodules

project=Path(SPECPATH).parent
datas=[]
for package in ('gradio','gradio_client','litellm','tiktoken','tree_sitter_rust','safehttpx','groovy'):
    datas+=collect_data_files(package,include_py_files=package=='gradio')
for package in ('gradio','gradio_client','litellm','openai','keyring','filelock','huggingface-hub'):
    datas+=copy_metadata(package,recursive=True)
hidden=['pefile','filelock','json5','gradio','litellm','openai','keyring',
    'unittest.mock','http.server',
    'dulwich','dulwich.client','dulwich.porcelain','dulwich.repo','dulwich.objects',
    'keyring.backends.Windows','win32ctypes.pywin32.win32cred','win32ctypes.pywin32.win32api',
    'tkinter','tkinter.filedialog','tree_sitter','tree_sitter_rust']
hidden+=collect_submodules('litellm',filter=lambda name:'.proxy' not in name and '.integrations' not in name)
analysis=Analysis([str(project/'packaging/backend_entry.py')],pathex=[],binaries=[],datas=datas,
    hiddenimports=hidden,hookspath=[],hooksconfig={},runtime_hooks=[],excludes=['frida','litellm.rust_bridge._native','matplotlib','torch','tensorflow','IPython','pytest'],noarchive=False)
archive=PYZ(analysis.pure)
executable=EXE(archive,analysis.scripts,[],exclude_binaries=True,name='zed-zh-runtime',debug=False,
    bootloader_ignore_signals=False,strip=False,upx=False,console=True,icon=str(project/'assets/icons/app.ico'))
collection=COLLECT(executable,analysis.binaries,analysis.datas,strip=False,upx=False,name='zed-zh-runtime')
