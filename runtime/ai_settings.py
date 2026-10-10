"""On-demand AI configuration UI. Gradio UI + LiteLLM transport; loopback only."""
import os
os.environ['GRADIO_ANALYTICS_ENABLED'] = 'False'
os.environ['LITELLM_LOCAL_MODEL_COST_MAP'] = 'True'
os.environ['LITELLM_LOG'] = 'ERROR'
import json
from pathlib import Path
import subprocess
from urllib.parse import urlparse
import logging

import gradio as gr
import keyring
import litellm
from openai import OpenAI
from common import ROOT, LOCAL, CONFIG, credential_id, settings, write_json

litellm.suppress_debug_info = True
for name in ('LiteLLM', 'litellm', 'httpx', 'httpcore'):
    logging.getLogger(name).setLevel(logging.CRITICAL)
PROVIDERS = {
    'DeepSeek': ('deepseek', 'https://api.deepseek.com/v1', 'deepseek-flash'),
    'GLM（国内 BigModel）': ('glm', 'https://open.bigmodel.cn/api/paas/v4', ''),
    'GLM（国际 Z.AI）': ('zai', 'https://api.z.ai/api/paas/v4', ''),
    '其他 OpenAI 兼容服务': ('compatible', '', ''),
}
KEYRING_SERVICE = 'zed-zh-addon-ai'

def endpoint_valid(endpoint):
    parsed = urlparse(endpoint)
    local = parsed.hostname in ('127.0.0.1', 'localhost', '::1')
    if not parsed.hostname or parsed.username or parsed.password or parsed.query or parsed.fragment:
        raise ValueError('API 地址不能包含密钥、用户名或查询参数')
    if parsed.scheme != 'https' and not (parsed.scheme == 'http' and local):
        raise ValueError('公网 API 必须使用 HTTPS；本地服务可使用回环 HTTP')
    return endpoint.rstrip('/')

def provider_changed(provider):
    _, endpoint, model = PROVIDERS[provider]
    return endpoint, gr.update(choices=[model] if model else [], value=model or None), '', '密钥仅在本地使用，不会写入项目配置。'

def resolve_key(provider, endpoint, key):
    if key.strip():
        return key.strip()
    identifier = credential_id(PROVIDERS[provider][0], endpoint)
    return keyring.get_password(KEYRING_SERVICE, identifier) or ''

def model_list(provider, endpoint, key):
    try:
        endpoint = endpoint_valid(endpoint)
        actual = resolve_key(provider, endpoint, key)
        if not actual: return gr.update(), '请填写密钥，或先在页面上自行保存到 Windows 凭据管理器。'
        # OpenAI SDK supplies the mature compatible model-list implementation.
        models = OpenAI(api_key=actual, base_url=endpoint, timeout=20, max_retries=0).models.list()
        names = sorted(item.id for item in models.data)
        return gr.update(choices=names, value=names[0] if names else None), f'已查询到 {len(names)} 个模型；也可手动输入模型名称。'
    except Exception as error:
        return gr.update(), f'查询失败（{type(error).__name__}）；部分供应商不提供模型列表，请手动输入。'

def test_connection(provider, endpoint, model, key):
    try:
        endpoint = endpoint_valid(endpoint)
        actual = resolve_key(provider, endpoint, key)
        if not actual or not model.strip(): return '请填写密钥和模型。'
        extra = {'thinking': {'type': 'disabled'}} if PROVIDERS[provider][0] in ('deepseek', 'glm', 'zai') else {}
        # LiteLLM normalizes transport/provider errors; no custom AI HTTP SDK.
        result = litellm.completion(model='openai/' + model.strip(), api_base=endpoint, api_key=actual,
            messages=[{'role': 'user', 'content': '只回复 OK，用于连接测试。'}], max_tokens=128,
            timeout=25, num_retries=0, extra_body=extra)
        if not result.choices: return '接口返回成功，但没有模型响应。'
        return '连接成功：接口已完成一次小型模型请求。'
    except Exception as error:
        # Never echo provider response bodies, request headers or credentials.
        return f'连接失败（{type(error).__name__}）；请核对地址、模型、额度和密钥。'

def save(provider, endpoint, model, key, store_key, automatic):
    try:
        endpoint = endpoint_valid(endpoint)
        if not model.strip(): return '模型名不能为空。'
        current = settings()
        current.update(provider=PROVIDERS[provider][0], api_base=endpoint, model=model.strip(), auto_translate=automatic)
        if store_key:
            if key.strip():
                keyring.set_password(KEYRING_SERVICE, credential_id(current['provider'], endpoint), key.strip())
        if automatic and not resolve_key(provider, endpoint, ''):
            return '自动翻译需要已保存的系统凭据；请自行勾选保存密钥，或关闭自动翻译。配置尚未保存。'
        write_json(CONFIG, current)
        return '配置已保存。' + ('密钥已由你选择存入 Windows 凭据管理器。' if store_key and key.strip() else '系统凭据未更改。')
    except Exception as error:
        return f'保存失败（{type(error).__name__}），未输出密钥或错误正文。'

