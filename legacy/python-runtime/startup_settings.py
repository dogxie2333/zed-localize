"""Installation UI glue; no reading login database or altering editor settings."""
import json
import os
from pathlib import Path
import subprocess
import sys
import time
from common import ROOT,LOCAL,CONFIG,settings,write_json,backend_command,launcher

MODES={'开机自启（登录后显示托盘，不自动打开 Zed）':'auto','手动启动（本次运行有效，下次登录不自启）':'manual'}

def close_settings():
    write_json(LOCAL/'settings-stop.json',{'time':time.time()})
    return '配置服务即将关闭；托盘和 Zed 汉化继续运行。再次打开可从托盘菜单进入。'

def resolve_zed(folder):
    path=Path(folder.strip().strip('"')).expanduser()
    if not folder.strip(): raise ValueError('请先选择安装文件夹')
    exe=path if path.is_file() else path/'Zed.exe'
    if exe.parent.name.lower()=='bin' and (exe.parent.parent/'Zed.exe').is_file():
        exe=exe.parent.parent/'Zed.exe'
    if exe.name.lower()!='zed.exe' or not exe.is_file():raise ValueError('该文件夹没有 Zed.exe，请选择官方安装根目录')
    import pefile
    image=pefile.PE(str(exe),fast_load=True)
    try:
        if image.OPTIONAL_HEADER.Subsystem!=2:raise ValueError('选中的是命令行程序，请选择包含图形版 Zed.exe 的根目录')
    finally:image.close()
    return exe.resolve()

def inspect_folder(folder):
    try:
        import hashlib
        exe=resolve_zed(folder)
        with exe.open('rb') as file:digest=hashlib.file_digest(file,'sha256').hexdigest()
        data=json.loads((ROOT/'data/ui-adapters.json').read_text(encoding='utf-8'))['adapters']
        cache=LOCAL/'verified-adapters.json'
        if cache.exists():data+=json.loads(cache.read_text(encoding='utf-8'))['adapters']
        approved=any(item.get('verified') and item['sha256']==digest for item in data)
        return f'已找到图形版：{exe}\n'+('当前安装包已适配，可应用中文显示补丁。' if approved else '当前安装包未适配，允许正常启动英文版；不会盲打补丁。')
    except ValueError as error:return str(error)
    except Exception as error:return f'检测失败（{type(error).__name__}）；请核对安装路径。'

def choose_folder(current):
    try:
        result=subprocess.run(backend_command(ROOT/'runtime/select_folder.py',current),
            stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,
            creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0),timeout=300)
        selected=result.stdout.decode('utf-8').strip()
        return selected or current, '已选择文件夹，请检测后保存。' if selected else '已取消选择，原路径未改变。'
    except Exception as error:return current,f'选择器不可用（{type(error).__name__}）；可以直接输入文件夹路径。'

def run_launcher(*args):
    subprocess.Popen([str(launcher()),*args],cwd=ROOT,
        stdin=subprocess.DEVNULL,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,
        creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))

def save_startup(folder,mode,replace):
    previous=settings()
    try:
        exe=resolve_zed(folder)
        if mode not in MODES:raise ValueError('请选择启动方式')
        current=dict(previous)
        current.update(zed_exe=str(exe),startup_mode=MODES[mode],replace_shortcuts=bool(replace),initialized=True)
        write_json(CONFIG,current)
        result=subprocess.run(['powershell.exe','-NoProfile','-ExecutionPolicy','Bypass','-File',
            str(ROOT/'runtime/install-startup.ps1')],cwd=ROOT,stdin=subprocess.DEVNULL,
            stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL,timeout=40,
            creationflags=getattr(subprocess,'CREATE_NO_WINDOW',0))
        if result.returncode:
            write_json(CONFIG,previous)
            return '启动配置应用失败，已还原配置；没有修改 Zed 用户资料。'
        run_launcher('--watch')
        return '启动配置已保存，托盘已启动。'+('下次 Windows 登录自动启动补丁，不自动打开 Zed。' if current['startup_mode']=='auto' else '开机自启已关闭；下次需手动启动补丁，或使用接管后的 Zed 快捷方式。')+' 原设置、登录和插件资料继续使用原目录。'
    except ValueError as error:return str(error)
    except Exception as error:
        write_json(CONFIG,previous)
        return f'启动配置未完成（{type(error).__name__}）；请检查路径和本地安装。'

def launch_chinese():
    try:
        resolve_zed(settings()['zed_exe'])
        run_launcher()
        return '已请求启动中文 Zed，使用现有用户资料。若已有英文窗口，先保存并关闭，再从中文入口启动。'
    except Exception as error:return f'启动失败（{type(error).__name__}）；请先检测并保存安装路径。'
