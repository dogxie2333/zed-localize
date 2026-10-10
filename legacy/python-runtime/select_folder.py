"""Use the standard Tk directory picker, never upload selected files."""
from pathlib import Path
import sys
import tkinter as tk
from tkinter import filedialog

if __name__=='__main__':
    window=tk.Tk()
    window.withdraw()
    window.attributes('-topmost',True)
    initial=sys.argv[1] if len(sys.argv)>1 else ''
    folder=filedialog.askdirectory(parent=window,title='选择 Zed 安装文件夹（包含 Zed.exe）',
        initialdir=initial if Path(initial).is_dir() else None,mustexist=True)
    window.destroy()
    sys.stdout.buffer.write(folder.encode('utf-8'))
