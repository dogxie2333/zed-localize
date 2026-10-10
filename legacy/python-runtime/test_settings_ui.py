"""Integration test for real Gradio callbacks, LiteLLM and model-list SDK.

Only loopback mock credentials; never use a user's API key.
"""
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT/'runtime'))
import ai_settings as ui
import update_compat

requests = []
class Handler(BaseHTTPRequestHandler):
    def log_message(self, *args): pass
    def reply(self, body):
        data=json.dumps(body).encode()
        self.send_response(200)
        self.send_header('Content-Type','application/json')
        self.send_header('Content-Length',str(len(data)))
        self.end_headers()
        self.wfile.write(data)
    def do_GET(self):
        assert self.path=='/v1/models'
        requests.append('models')
        self.reply({'object':'list','data':[{'id':'test-model','object':'model','created':0,'owned_by':'mock'}]})
    def do_POST(self):
        assert self.path=='/v1/chat/completions'
        body=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        assert body['model']=='test-model'
        requests.append('completion')
        self.reply({'id':'mock','object':'chat.completion','created':0,'model':'test-model',
            'choices':[{'index':0,'message':{'role':'assistant','content':'OK'},'finish_reason':'stop'}],
            'usage':{'prompt_tokens':1,'completion_tokens':1,'total_tokens':2}})

server=ThreadingHTTPServer(('127.0.0.1',0),Handler)
threading.Thread(target=server.serve_forever,daemon=True).start()
try:
    endpoint=f'http://127.0.0.1:{server.server_port}/v1'
    with tempfile.TemporaryDirectory() as folder:
        config=Path(folder)/'settings.json'
        with patch.object(ui,'CONFIG',config), patch.object(ui,'settings',lambda: {'watch_enabled':True}), \
             patch.object(ui.keyring,'get_password',return_value=None), patch.object(ui.keyring,'set_password') as vault:
            update,status=ui.model_list('其他 OpenAI 兼容服务',endpoint,'mock-test-only')
            assert update['choices']==['test-model'] and '1 个模型' in status
            assert '连接成功' in ui.test_connection('其他 OpenAI 兼容服务',endpoint,'test-model','mock-test-only')
            assert '配置已保存' in ui.save('其他 OpenAI 兼容服务',endpoint,'test-model','mock-test-only',False,False)
            assert 'mock-test-only' not in config.read_text(encoding='utf-8')
            vault.assert_not_called()
            assert '需要已保存' in ui.save('其他 OpenAI 兼容服务',endpoint,'test-model','',False,True)
            assert not json.loads(config.read_text())['auto_translate']
            for bad in ('https://','http://example.com','https://example.com?key=mock','https://user:mock@example.com'):
                try: ui.endpoint_valid(bad)
                except ValueError: pass
                else: raise AssertionError('Unsafe endpoint accepted')
            changed=ui.provider_changed('GLM（国内 BigModel）')
            assert changed[0]=='https://open.bigmodel.cn/api/paas/v4' and changed[1]['choices']==[] and changed[2]==''
        assert len(ui.build().blocks)>15
    with patch.object(update_compat,'settings',lambda:{'auto_translate':False}), patch.object(ui.keyring,'get_password') as vault:
        update_compat.automatic_translation()
        vault.assert_not_called()
    with tempfile.TemporaryDirectory() as folder:
        local=Path(folder);(local/'bin').mkdir();(local/'bin/zed-zh-maintain.exe').touch()
        current={'auto_translate':True,'provider':'compatible','api_base':endpoint,'model':'test-model'}
        with patch.object(update_compat,'LOCAL',local),patch.object(update_compat,'settings',lambda:current), \
             patch.object(ui.keyring,'get_password',return_value='mock-auto-only'), \
             patch('subprocess.run') as execute:
            execute.return_value.returncode=0
            update_compat.automatic_translation()
            execute.assert_called_once()
            assert execute.call_args.kwargs['env'].get('ZED_I18N_API_KEY') is None # removed after child run
            assert all('mock-auto-only' not in file.read_text(encoding='utf-8') for file in local.glob('*.*'))
    print(json.dumps({'passed':True,'real_sdk_loopback_requests':requests,'credentials_serialized':False}))
finally:
    server.shutdown()
