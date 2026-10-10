// Route this exact Zed's GUI restart through the already-verified Rust launcher.
// Headless environment/version/crash helpers and unrelated subprocesses are untouched.
function installRestartBridge(parameters) {
    const normalize=value=>{
        let path=value.toLowerCase().replaceAll('/',String.fromCharCode(92));
        if(path.startsWith(String.fromCharCode(92,92,63,92)) || path.startsWith(String.fromCharCode(92,63,63,92)))path=path.slice(4);
        return path;
    };
    const expected=normalize(Process.mainModule.path);
    const address=Process.getModuleByName('kernel32.dll').getExportByName('CreateProcessW');
    if(hooked.has(address.toString()))return;
    hooked.add(address.toString());
    Interceptor.attach(address,{
        onEnter(args){
            this.application=null;this.command=null;
            if(!enabled)return;
            try {
                const command=args[1].isNull()?'':args[1].readUtf16String();
                if(/(?:^|\s|")--(?:printenv|version|help|dump-all-actions|crash-handler)(?:\s|"|$)/.test(command))return;
                const match=command.match(/^\s*(?:"([^"]+)"|(\S+))/);
                const application=args[0].isNull()?(match?(match[1]||match[2]):''):args[0].readUtf16String();
                if(normalize(application)!==expected || !parameters.launcher_path)return;
                const tail=match?command.slice(match[0].length):'';
                this.application=Memory.allocUtf16String(parameters.launcher_path);
                this.command=Memory.allocUtf16String('"'+parameters.launcher_path+'"'+tail);
                args[0]=this.application;args[1]=this.command;
            }catch(_){}
        },
        onLeave(){this.application=null;this.command=null;}
    });
}
