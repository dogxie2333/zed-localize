<script setup>
import {ref, computed, onMounted, onBeforeUnmount, watch} from 'vue';
import {ElButton,ElInput,ElSelect,ElOption,ElRadioGroup,ElRadio,ElCheckbox,ElConfigProvider} from 'element-plus';
import zhCN from 'element-plus/es/locale/lang/zh-cn';
import zhTW from 'element-plus/es/locale/lang/zh-tw';
import en from 'element-plus/es/locale/lang/en';
import ja from 'element-plus/es/locale/lang/ja';
import ko from 'element-plus/es/locale/lang/ko';
import de from 'element-plus/es/locale/lang/de';
import fr from 'element-plus/es/locale/lang/fr';
import es from 'element-plus/es/locale/lang/es';
import it from 'element-plus/es/locale/lang/it';
import ptBR from 'element-plus/es/locale/lang/pt-br';
import ru from 'element-plus/es/locale/lang/ru';
import pl from 'element-plus/es/locale/lang/pl';
import cs from 'element-plus/es/locale/lang/cs';
import tr from 'element-plus/es/locale/lang/tr';
import languageCatalog from '../../data/locales.json';
import {messages} from './locales';
const tab=ref('startup'),language=ref('zh-CN'),config=ref({}),folder=ref(''),key=ref(''),storeKey=ref(false),models=ref([]),runtime=ref({}),busy=ref(false),notice=ref(null);
const translationLanguages=ref([]),aiLanguages=ref([]);
const forcedTheme=new URLSearchParams(location.search).get('__theme');
const dark=ref(forcedTheme==='dark'||(forcedTheme!=='light'&&window.matchMedia('(prefers-color-scheme:dark)').matches));
const t=computed(()=>messages[language.value]||messages['zh-CN']);
const uiLanguages=languageCatalog.languages;
const elementLocales={'zh-CN':zhCN,'zh-TW':zhTW,en,'ja-JP':ja,'ko-KR':ko,'de-DE':de,'fr-FR':fr,'es-ES':es,'it-IT':it,'pt-BR':ptBR,'ru-RU':ru,'pl-PL':pl,'cs-CZ':cs,'tr-TR':tr};
const locale=computed(()=>elementLocales[language.value]||zhCN);
const providerNames=computed(()=>[{id:'deepseek',label:'DeepSeek'},{id:'glm',label:'GLM · BigModel'},{id:'zai',label:'GLM · Z.AI'},{id:'compatible',label:t.value.compatibleProvider}]);
const token=document.querySelector('meta[name=zed-zh-token]').content;
let heartbeat,statusTimer;
async function api(path,data){
 const response=await fetch('/api/'+path,{method:data===undefined?'GET':'POST',headers:{'Content-Type':'application/json','X-Zed-Zh-Token':token},...(data!==undefined?{body:JSON.stringify(data)}:{})});
 if(!response.ok)throw new Error('request');
 return response.json();
}
async function action(path,data){busy.value=true;notice.value=null;try{const result=await api(path,data);notice.value=result.code?result:null;if(result.models){models.value=result.models;if(!config.value.model)config.value.model=result.models[0]||'';}if(result.folder)folder.value=result.folder;if(path==='status')runtime.value=result;if(path==='ai/save'&&result.code==='saved')key.value='';return result;}catch{notice.value={code:'failed'};}finally{busy.value=false;}}
function aiInput(){return {provider:config.value.provider,api_base:config.value.api_base,model:config.value.model||'',key:key.value,store_key:storeKey.value,auto_translate:!!config.value.auto_translate,translation_languages:aiLanguages.value};}
function changeProvider(){const defaults={deepseek:['https://api.deepseek.com/v1','deepseek-flash'],glm:['https://open.bigmodel.cn/api/paas/v4',''],zai:['https://api.z.ai/api/paas/v4',''],compatible:['','']}[config.value.provider];[config.value.api_base,config.value.model]=defaults;key.value='';models.value=defaults[1]?[defaults[1]]:[];storeKey.value=false;notice.value=null;}
async function changeLanguage(){
 const previous=config.value.ui_language||'zh-CN';
 const result=await action('language',{language:language.value});
 if(result?.code==='localeSaved'){config.value.ui_language=language.value;}else{language.value=previous;}
 document.documentElement.lang=language.value;
}
async function changeTranslationLanguage(value){
 const previous=config.value.translation_language;
 const result=await action('translation-language',{language:value});
 if(result?.code==='translationLanguageSaved'){config.value.translation_language=value;}else{config.value.translation_language=previous;}
}
function theme(){document.documentElement.classList.toggle('dark',dark.value);}
watch(dark,theme);
const state=computed(()=>runtime.value.display?.code||'checking');
const noticeText=computed(()=>notice.value?(t.value[notice.value.code]||t.value.failed)+(notice.value.count!==undefined?' · '+notice.value.count:''):'');
onMounted(async()=>{theme();try{const result=await api('config');config.value=result.config;config.value.show_tray_icon=config.value.show_tray_icon!==false;folder.value=result.folder;translationLanguages.value=result.languages||[];language.value=config.value.ui_language||'zh-CN';document.documentElement.lang=language.value;aiLanguages.value=config.value.translation_languages||[];models.value=config.value.model?[config.value.model]:[];runtime.value=await api('status');}catch{notice.value={code:'failed'};}heartbeat=setInterval(()=>api('ping',{}).catch(()=>{}),5000);statusTimer=setInterval(()=>api('status').then(value=>runtime.value=value).catch(()=>{}),2000);});
function leave(){fetch('/api/leave',{method:'POST',headers:{'Content-Type':'application/json','X-Zed-Zh-Token':token},body:'{}',keepalive:true}).catch(()=>{});}
window.addEventListener('pagehide',leave);
onBeforeUnmount(()=>{clearInterval(heartbeat);clearInterval(statusTimer);window.removeEventListener('pagehide',leave);});
</script>
<template>
 <ElConfigProvider :locale="locale">
 <aside class="sidebar">
  <div class="brand"><img src="/mark.svg" alt=""/><span>Localize</span></div>
  <nav :aria-label="t.navigation"><button v-for="name in ['startup','ai','runtime','help']" :key="name" :class="{selected:tab===name}" :aria-current="tab===name?'page':undefined" @click="tab=name;notice=null">{{t[name]}}</button></nav>
  <div class="sidebar-bottom"><div class="sidebar-state" role="status"><span class="state-dot"/>{{t[state]}}</div><label>{{t.pageLanguage}}</label><ElSelect v-model="language" :aria-label="t.pageLanguage" :disabled="busy" @change="changeLanguage"><ElOption v-for="item in uiLanguages" :key="item.id" :label="item.name" :value="item.id"/></ElSelect><button class="theme-button" :aria-label="t.theme" @click="dark=!dark">{{dark?'◐':'◑'}}</button></div>
 </aside>
 <main class="content">
  <h1>{{t[tab]}}</h1>
  <section v-show="tab==='startup'" class="panel">
   <label>{{t.translationLanguage}}</label>
   <ElSelect :model-value="config.translation_language" :aria-label="t.translationLanguage" :disabled="busy" @change="changeTranslationLanguage"><ElOption v-for="item in translationLanguages" :key="item.id" :label="item.name" :value="item.id"/></ElSelect>
   <label for="zed-folder">{{t.folder}}</label>
   <div class="inline"><ElInput id="zed-folder" v-model="folder"/><ElButton :disabled="busy" @click="action('browse',{folder,language})">{{t.browse}}</ElButton></div>
   <ElButton :disabled="busy" @click="action('inspect',{folder})">{{t.inspect}}</ElButton>
   <label>{{t.mode}}</label>
   <ElRadioGroup v-model="config.startup_mode"><ElRadio value="auto">{{t.auto}}</ElRadio><ElRadio value="manual">{{t.manual}}</ElRadio></ElRadioGroup>
   <ElCheckbox v-model="config.replace_shortcuts">{{t.shortcuts}}</ElCheckbox>
   <ElCheckbox v-model="config.show_tray_icon">{{t.showTray}}</ElCheckbox>
   <div class="actions"><ElButton type="primary" :disabled="busy" @click="action('startup',{folder,startup_mode:config.startup_mode,replace_shortcuts:!!config.replace_shortcuts,show_tray_icon:!!config.show_tray_icon})">{{t.saveStartup}}</ElButton><ElButton :disabled="busy" @click="action('launch',{})">{{t.launch}}</ElButton></div>
  </section>
  <section v-show="tab==='ai'" class="panel">
   <label>{{t.aiLanguages}}</label><ElSelect v-model="aiLanguages" multiple collapse-tags :aria-label="t.aiLanguages" :placeholder="t.currentLanguage"><ElOption v-for="item in translationLanguages.filter(item=>item.id!=='en')" :key="item.id" :label="item.name" :value="item.id"/></ElSelect>
   <div class="form-grid"><div><label>{{t.provider}}</label><ElSelect v-model="config.provider" :aria-label="t.provider" @change="changeProvider"><ElOption v-for="p in providerNames" :key="p.id" :label="p.label" :value="p.id"/></ElSelect></div><div><label for="api-base">{{t.endpoint}}</label><ElInput id="api-base" v-model="config.api_base"/></div></div>
   <div class="form-grid"><div><label>{{t.model}}</label><ElSelect v-model="config.model" filterable allow-create default-first-option :aria-label="t.model" :placeholder="t.manualModel"><ElOption v-for="model in models" :key="model" :label="model" :value="model"/></ElSelect></div><div><label for="api-key">{{t.key}}</label><ElInput id="api-key" v-model="key" type="password" autocomplete="off"/></div></div>
   <ElCheckbox v-model="storeKey">{{t.saveKey}}</ElCheckbox><ElCheckbox v-model="config.auto_translate">{{t.autoTranslate}}</ElCheckbox>
   <div class="actions"><ElButton type="primary" :disabled="busy" @click="action('ai/save',aiInput())">{{t.saveAI}}</ElButton><ElButton :disabled="busy" @click="action('ai/models',aiInput())">{{t.models}}</ElButton><ElButton :disabled="busy" @click="action('ai/test',aiInput())">{{t.test}}</ElButton><ElButton :disabled="busy" @click="action('ai/translate',aiInput())">{{t.translate}}</ElButton></div>
  </section>
  <section v-show="tab==='runtime'" class="panel"><div class="runtime-state"><span class="state-dot"/>{{t[state]}}</div><ElButton :disabled="busy" @click="action('status')">{{t.refresh}}</ElButton><details><summary>{{t.details}}</summary><pre>{{JSON.stringify(runtime,null,2)}}</pre></details></section>
  <section v-show="tab==='help'" class="panel guide"><ol><li v-for="step in t.steps" :key="step">{{step}}</li></ol><p>{{t.updateNote}}</p><h2>{{t.ack}}</h2><p>Zed Industries · LI-NA/zed-i18n · Frida · dll-syringe · Vue · Element Plus · Axum · async-openai · redb · keyring · rfd</p></section>
  <div v-if="busy||noticeText" class="notice" role="status">{{busy?t.busy:noticeText}}</div>
  <div class="bottom-actions"><ElButton :disabled="busy" @click="action('close',{})">{{t.close}}</ElButton></div>
 </main>
 </ElConfigProvider>
</template>
<style scoped>
.sidebar{overflow-y:auto}
.brand,nav,.sidebar-bottom{flex-shrink:0}
nav button{height:auto;min-height:48px;white-space:normal;overflow-wrap:anywhere;padding:12px 16px}
.actions :deep(.el-button){height:auto;white-space:normal}
.actions :deep(.el-button>span){overflow-wrap:anywhere;line-height:1.5}
@media(max-width:750px){nav button{min-height:46px;padding:10px}}
</style>
