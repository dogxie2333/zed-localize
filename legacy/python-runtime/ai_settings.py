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
import time

import gradio as gr
import keyring
import litellm
from openai import OpenAI
from common import ROOT, LOCAL, CONFIG, credential_id, settings, write_json, backend_command
from startup_settings import MODES, choose_folder, inspect_folder, save_startup, launch_chinese, close_settings
from ui_theme import theme

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
_auto_close_at=time.time()+120

def configuration_alive():
    global _auto_close_at
    _auto_close_at=None

def configuration_departed():
    global _auto_close_at
    # Reloads/new tabs can reconnect before the SDK-driven grace period ends.
    _auto_close_at=time.time()+15

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
    tool = LOCAL/'bin/zed-zh-maintain.exe'
    if not tool.is_file():tool=ROOT/'zed-zh-maintain.exe'
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
        result = subprocess.run(backend_command(ROOT/'tools/update.py',
            '--maintainer', str(tool), '--config', str(config)), cwd=ROOT, env=environment,
            stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL,
            text=True, encoding='utf-8', timeout=600)
        return '维护任务完成；结果已生成待审译文，不会自动扩大二进制兼容范围。' if result.returncode==0 else '维护任务失败；已验证批次缓存会保留，请检查本地状态。'
    except Exception as error:
        return f'维护未完成（{type(error).__name__}），密钥未写入日志。'
    finally:
        environment.pop('ZED_I18N_API_KEY',None)

def runtime_view():
    def read(name, default):
        try:return json.loads((LOCAL/name).read_text(encoding='utf-8-sig'))
        except (OSError,ValueError):return default
    tray=read('tray-status.json',{})
    runtime=read('runtime-status.json',{'sessions':[]})
    healthy=tray.get('running') and time.time()-tray.get('updated',0)<20
    title=tray.get('state','补丁正在运行') if healthy else '托盘当前未运行'
    sessions=runtime.get('sessions',[]) if time.time()-runtime.get('updated',0)<40 else []
    chinese=sum(item.get('mode')=='chinese' for item in sessions)
    description=f'当前有 **{chinese} 个** Zed 进程启用汉化。' if chinese else '打开已适配的 Zed 后，会在这里显示汉化状态。'
    return f'### {title}\n\n{description}\n\n配置页关闭后，托盘与已启用的汉化继续运行。',runtime

