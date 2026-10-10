"""Fetch the pinned official Frida asset and verify compressed and DLL hashes."""
from pathlib import Path
import hashlib,json,lzma,os,urllib.request
root=Path(__file__).resolve().parents[1]
pin=json.loads((root/'data/gadget.lock.json').read_text())
vendor=Path(os.environ.get('ZED_ZH_VENDOR_DIR',root/'local/vendor'))
vendor.mkdir(parents=True,exist_ok=True)
dll=vendor/f"frida-gadget-{pin['version']}.dll"
if not dll.is_file() or hashlib.sha256(dll.read_bytes()).hexdigest()!=pin['sha256_dll']:
    url=f"https://github.com/frida/frida/releases/download/{pin['version']}/{pin['asset']}"
    with urllib.request.urlopen(urllib.request.Request(url,headers={'User-Agent':'zed-zh-addon-build'}),timeout=120) as response:compressed=response.read()
    assert hashlib.sha256(compressed).hexdigest()==pin['sha256_xz'],'Official asset digest mismatch'
    data=lzma.decompress(compressed)
    assert hashlib.sha256(data).hexdigest()==pin['sha256_dll'],'DLL digest mismatch'
    dll.write_bytes(data)
(vendor/'frida-gadget-origin.json').write_text(json.dumps(pin),encoding='utf-8')
print('Verified pinned official Frida Gadget')
