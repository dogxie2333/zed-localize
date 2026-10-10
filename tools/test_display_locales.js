// Exercise the actual Frida dictionary switching code with offline fixtures.
const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict'),path=require('node:path');
const root=path.resolve(__dirname,'..');
const context=vm.createContext({rpc:{},File:{readAllText:filename=>fs.readFileSync(filename,'utf8')}});
vm.runInContext(fs.readFileSync(path.join(root,'tools/ui_overlay.js'),'utf8'),context);
const languages=JSON.parse(fs.readFileSync(path.join(root,'data/locales.json'))).languages;
context.paths=Object.fromEntries(languages.filter(item=>item.id!=='en').map(item=>[item.id,path.join(root,`assets/locales/${item.id}.json`)]));
vm.runInContext("catalogs=paths;dictionary=new Map();currentLocale='en';",context);
for(const language of languages.filter(item=>item.id!=='en')) {
    context.locale=language.id;
    vm.runInContext('selectLocale(locale)',context);
    assert.equal(vm.runInContext('currentLocale',context),language.id);
    assert.equal(vm.runInContext('dictionary.size',context),language.eligible);
}
vm.runInContext("selectLocale('en')",context);
assert.equal(vm.runInContext('dictionary.size',context),0);
vm.runInContext("selectLocale('../not-a-catalog')",context);
assert.equal(vm.runInContext('currentLocale',context),'en');
console.log(JSON.stringify({passed:true,languages:languages.length,offline:true}));
