// Diagnostic cold-page profile. Owns and reaps one browser; no application code runs in Node.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawn} from 'node:child_process';
const [output,base='http://127.0.0.1:4317/',cpu='1']=process.argv.slice(2);assert(output,'Pass an evidence output prefix');
const profile=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-startup-browser-'));let browser,socket,timer;
try {
 browser=spawn(process.env.CHROME||'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new','--disable-extensions','--no-first-run','--no-default-browser-check','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:'ignore'});
 const portFile=path.join(profile,'DevToolsActivePort');for(let i=0;!fs.existsSync(portFile)&&i<100;i++)await new Promise(r=>setTimeout(r,100));assert(fs.existsSync(portFile));
 const port=fs.readFileSync(portFile,'utf8').split('\n')[0];const page=await(await fetch(`http://127.0.0.1:${port}/json/new?about:blank`,{method:'PUT'})).json();socket=new WebSocket(page.webSocketDebuggerUrl);await new Promise((r,j)=>{socket.addEventListener('open',r,{once:true});socket.addEventListener('error',j,{once:true});});
 let next=0;const pending=new Map(),errors=[];socket.addEventListener('message',event=>{const m=JSON.parse(event.data);if(m.id){const s=pending.get(m.id);pending.delete(m.id);m.error?s.reject(m.error):s.resolve(m.result);}if(m.method==='Runtime.exceptionThrown')errors.push(m.params.exceptionDetails);});
 const call=(method,params={})=>new Promise((resolve,reject)=>{const id=++next;pending.set(id,{resolve,reject});socket.send(JSON.stringify({id,method,params}));});
 const evaluate=async expression=>{const r=await call('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert(!r.exceptionDetails,JSON.stringify(r.exceptionDetails));return r.result.value;};
 timer=setTimeout(()=>browser.kill('SIGKILL'),45000);await call('Runtime.enable');await call('Page.enable');await call('Network.enable');await call('Network.setCacheDisabled',{cacheDisabled:true});await call('Emulation.setCPUThrottlingRate',{rate:Number(cpu)});
 await call('Page.addScriptToEvaluateOnNewDocument',{source:`window.startup={tasks:[],gaps:[],wasm:[],assigned:0};new PerformanceObserver(list=>{for(const task of list.getEntries())startup.tasks.push({start:task.startTime,duration:task.duration});}).observe({type:'longtask',buffered:true});let last=performance.now();const tick=setInterval(()=>{const now=performance.now();startup.gaps.push(now-last);last=now;},16);for(const method of ['instantiateStreaming','instantiate']){const original=WebAssembly[method];WebAssembly[method]=async function(...args){const start=performance.now();try{return await original.apply(this,args);}finally{startup.wasm.push({method,start,duration:performance.now()-start});}};}let ui;Object.defineProperty(window,'ui',{configurable:true,get:()=>ui,set:value=>{ui=value;startup.assigned=performance.now();setTimeout(()=>clearInterval(tick),1500);}});`});
 await call('Profiler.enable');await call('Profiler.start');await call('Page.navigate',{url:base});for(let i=0;i<200&&!await evaluate('Boolean(window.ui)');i++)await new Promise(r=>setTimeout(r,50));assert(await evaluate('Boolean(window.ui)'),JSON.stringify(errors));await new Promise(r=>setTimeout(r,1600));
 const {profile:trace}=await call('Profiler.stop');fs.writeFileSync(output+'.cpuprofile',JSON.stringify(trace));
 const result=await evaluate(`({...startup,nodes:document.querySelectorAll('[data-tsrust-node]').length,articles:document.querySelectorAll('[data-slot=route-page]').length,headings:document.querySelectorAll('h1').length,wasmBytes:performance.getEntriesByType('resource').filter(r=>r.name.includes('app_bg.wasm')).map(r=>({duration:r.duration,bytes:r.transferSize})),maxGap:Math.max(...startup.gaps),location:location.pathname})`);
 const samples=new Map();for(const id of trace.samples||[])samples.set(id,(samples.get(id)||0)+1);result.top=trace.nodes.map(n=>({name:n.callFrame.functionName,url:n.callFrame.url,samples:samples.get(n.id)||0})).sort((a,b)=>b.samples-a.samples).slice(0,18);result.cpuRate=Number(cpu);result.errors=errors;assert.deepEqual(errors,[]);fs.writeFileSync(output+'.json',JSON.stringify(result,null,2));console.log(JSON.stringify(result));
 await evaluate('window.ui.dispose()');
} finally {
 clearTimeout(timer);socket?.close();if(browser){const exited=new Promise(r=>browser.once('exit',r));browser.kill('SIGTERM');await Promise.race([exited,new Promise(r=>setTimeout(()=>{browser.kill('SIGKILL');r();},2000))]);}fs.rmSync(profile,{recursive:true,force:true,maxRetries:20,retryDelay:100});
}
