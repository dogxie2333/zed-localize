// Optional UI-display overlay for one verified Zed build. No network or AI.
let moduleObserver;
let dictionary;
let fonts;
let marker;
const protectedNames = new Set();
let publicKeys = new Set();
const hooked = new Set();
const hits = new Map();
const denied = new Map();
const fontFunctions = new Map();
let enabled = true;
let attachExisting = false;
let currentLocale = 'zh-CN';
let catalogs = {};
function selectLocale(locale) {
    if (!locale || locale === currentLocale) return;
    if (locale !== 'en' && !Object.prototype.hasOwnProperty.call(catalogs,locale)) return;
    const data = locale === 'en' ? {messages:{}} : JSON.parse(File.readAllText(catalogs[locale]));
    if (locale !== 'en' && (data.schema !== 1 || data.locale !== locale)) return;
    const entries = Object.entries(data.messages);
    if (!entries.every(([source,text]) => typeof text === 'string' && text.length > 0 &&
        text.length <= source.length && !/[{}\n\r\t`*_\[\]<>\x00]/.test(source+text))) return;
    dictionary = new Map(entries);
    currentLocale = locale;
    hits.clear();
}
function fontName(format) {
    const table = format.readPointer();
    const key=table.toString();
    let functions=fontFunctions.get(key);
    if (functions===undefined) {
        functions={length:new NativeFunction(table.add(20 * Process.pointerSize).readPointer(),'uint',['pointer']),
                   name:new NativeFunction(table.add(21 * Process.pointerSize).readPointer(),'int',['pointer','pointer','uint'])};
        fontFunctions.set(key,functions);
    }
    const length = functions.length(format);
    if (length > 200) return '';
    const buffer = Memory.alloc((length+1)*2);
    const result = functions.name(format,buffer,length+1);
    return result >= 0 ? buffer.readUtf16String(length) : '';
}
function protectFiles(module) {
    const api=module.findExportByName('CreateFileW');
    if (api===null || hooked.has(api.toString())) return;
    hooked.add(api.toString());
    Interceptor.attach(api,{onEnter(args){
        try {
            if (args[0].isNull()) return;
            // Never retain or report file paths. Only protect exact public
            // dictionary keys that happen to be used as a file basename.
            const path=args[0].readUtf16String();
            const name=path.split(/[\\/]/).pop();
            if (publicKeys.has(name)) protectedNames.add(name);
        } catch (_) {}
    }});
}
function installFactory(factory) {
    const method=factory.readPointer().add(18*Process.pointerSize).readPointer();
    if (hooked.has(method.toString())) return;
    hooked.add(method.toString());
    Interceptor.attach(method,{onEnter(args){
        if (!enabled) return;
        const length=args[2].toUInt32();
        if (!length || length>4096 || args[1].isNull()) return;
        const text=args[1].readUtf16String(length);
        const translated=dictionary.get(text);
        if (translated===undefined || protectedNames.has(text)) return;
        if (!fonts.has(fontName(args[3]))) {denied.set('font',(denied.get('font')||0)+1);return;}
        const main=Process.mainModule;
        // UI TextLayout has a verified caller marker. Editor/terminal shaping
        // bypasses this caller even if the user selects the same font family.
        const ui=Thread.backtrace(this.context,Backtracer.ACCURATE).some(address=>
            address.compare(main.base)>=0 && address.compare(main.base.add(main.size))<0 &&
            address.sub(main.base).toString()===marker);
        if (!ui) {denied.set('caller',(denied.get('caller')||0)+1);return;}
        if (translated.length>length) return;
        // Keep UTF-16 indexing unchanged for DWrite style ranges and GPUI's
        // original UTF-8<->UTF-16 converter. The displayed text is ephemeral.
        this.buffer=Memory.allocUtf16String(translated+' '.repeat(length-translated.length));
        args[1]=this.buffer;
        hits.set(text,(hits.get(text)||0)+1);
    },onLeave(){this.buffer=null;}});
}
function installExistingFactory(module) {
    const iid=Memory.alloc(16);
    iid.writeByteArray([0x5a,0xee,0x59,0xb8,0x38,0xd8,0x5b,0x4b,0xa2,0xe8,0x1a,0xdc,0x7d,0x93,0xdb,0x48]);
    const output=Memory.alloc(Process.pointerSize);
    const create=new NativeFunction(module.getExportByName('DWriteCreateFactory'),'int',['uint','pointer','pointer']);
    if (create(0,iid,output)<0) throw new Error('Cannot obtain DirectWrite interface');
    const factory=output.readPointer();
    installFactory(factory);
    new NativeFunction(factory.readPointer().add(2*Process.pointerSize).readPointer(),'uint',['pointer'])(factory);
}
rpc.exports = {
    configure(config) {
        if (config.image_size!==Process.mainModule.size) throw new Error('Unexpected image size');
        dictionary=new Map(Object.entries(config.messages));
        currentLocale=config.locale||'zh-CN';
        catalogs=config.catalogs||{};
        publicKeys=new Set(config.public_sources||Object.keys(config.messages));
        fonts=new Set(config.ui_fonts);
        marker=config.ui_caller_rva;
        attachExisting=config.attach_existing===true;
        for (const name of config.protected_names||[]) protectedNames.add(name);
        moduleObserver=Process.attachModuleObserver({onAdded(module){
            if (['kernel32.dll','kernelbase.dll'].includes(module.name.toLowerCase())) protectFiles(module);
            if(module.name.toLowerCase()!=='dwrite.dll') return;
            Interceptor.attach(module.getExportByName('DWriteCreateFactory'),{
                onEnter(args){this.output=args[2];},
                onLeave(result){if(result.toInt32()>=0) installFactory(this.output.readPointer());}
            });
            if(attachExisting) installExistingFactory(module);
        }});
        return {messages:dictionary.size,enabled:true};
    },
    stats(){return {translated_keys:Object.fromEntries(hits),denied:Object.fromEntries(denied),protected_count:protectedNames.size};},
    disable(){enabled=false;Interceptor.detachAll();return {disabled:true};}
};