def translate_pending(provider, endpoint, model, key):
    """Only explicit maintenance runs pass a key to a short-lived subprocess."""
    try:
        endpoint = endpoint_valid(endpoint)
        actual = resolve_key(provider, endpoint, key)
    except Exception as error:
        return f'配置不可用（{type(error).__name__}）。'
    if not actual: return '请先填写或保存密钥。'
    if not model.strip(): return '请先填写模型。'
    tool = ROOT / 'local/bin/zed-zh-maintain.exe'
    if not tool.exists(): return '维护版尚未安装；请按照本地安装说明构建或下载维护版。'
    config = LOCAL / 'ai-maintenance.toml'
    content = f'api_base={json.dumps(endpoint_valid(endpoint))}\nmodel={json.dumps(model)}\nuse_legacy_max_tokens=true\n'
    if PROVIDERS[provider][0] in ('deepseek','glm','zai'):
        content += '[extra_body.thinking]\ntype="disabled"\n'
    config.write_text(content, encoding='utf-8')
    environment = dict(os.environ)
    environment['ZED_I18N_API_KEY'] = actual
    environment['ZED_I18N_MODEL'] = model.strip()
    try:
        result = subprocess.run([str(ROOT/'.venv/Scripts/python.exe'), str(ROOT/'tools/update.py'),
            '--maintainer', str(tool), '--config', str(config)], cwd=ROOT, env=environment,
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
            text=True, encoding='utf-8', timeout=600)
        return '维护任务完成；结果已生成待审译文，不会自动扩大二进制兼容范围。' if result.returncode==0 else '维护任务失败；已验证批次缓存会保留，请检查本地状态。'
    except Exception as error:
        return f'维护未完成（{type(error).__name__}），密钥未写入日志。'
    finally:
        environment.pop('ZED_I18N_API_KEY',None)

def build():
    current = settings()
    label = next((label for label, value in PROVIDERS.items() if value[0] == current['provider']), 'DeepSeek')
    with gr.Blocks(title='Zed 汉化补丁设置', analytics_enabled=False) as page:
        gr.Markdown('# Zed 汉化补丁设置\nAI 仅用于维护时批量翻译。Zed 启动、输入和绘制不调用模型。')
        with gr.Tab('AI 接入'):
            provider = gr.Dropdown(list(PROVIDERS), value=label, label='供应商')
            endpoint = gr.Textbox(value=current['api_base'], label='API 地址')
            model = gr.Dropdown(choices=[current['model']] if current['model'] else [], value=current['model'] or None,
                                allow_custom_value=True, label='模型（可手动输入）')
            key = gr.Textbox(type='password', label='API 密钥', value='', placeholder='不自动读取或展示聊天中提供的密钥')
            store = gr.Checkbox(False, label='由我选择将密钥保存到 Windows 凭据管理器')
            automatic = gr.Checkbox(current.get('auto_translate',False), label='允许更新维护使用已保存的凭据批量翻译')
            with gr.Row():
                query = gr.Button('查询模型')
                test = gr.Button('测试连接')
                save_button = gr.Button('保存配置', variant='primary')
                maintain = gr.Button('检查新增文案并翻译')
            status = gr.Textbox(label='状态', interactive=False, value='密钥为空；不会自动请求收费接口。')
            provider.change(provider_changed,provider,[endpoint,model,key,status])
            query.click(model_list,[provider,endpoint,key],[model,status])
            test.click(test_connection,[provider,endpoint,model,key],status)
            save_button.click(save,[provider,endpoint,model,key,store,automatic],status)
            maintain.click(translate_pending,[provider,endpoint,model,key],status)
        with gr.Tab('启动与更新'):
            gr.Markdown('桌面/开始菜单 Zed 入口由独立启动器接管；官方文件不修改。已适配版本自动汉化，未知版本保留英文并等待经过验证的适配文件。')
            runtime = gr.JSON(label='本机运行状态')
            refresh = gr.Button('刷新状态')
            refresh.click(lambda: json.loads((LOCAL/'runtime-status.json').read_text(encoding='utf-8')) if (LOCAL/'runtime-status.json').exists() else {'sessions':[]}, outputs=runtime)
        with gr.Tab('使用说明'):
            gr.Markdown((ROOT/'docs/LOCAL_USAGE.md').read_text(encoding='utf-8') if (ROOT/'docs/LOCAL_USAGE.md').exists() else '使用说明正在生成。')
    return page

if __name__ == '__main__':
    from filelock import FileLock, Timeout
    import webbrowser
    LOCAL.mkdir(exist_ok=True)
    try:
        with FileLock(LOCAL / 'settings-ui.lock', timeout=0):
            build().launch(server_name='127.0.0.1', server_port=18743, inbrowser=True, share=False, show_error=False,
                           run_history=False, enable_monitoring=False, strict_cors=True, quiet=True)
    except Timeout:
        webbrowser.open('http://127.0.0.1:18743')
