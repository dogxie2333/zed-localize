// Research observer: retain only known public catalogue text and image RVAs.
let observer;
const hooks = new Set();
const samples = new Map();
const calls = new Map();
const formats = new Map();
let known;
function fontName(format) {
    const table = format.readPointer();
    const length = new NativeFunction(table.add(20 * Process.pointerSize).readPointer(), 'uint', ['pointer'])(format);
    if (length > 200) return '?';
    const buffer = Memory.alloc((length + 1) * 2);
    const result = new NativeFunction(table.add(21 * Process.pointerSize).readPointer(), 'int', ['pointer','pointer','uint'])(format,buffer,length+1);
    return result >= 0 ? buffer.readUtf16String(length) : '?';
}
function install(factory) {
    const table = factory.readPointer();
    const method = table.add(18 * Process.pointerSize).readPointer();
    if (hooks.has(method.toString())) return;
    hooks.add(method.toString());
    Interceptor.attach(method,{onEnter(args) {
        const n = args[2].toUInt32();
        if (!n || n > 1000 || args[1].isNull()) return;
        const text = args[1].readUtf16String(n);
        if (!known.has(text)) return;
        calls.set(text,(calls.get(text)||0)+1);
        const font = fontName(args[3]);
        const main = Process.mainModule;
        const frames = Thread.backtrace(this.context,Backtracer.ACCURATE)
            .filter(address=>address.compare(main.base)>=0 && address.compare(main.base.add(main.size))<0)
            .map(address=>address.sub(main.base).toString());
        const record = {font,frames};
        const values = samples.get(text)||[];
        if (values.length<12 && !values.some(x=>JSON.stringify(x)===JSON.stringify(record))) values.push(record);
        samples.set(text,values);
    }});
}
rpc.exports = {
    start(sources) {
        known = new Set(sources);
        observer = Process.attachModuleObserver({onAdded(module) {
            if (module.name.toLowerCase()!=='dwrite.dll') return;
            Interceptor.attach(module.getExportByName('DWriteCreateFactory'),{
                onEnter(args) {this.output=args[2];},
                onLeave(result) {if(result.toInt32()>=0) install(this.output.readPointer());}
            });
        }});
        return {installed:true};
    },
    samples() {return {hooks:hooks.size, samples:Object.fromEntries(samples), calls:Object.fromEntries(calls)};}
};
