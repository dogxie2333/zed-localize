// Maintainer microbenchmark only. The synthetic caller belongs to this RPC
// thread; other threads retain the real stack check. Never packaged for users.
let benchmarkThread=0;
function benchmarkBacktrace(context,mode,threadId){
    if(threadId===benchmarkThread)return [Process.mainModule.base.add(marker)];
    return Thread.backtrace(context,mode);
}
rpc.exports.benchmark=function(){
 send({phase:'factory'});
 const module=Process.getModuleByName('dwrite.dll');
 const iid=Memory.alloc(16);iid.writeByteArray([0x5a,0xee,0x59,0xb8,0x38,0xd8,0x5b,0x4b,0xa2,0xe8,0x1a,0xdc,0x7d,0x93,0xdb,0x48]);
 const output=Memory.alloc(Process.pointerSize);
 const create=new NativeFunction(module.getExportByName('DWriteCreateFactory'),'int',['uint','pointer','pointer'],{traps:'all'});
 if(create(1,iid,output)<0)throw new Error('Factory creation failed');
 const factory=output.readPointer(),table=factory.readPointer();
 const formatOutput=Memory.alloc(Process.pointerSize);
 send({phase:'format'});
 const createFormat=new NativeFunction(table.add(15*Process.pointerSize).readPointer(),'int',['pointer','pointer','pointer','uint','uint','uint','float','pointer','pointer'],{traps:'all'});
 const family=Memory.allocUtf16String(Array.from(fonts)[0]);
 if(createFormat(factory,family,ptr(0),400,0,5,14,Memory.allocUtf16String('en-US'),formatOutput)<0)throw new Error('Format creation failed');
 const format=formatOutput.readPointer();
 const layout=new NativeFunction(table.add(18*Process.pointerSize).readPointer(),'int',['pointer','pointer','uint','pointer','float','float','pointer'],{traps:'all'});
 const release=new NativeFunction(factory.readPointer().add(2*Process.pointerSize).readPointer(),'uint',['pointer']);
 const releaseFormat=new NativeFunction(format.readPointer().add(2*Process.pointerSize).readPointer(),'uint',['pointer']);
 const kernel=Process.getModuleByName('kernel32.dll');
 const qpc=new NativeFunction(kernel.getExportByName('QueryPerformanceCounter'),'int',['pointer']);
 const frequency=Memory.alloc(8);new NativeFunction(kernel.getExportByName('QueryPerformanceFrequency'),'int',['pointer'])(frequency);
 const hz=Number(frequency.readS64().toString()),counter=Memory.alloc(8);
 const tick=()=>{qpc(counter);return Number(counter.readS64().toString());};
 const text='Open Settings',source=Memory.allocUtf16String(text),result=Memory.alloc(Process.pointerSize);
 benchmarkThread=Process.getCurrentThreadId();
 send({phase:'layouts'});
 const run=()=>{if(layout(factory,source,text.length,format,800,100,result)<0)throw new Error('Layout failed');const object=result.readPointer();new NativeFunction(object.readPointer().add(2*Process.pointerSize).readPointer(),'uint',['pointer'])(object);};
 for(let i=0;i<100;i++)run();
 const samples=[];
 for(let batch=0;batch<20;batch++){const start=tick();for(let i=0;i<100;i++)run();samples.push((tick()-start)*1000/hz/100);}
 const translated=hits.get(text)||0;
 benchmarkThread=0;
 const denyBefore=translated;run();
 if((hits.get(text)||0)!==denyBefore)throw new Error('Editor/caller isolation failed');
 const stats=rpc.exports.stats();
 releaseFormat(format);release(factory);
 samples.sort((a,b)=>a-b);
 if(translated<2100)throw new Error('Benchmark did not exercise translation');
 return {median_ms:samples[10],p95_ms:samples[19],calls:2100,caller_protection:true,stats};
};
