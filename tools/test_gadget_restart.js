// Appended only by the isolated Rust self-test, never normal runtime scripts.
const originalInit=rpc.exports.init;
rpc.exports.init=function(stage,parameters){
    originalInit(stage,parameters);
    setTimeout(()=>{
        const kernel=Process.getModuleByName('kernel32.dll');
        const create=new NativeFunction(kernel.getExportByName('CreateProcessW'),'int',
            ['pointer','pointer','pointer','pointer','int','uint','pointer','pointer','pointer','pointer'],{traps:'all'});
        const startup=Memory.alloc(104);startup.writeByteArray(new Uint8Array(104));startup.writeU32(104);
        const info=Memory.alloc(24);info.writeByteArray(new Uint8Array(24));
        if(create(ptr(0),Memory.allocUtf16String(__COMMAND__),ptr(0),ptr(0),0,0,ptr(0),ptr(0),startup,info)){
            File.writeAllText(__OWNER_PATH__,JSON.stringify({parent_pid:Process.id,pid:info.add(16).readU32()}));
            const close=new NativeFunction(kernel.getExportByName('CloseHandle'),'int',['pointer']);
            close(info.readPointer());close(info.add(8).readPointer());
        }
    },3000);
};