def build():
    current = settings()
    label = next((label for label, value in PROVIDERS.items() if value[0] == current['provider']), 'DeepSeek')
    mark=(ROOT/'assets/icons/mark-light.svg').read_text(encoding='utf-8').replace('stroke="black"','stroke="currentColor"')
    with gr.Blocks(title='Zed 汉化补丁设置', analytics_enabled=False) as page:
        gr.HTML(f'<header class="zh-hero">{mark}<div><p class="zh-eyebrow">本地设置 · 按需运行</p><h1 class="zh-title">Zed 中文补丁</h1><p class="zh-subtitle">熟悉的编辑器，熟悉的中文。保留你的设置、登录与插件。</p></div><span class="zh-local">本机配置</span></header>')
        with gr.Tabs(elem_id='settings-tabs'):
            with gr.Tab('启动设置'):
                gr.Markdown('## 从这里开始\n选择 Zed 的安装目录，设定补丁的启动方式。AI 翻译可以稍后配置。',elem_classes=['zh-intro'])
                with gr.Row(elem_classes=['startup-grid']):
                    with gr.Column(scale=3,min_width=300):
                        folder=gr.Textbox(value=str(Path(current['zed_exe']).parent),label='Zed 安装文件夹',info='选择包含 Zed.exe 的安装根目录。',placeholder='例如 D:\\Zed')
                        with gr.Row():
                            browse=gr.Button('选择文件夹')
                            inspect=gr.Button('检测兼容性')
                        mode=gr.Radio(list(MODES),value=next(label for label,value in MODES.items() if value==current['startup_mode']),label='补丁启动方式')
                        replace=gr.Checkbox(current['replace_shortcuts'],label='接管 Zed 快捷方式',info='通过原来的 Zed 入口启动时，同时加载汉化。')
                        with gr.Row():
                            apply_startup=gr.Button('保存并启用托盘',variant='primary')
                            open_zed=gr.Button('启动中文 Zed')
                        startup_status=gr.Textbox(label='启动状态',interactive=False,value='已有设置和插件继续保留。已适配版本无需配置 AI 即可汉化。',lines=2,elem_classes=['zh-status'])
                    with gr.Column(scale=2,min_width=260):
                        gr.HTML('<aside class="zh-guide"><h3>设置一次，照常使用</h3><div class="zh-guide-item"><span class="zh-step">01</span><div><strong>找到编辑器</strong><p>选择安装根目录，检测版本兼容性。用户资料目录与 bin 子目录无需选择。</p></div></div><div class="zh-guide-item"><span class="zh-step">02</span><div><strong>选择启动方式</strong><p>开机自启只启动右下角托盘。手动启动则通过补丁或接管后的 Zed 快捷方式启用。</p></div></div><div class="zh-guide-item"><span class="zh-step">03</span><div><strong>开始编辑</strong><p>保存后照常打开 Zed。重启编辑器会重新加载汉化，已有设置、登录与插件继续保留。</p></div></div></aside>')
                        gr.Markdown('关闭配置网页后，汉化继续运行。下次修改设置，在右下角托盘图标上右键打开。',elem_classes=['zh-soft-note'])
                        close_page=gr.Button('完成设置 · 关闭配置服务')
                browse.click(choose_folder,folder,[folder,startup_status])
                inspect.click(inspect_folder,folder,startup_status)
                apply_startup.click(save_startup,[folder,mode,replace],startup_status)
                open_zed.click(launch_chinese,outputs=startup_status)
                close_page.click(close_settings,outputs=startup_status)
            with gr.Tab('AI 翻译'):
                gr.Markdown('## 可选的翻译维护\n更新时批量处理新增文案。日常编辑使用本地译文，不调用 AI。',elem_classes=['zh-intro'])
                with gr.Row():
                    provider = gr.Dropdown(list(PROVIDERS), value=label, label='供应商',scale=1)
                    endpoint = gr.Textbox(value=current['api_base'], label='API 地址',scale=2)
                with gr.Row():
                    model = gr.Dropdown(choices=[current['model']] if current['model'] else [], value=current['model'] or None,
                                        allow_custom_value=True, label='模型',info='可查询，也可手动输入模型名称。')
                    key = gr.Textbox(type='password', label='API 密钥', value='', placeholder='输入密钥，仅在本地使用',info='已保存的密钥不会显示在这里。')
                store = gr.Checkbox(False, label='将密钥保存到 Windows 凭据管理器')
                automatic = gr.Checkbox(current.get('auto_translate',False), label='更新维护时自动翻译新增文案',info='需要你选择保存密钥；翻译不会在编辑过程中运行。')
                with gr.Row():
                    save_button = gr.Button('保存 AI 配置', variant='primary')
                    query = gr.Button('查询模型')
                    test = gr.Button('测试连接')
                    maintain = gr.Button('检查新增文案并翻译')
                status = gr.Textbox(label='连接与维护状态', interactive=False, value='等待配置。测试连接与翻译仅在你点击后运行。',lines=2,elem_classes=['zh-status'])
                provider.change(provider_changed,provider,[endpoint,model,key,status])
                query.click(model_list,[provider,endpoint,key],[model,status])
                test.click(test_connection,[provider,endpoint,model,key],status)
                save_button.click(save,[provider,endpoint,model,key,store,automatic],status)
                maintain.click(translate_pending,[provider,endpoint,model,key],status)
            with gr.Tab('运行状态'):
                gr.Markdown('## 当前运行情况\n已适配版本自动汉化。新版本的兼容性经过验证后才会启用。',elem_classes=['zh-intro'])
                initial_summary,initial_runtime=runtime_view()
                summary=gr.Markdown(initial_summary)
                refresh = gr.Button('刷新状态',scale=0)
                with gr.Accordion('查看详细运行数据',open=False):
                    runtime = gr.JSON(label='本机运行状态',value=initial_runtime)
                refresh.click(runtime_view,outputs=[summary,runtime])
            with gr.Tab('使用说明'):
                gr.Markdown((ROOT/'docs/LOCAL_USAGE.md').read_text(encoding='utf-8') if (ROOT/'docs/LOCAL_USAGE.md').exists() else '使用说明正在生成。')
        gr.HTML('<footer class="zh-footer"><span>Zed 中文补丁 · 独立社区工具</span><span>配置仅在本机保存 · 官方更新渠道继续保留</span></footer>')
        page.load(configuration_alive,api_name=False,queue=False)
        page.unload(configuration_departed)
        gr.Timer(5).tick(configuration_alive,api_name=False,queue=False,show_progress='hidden')
    return page

if __name__ == '__main__':
    from filelock import FileLock, Timeout
    import webbrowser
    import time
    LOCAL.mkdir(exist_ok=True)
    try:
        with FileLock(LOCAL / 'settings-ui.lock', timeout=0):
            opened=time.time()
            page=build()
            page.launch(server_name='127.0.0.1', server_port=18743, inbrowser=True, share=False, show_error=False,
                           run_history=False, enable_monitoring=False, strict_cors=True, quiet=True,prevent_thread_lock=True,
                           theme=theme(),css_paths=[ROOT/'runtime/settings.css'],favicon_path=ROOT/'assets/icons/app.ico',footer_links=[])
            stop=LOCAL/'settings-stop.json'
            try:
                while True:
                    if stop.exists() and json.loads(stop.read_text(encoding='utf-8'))['time']>=opened:break
                    if _auto_close_at is not None and time.time()>=_auto_close_at:break
                    time.sleep(0.5)
            finally:page.close()
    except Timeout:
        webbrowser.open('http://127.0.0.1:18743')
