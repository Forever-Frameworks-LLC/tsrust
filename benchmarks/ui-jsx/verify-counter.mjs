// Real Chromium input and DOM observations; no Playwright daemon or persistent server.
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawn,spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const [site, mode]=process.argv.slice(2);
assert(site,'Pass the built counter site');
assert(mode===undefined || mode==='--presentation-only','Unknown counter verification mode');
const profile=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-counter-browser-'));
const here=path.dirname(fileURLToPath(import.meta.url));
if(mode!=='--presentation-only') {
const reference=spawnSync(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',profile,path.join(here,'counter-reference.ts')],{encoding:'utf8',timeout:30000});
assert.equal(reference.status,0,reference.stderr||reference.stdout);
const oracle=spawnSync(process.execPath,[path.join(profile,'counter-reference.js')],{encoding:'utf8',timeout:30000});
assert.equal(oracle.status,0,oracle.stderr);
}
const server=http.createServer((request,response)=>{
  const name=request.url==='/'?'index.html':request.url.slice(1);
  if (!['index.html','app.js','app_bg.wasm'].includes(name)) {response.writeHead(404).end();return;}
  response.setHeader('Content-Type',name.endsWith('.wasm')?'application/wasm':name.endsWith('.js')?'text/javascript':'text/html');
  response.end(fs.readFileSync(path.join(site,name)));
});
await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
const browser=spawn(process.env.CHROME||'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new','--disable-extensions','--no-first-run','--no-default-browser-check','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:'ignore'});
let socket;let timeout;
try {
  const portFile=path.join(profile,'DevToolsActivePort');
  for(let i=0;!fs.existsSync(portFile)&&i<100;i++) await new Promise(r=>setTimeout(r,100));
  assert(fs.existsSync(portFile),'Chrome did not start');
  const port=fs.readFileSync(portFile,'utf8').split('\n')[0];
  const page=await (await fetch(`http://127.0.0.1:${port}/json/new?about:blank`,{method:'PUT'})).json();
  socket=new WebSocket(page.webSocketDebuggerUrl);
  await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
  let id=0;const pending=new Map();const errors=[];let requests=0;
  socket.addEventListener('message',event=>{const message=JSON.parse(event.data);if(message.id){const slot=pending.get(message.id);pending.delete(message.id);message.error?slot.reject(message.error):slot.resolve(message.result);}if(message.method==='Network.requestWillBeSent')requests++;if(message.method==='Runtime.exceptionThrown')errors.push(message.params.exceptionDetails);});
  function call(method,params={}) {return new Promise((resolve,reject)=>{const key=++id;pending.set(key,{resolve,reject});socket.send(JSON.stringify({id:key,method,params}));});}
  async function evaluate(expression){const result=await call('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;}
  timeout=setTimeout(()=>{browser.kill('SIGKILL');},45000);
  await call('Runtime.enable');await call('Page.enable');await call('Network.enable');
  const navigation=await call('Page.navigate',{url:`http://127.0.0.1:${server.address().port}/`});
  for(let i=0;i<100;i++){if(await evaluate('Boolean(window.ui)'))break;await new Promise(r=>setTimeout(r,100));}
  if(!await evaluate('Boolean(window.ui)'))console.log(JSON.stringify({navigation,requests,page:await evaluate('({url:location.href,ready:document.readyState,html:document.documentElement.outerHTML,resources:performance.getEntriesByType("resource").map(r=>({name:r.name,duration:r.duration}))})')}));
  assert(await evaluate('Boolean(window.ui)'),'Wasm entry did not mount: '+JSON.stringify(errors));
  const presentation=[];
  for (const width of [640,800,480,640]) {
    await call('Emulation.setDeviceMetricsOverride',{width,height:480,deviceScaleFactor:1,mobile:false});
    const styles=await evaluate('[...document.querySelectorAll("button")].map(e=>{const s=getComputedStyle(e);const r=e.getBoundingClientRect();return {align:s.textAlign,radius:s.borderRadius,fontSize:s.fontSize,height:r.height,width:r.width}})');
    assert(styles.every(s=>s.align==='center'),'Button labels must be centered');
    assert.equal(styles[0].radius,'8px');
    if(presentation.length) assert.deepEqual(styles.map(s=>[s.fontSize,s.height]),presentation[0].styles.map(s=>[s.fontSize,s.height]));
    presentation.push({width,styles});
  }
  console.log(JSON.stringify({case:'counter-browser-presentation',presentation,timed:false}));
  for(const name of ['app.js','app_bg.wasm']) console.log(JSON.stringify({artifact:name,sha256:createHash('sha256').update(fs.readFileSync(path.join(site,name))).digest('hex')}));
  console.log(JSON.stringify({verifier:'verify-counter.mjs',sha256:createHash('sha256').update(fs.readFileSync(fileURLToPath(import.meta.url))).digest('hex')}));
  if(mode==='--presentation-only') {assert.deepEqual(errors,[]); await evaluate('window.ui.dispose()'); process.exitCode=0;}
  else {
  const initialRequests=requests;
  const counts=()=>evaluate('[...document.querySelectorAll("span")].slice(1).map(n=>n.textContent.replace(/^Count: /,""))');
  assert.deepEqual(await counts(),['0','0']);
  await evaluate('window.originalNodes=[...document.querySelectorAll("[data-tsrust-node]")];window.mutations=[];window.observer=new MutationObserver(records=>window.mutations.push(...records.map(r=>r.target.dataset.tsrustNode)));window.observer.observe(document.body,{subtree:true,childList:true,characterData:true})');
  const bounds=await evaluate('[...document.querySelectorAll("button")].map(e=>{const r=e.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2}})');
  async function click(point){await call('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...point});await call('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...point});}
  async function key(key,code,virtual){const text=key==='Enter'?'\r':key===' '?' ':'';await call('Input.dispatchKeyEvent',{type:text?'keyDown':'rawKeyDown',key,code,windowsVirtualKeyCode:virtual,text,unmodifiedText:text});await call('Input.dispatchKeyEvent',{type:'keyUp',key,code,windowsVirtualKeyCode:virtual});}
  await click(bounds[0]);assert.deepEqual(await counts(),['1','0']);
  await click(bounds[1]);assert.deepEqual(await counts(),['1','0']);
  await evaluate('document.querySelectorAll("button")[0].focus()');
  await key('Tab','Tab',9);assert(await evaluate('document.activeElement===document.querySelectorAll("button")[2]'),'Tab must skip disabled controls');
  await key('Enter','Enter',13);assert.deepEqual(await counts(),['1','1']);
  await key(' ','Space',32);assert.deepEqual(await counts(),['1','2']);
  assert(await evaluate('window.originalNodes.every(n=>n.isConnected)'),'Component rerender replaced nodes');
  const mutations=await evaluate('window.mutations');assert.equal(new Set(mutations).size,2,'Only two signal labels should change');
  await evaluate('window.savedButton=document.querySelector("button");window.ui.dispose();window.savedButton.click()');
  assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);
  assert.deepEqual(errors,[]);assert.equal(requests,initialRequests,'Local interaction must not issue requests');
  for(const source of ['counter-reference.ts','../../examples/ui-counter/screen.tsrx','../../examples/ui-counter/main.tsr'])console.log(JSON.stringify({source,sha256:createHash('sha256').update(fs.readFileSync(path.join(here,source))).digest('hex')}));
  console.log(JSON.stringify({case:'paired-counter-browser-input',status:'pass',observed:['0,0','1,0','1,1','1,2'],checks:['pointer','disabled','Tab','Enter','Space','stable nodes','targeted text','dispose','late click'],timed:false}));
  }
} catch(error) { console.error(error);throw error; } finally {
  clearTimeout(timeout);if(socket)socket.close();browser.kill('SIGTERM');
  await Promise.race([new Promise(resolve=>browser.once('exit',resolve)),new Promise(resolve=>setTimeout(()=>{browser.kill('SIGKILL');resolve();},2000))]);
  await new Promise(resolve=>server.close(resolve));fs.rmSync(profile,{recursive:true,force:true,maxRetries:20,retryDelay:100});
}
