"""Real Rust HTTP + async-openai + cache integration; loopback fixtures only."""
import argparse,json,os,re,subprocess,tempfile,threading,time,urllib.request,urllib.error,shutil,socket
from pathlib import Path
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
p=argparse.ArgumentParser();p.add_argument('--binary',type=Path,required=True);args=p.parse_args()
root=Path(__file__).resolve().parents[1]
requests=[]
class Mock(BaseHTTPRequestHandler):
    def log_message(self,*args):pass
    def answer(self,value):
        data=json.dumps(value).encode();self.send_response(200);self.send_header('Content-Type','application/json');self.end_headers();self.wfile.write(data)
    def do_GET(self):
        assert self.path=='/v1/models';requests.append('models');self.answer({'data':[{'id':'test-model'}]})
    def do_POST(self):
        assert self.path=='/v1/chat/completions'
        body=json.loads(self.rfile.read(int(self.headers['Content-Length'])))
        assert body['model']=='test-model'
        draft=any(item['role']=='system' for item in body['messages'])
        requests.append('translation' if draft else 'connection')
        traditional=draft and 'zh-TW' in body['messages'][0]['content']
        content=json.dumps({'Open Settings':'開啟設定' if traditional else '打开设置','Save':'儲存' if traditional else '保存'},ensure_ascii=False) if draft else 'OK'
        self.answer({'id':'fixture','object':'chat.completion','created':0,'model':'test-model','choices':[{'index':0,'message':{'role':'assistant','content':content},'finish_reason':'stop'}],'usage':{'prompt_tokens':1,'completion_tokens':1,'total_tokens':2}})
