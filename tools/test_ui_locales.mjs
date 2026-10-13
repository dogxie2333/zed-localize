// Offline settings-language regression: run with node tools/test_ui_locales.mjs.
// Compiles the real Vue setup and exercises it with loopback-free API fixtures.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import {createRequire} from 'node:module';
import {fileURLToPath} from 'node:url';
import {messages} from '../web/src/locales.js';

const root=new URL('../',import.meta.url);
const read=path=>fs.readFileSync(new URL(path,root),'utf8');
const require=createRequire(new URL('web/package.json',root));
const vue=require('vue');
const {parse,compileScript}=require('@vue/compiler-sfc');
const catalog=JSON.parse(read('data/locales.json'));
const ids=catalog.languages.map(item=>item.id);
const keys=Object.keys(messages.en).sort();
assert.equal(ids.length,14,'Editor catalog must contain 13 translations plus English');
assert.deepEqual(Object.keys(messages).sort(),[...ids].sort());
for(const id of ids){
 const dictionary=messages[id];
 assert.deepEqual(Object.keys(dictionary).sort(),keys,`${id}: all UI keys required`);
 for(const key of keys){
  if(key==='steps'){
   assert.equal(dictionary.steps.length,3,`${id}: all guide steps required`);
   assert.ok(dictionary.steps.every(step=>typeof step==='string'&&step.trim()));
  }else{
   assert.equal(typeof dictionary[key],'string',`${id}.${key}`);
   // The backend's ready response deliberately has no notification text.
   assert.ok(key==='ready'||dictionary[key].trim(),`${id}.${key}: empty translation`);
  }
 }
 assert.ok(dictionary.updateNote.includes('Windows'),`${id}: official Windows guide`);
 if(id!=='en'){
  for(const key of ['pageLanguage','translationLanguage','compatible','unsupported','notCompatible','updateNote']){
   assert.notEqual(dictionary[key],messages.en[key],`${id}.${key}: no English fallback`);
  }
 }
}

const source=read('web/src/App.vue');
const {descriptor,errors}=parse(source);
assert.deepEqual(errors,[]);
const script=compileScript(descriptor,{id:'settings-locale-test',genDefaultAs:'__component'}).content;
const bindings={messages,languageCatalog:catalog};
// Resolve exactly the Element Plus locales imported by App.vue, using the
// already installed dependency. No translated widget strings are duplicated.
for(const match of script.matchAll(/^import (\w+) from '(element-plus\/es\/locale\/lang\/[^']+)';/gm)){
 bindings[match[1]]=(await import(new URL(`web/node_modules/${match[2]}.mjs`,root))).default;
}
const widgetNames={'en':'en','zh-CN':'zh-cn','zh-TW':'zh-tw','ja-JP':'ja','ko-KR':'ko','de-DE':'de','fr-FR':'fr','es-ES':'es','it-IT':'it','pt-BR':'pt-br','ru-RU':'ru','pl-PL':'pl','cs-CZ':'cs','tr-TR':'tr'};
const executable=script.replace(/^import .* from '[^']+';\s*$/gm,'')+'\n__component;';
let persisted={ui_language:'zh-CN',translation_language:'ja-JP',translation_languages:['ko-KR'],provider:'compatible',model:''};
let failure=null;
const requests=[];
const document={querySelector:()=>({content:'offline-ui-fixture'}),documentElement:{lang:'',classList:{toggle(){}}}};
function createApp(){
 let mount;
 const context={
  ...bindings,...vue,onMounted:callback=>{mount=callback;},onBeforeUnmount(){},
  // Component imports are irrelevant to setup; the build verifies rendering.
  ...Object.fromEntries(['ElButton','ElInput','ElSelect','ElOption','ElRadioGroup','ElRadio','ElCheckbox','ElConfigProvider'].map(name=>[name,{}])),
  location:{search:''},window:{matchMedia:()=>({matches:false}),addEventListener(){},removeEventListener(){}},
  document,URLSearchParams,setInterval:()=>1,clearInterval(){},
  fetch:async(path,options)=>{
   const data=options.body?JSON.parse(options.body):undefined;
   requests.push({path,data});
   if(path==='/api/config')return {ok:true,json:async()=>({config:structuredClone(persisted),folder:'',languages:catalog.languages})};
   if(path==='/api/status')return {ok:true,json:async()=>({display:{code:'active'}})};
   assert.equal(path,'/api/language');
   if(failure==='network')throw new Error('offline fixture failure');
   if(failure==='http')return {ok:false};
   if(failure==='rejected')return {ok:true,json:async()=>({code:'failed'})};
   assert.ok(ids.includes(data.language));
   persisted.ui_language=data.language;
   return {ok:true,json:async()=>({code:'localeSaved'})};
  }
 };
 const component=vm.runInNewContext(executable,context,{filename:fileURLToPath(new URL('web/src/App.vue',root))});
 const app=component.setup({}, {expose(){}});
 return {app,mount:()=>mount()};
}
const {app,mount}=createApp();
await mount();
assert.deepEqual(Array.from(app.uiLanguages,item=>item.id),ids,'Page selector uses the editor catalog');
assert.deepEqual(Object.keys(app.elementLocales).sort(),[...ids].sort());
assert.match(descriptor.template.content,/v-for="item in uiLanguages"/);
assert.match(descriptor.template.content,/:aria-label="t\.navigation"/);
const css=read('web/src/style.css');
assert.match(css,/\.el-select__popper \.el-scrollbar__wrap\{max-height:min\(274px,calc\(100vh - 96px\)\)\}/,'14 options remain in a bounded scroll area');
assert.ok(descriptor.styles.some(style=>style.content.includes('overflow-y:auto')&&style.content.includes('white-space:normal')),'Long localized navigation can wrap and scroll');
for(const id of ids){
 app.language.value=id;
 await app.changeLanguage();
 assert.equal(app.t.value,messages[id],`${id}: selected dictionary`);
 assert.equal(app.locale.value.name,widgetNames[id],`${id}: Element Plus locale`);
 assert.equal(app.providerNames.value.find(item=>item.id==='compatible').label,messages[id].compatibleProvider);
 assert.equal(app.noticeText.value,messages[id].localeSaved);
 assert.equal(document.documentElement.lang,id);
 assert.equal(persisted.ui_language,id);
 assert.equal(persisted.translation_language,'ja-JP','UI selection must not change editor language');
 assert.deepEqual(persisted.translation_languages,['ko-KR'],'UI selection must not change AI languages');
 const reload=createApp();
 await reload.mount();
 assert.equal(reload.app.language.value,id,`${id}: restores saved language`);
 assert.equal(reload.app.locale.value.name,widgetNames[id]);
 assert.equal(reload.app.t.value,messages[id]);
}
for(const kind of ['rejected','http','network']){
 failure=kind;
 app.language.value='ja-JP';
 await app.changeLanguage();
 assert.equal(app.language.value,'tr-TR',`${kind}: revert failed save`);
 assert.equal(app.config.value.ui_language,'tr-TR');
 assert.equal(document.documentElement.lang,'tr-TR');
 assert.equal(app.noticeText.value,messages['tr-TR'].failed);
 assert.equal(app.busy.value,false);
}
assert.equal(requests.filter(item=>item.path==='/api/language').length,17);
console.log(JSON.stringify({passed:true,languages:ids.length,keysPerLanguage:keys.length,guideSteps:3,elementPlusLocales:14,persistenceReload:true,failedSaveRollback:true,editorAiLanguageIsolation:true,boundedLanguageMenu:true,offline:true}));