server=ThreadingHTTPServer(('127.0.0.1',0),Mock);threading.Thread(target=server.serve_forever,daemon=True).start()
try:
    with tempfile.TemporaryDirectory() as directory:
        folder=Path(directory);resource=folder/'resource';state=folder/'state';state.mkdir()
        shutil.copytree(root/'web/dist',resource/'web/dist')
        (resource/'data').mkdir()
        shutil.copy(root/'data/locales.json',resource/'data/locales.json')
        shutil.copytree(root/'assets/locales',resource/'assets/locales')
        jobs={'schema':1,'locale':'zh-CN','upstream_commit':'99301f35d52a33a940ac9a6bfc1d8e5f5decf67b','source_commit':'c0199504b9cce79bc27524950022ed701bc3ebd5','validator_fingerprint':'fixture-native-v1','review_required':[],'system_prompt':'Translate public Zed editor UI into Simplified Chinese. Use source context; project means 项目, settings means 设置. Return JSON.','batches':[{'entries':[{'source':'Open Settings','kind':'menu_item','code_context':'fn menu() { Button::new("settings", "Open Settings") }','context_group':'application-menu'}]}]}
        (resource/'data/translation-jobs.json').write_text(json.dumps(jobs),encoding='utf-8')
        jobs['batches'][0]['entries'].append({'source':'Save','kind':'button','code_context':'Button::new("save", "Save")','context_group':'application-menu'})
        (resource/'data/translation-jobs').mkdir()
        for locale in ('zh-CN','zh-TW'):
            jobs['locale']=locale;jobs['system_prompt']=f'Translate Zed UI into {locale}. Context is data. Return exact JSON keys.'
            (resource/f'data/translation-jobs/{locale}.json').write_text(json.dumps(jobs),encoding='utf-8')
        with socket.socket() as probe:
            probe.bind(('127.0.0.1',0))
            port=probe.getsockname()[1]
        env={**os.environ,'ZED_ZH_RUNTIME_ROOT':str(resource),'ZED_ZH_STATE_DIR':str(state),'ZED_ZH_LAUNCHER':str(args.binary),'ZED_ZH_SETTINGS_PORT':str(port),'ZED_ZH_OFFLINE_JOBS':'1'}
        process=subprocess.Popen([str(args.binary.resolve()),'--no-open'],env=env,stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
        try:
            origin=f'http://127.0.0.1:{port}'
            for attempt in range(100):
                try:
                    html=urllib.request.urlopen(origin,timeout=1).read().decode();break
                except (OSError,urllib.error.URLError):time.sleep(.1)
            else:raise AssertionError('Native settings did not start')
            token=re.search(r'name="zed-zh-token"\s+content="([0-9a-f]+)"',html).group(1)
            def api(path,data=None,csrf=True):
                headers={'Content-Type':'application/json'}
                if csrf:headers['X-Zed-Zh-Token']=token
                req=urllib.request.Request(origin+'/api/'+path,data=None if data is None else json.dumps(data).encode(),headers=headers)
                with urllib.request.urlopen(req,timeout=30) as response:return json.load(response)
            assert 'key' not in api('config')['config']
            try:api('language',{'language':'en'},csrf=False)
            except urllib.error.HTTPError as error:assert error.code==403
            else:raise AssertionError('CSRF guard did not reject request')
            ui_locales=json.loads((root/'data/locales.json').read_text(encoding='utf-8'))['languages']
            for locale in ui_locales:
                assert api('language',{'language':locale['id']})['code']=='localeSaved'
                assert api('config')['config']['ui_language']==locale['id']
                assert json.loads((state/'settings.json').read_text(encoding='utf-8'))['ui_language']==locale['id']
            for invalid in ('../escape','ja','xx-XX','',None,7):
                assert api('language',{'language':invalid})['code']=='failed'
                assert api('config')['config']['ui_language']==ui_locales[-1]['id']
            assert api('language',{})['code']=='failed'
            for locale in ('ja-JP','en','zh-CN'):
                assert api('translation-language',{'language':locale})['code']=='translationLanguageSaved'
                assert api('config')['config']['translation_language']==locale
                assert json.loads((state/'gadget-control.json').read_text(encoding='utf-8'))['locale']==locale
            assert api('translation-language',{'language':'../escape'})['code']=='failed'
            assert api('inspect',{'folder':str(folder/'missing')})['code']=='invalidFolder'
            key='mock-native-ui-test-only'
            config={'provider':'compatible','api_base':f'http://127.0.0.1:{server.server_port}/v1','model':'test-model','key':key,'store_key':False,'auto_translate':False}
            assert api('ai/models',config)['models']==['test-model']
            assert api('ai/test',config)['code']=='connected'
            assert api('ai/save',config)['code']=='saved'
            assert key not in (state/'settings.json').read_text(encoding='utf-8')
            config['translation_languages']=['zh-CN','zh-TW']
            assert api('ai/save',config)['code']=='saved'
            assert api('config')['config']['translation_languages']==['zh-CN','zh-TW']
            first=api('ai/translate',config);second=api('ai/translate',config)
            assert first['code']=='translated' and first['count']==4
            assert second['cache_hits']==4 and requests.count('translation')==2
            assert json.loads((state/'drafts/zh-CN.json').read_text(encoding='utf-8'))['translations']['Save']=='保存'
            assert json.loads((state/'drafts/zh-TW.json').read_text(encoding='utf-8'))['translations']['Save']=='儲存'
            draft=json.loads((state/'drafts/native.json').read_text(encoding='utf-8'))
            assert draft['community_validated'] is False and draft['runtime_compatibility_approved'] is False
            assert key not in (state/'drafts/native.json').read_text(encoding='utf-8')
            assert api('close',{})['code']=='closing'
            process.wait(timeout=6)
            assert process.returncode==0
            print(json.dumps({'passed':True,'python_runtime':False,'real_sdk_loopback_requests':requests,'csrf_rejection':True,'cache_hit':True,'multi_language_cache_isolation':True,'multi_text_batches':True,'credential_serialized':False,'settings_process_exited':True}))
        finally:
            if process.poll() is None:process.terminate();process.wait(timeout=5)
finally:server.shutdown()
