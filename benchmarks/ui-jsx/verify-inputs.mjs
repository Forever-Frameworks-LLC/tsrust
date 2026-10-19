// Untimed browser editable-controls and independent Node-after-tsc oracle. Owns and closes every process.
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawn,spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const [site]=process.argv.slice(2);
assert(site,'Pass the built input site');
const profile=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-input-browser-'));
const here=path.dirname(fileURLToPath(import.meta.url));
function run(command,args){const result=spawnSync(command,args,{encoding:'utf8',timeout:30000});assert.equal(result.status,0,result.stderr||result.stdout||String(result.error));return result.stdout;}
let expected;
try {
 run(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',profile,path.join(here,'input-reference.ts')]);
 expected=JSON.parse(run(process.execPath,[path.join(profile,'input-reference.js')]));
} catch(error) {fs.rmSync(profile,{recursive:true,force:true});throw error;}
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
  await call('Emulation.setDeviceMetricsOverride',{width:640,height:1400,deviceScaleFactor:1,mobile:false});
  await evaluate('new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))');
  const observed=[];
  async function observe(){observed.push(await evaluate(`({name:document.querySelector('[data-slot=result]').textContent.split(' / ')[0],notes:document.querySelector('#notes').value,derived:document.querySelector('[aria-label="Derived name"]').value,locked:document.querySelector('#name').readOnly,edits:Number(document.querySelector('[data-slot=result]').textContent.split(' / ')[2]),uncontrolled:document.querySelector('[aria-label="Uncontrolled"]').value,last:document.querySelector('[data-slot=result]').textContent.split(' / ')[4]})`));}
  async function type(selector,text){await evaluate(`(()=>{const n=document.querySelector(${JSON.stringify(selector)});n.focus();n.select();})()`);await call('Input.insertText',{text});}
  await observe();await type('#name','Grace');await observe();
  await evaluate(`document.querySelector('#name').setSelectionRange(2,2)`);await call('Input.insertText',{text:'é'});await observe();
  assert.deepEqual(await evaluate(`({start:document.querySelector('#name').selectionStart,end:document.querySelector('#name').selectionEnd})`),{start:3,end:3},'controlled echo must not reset the caret');
  await evaluate(`(()=>{const n=document.querySelector('#name');n.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));n.value='Gr日本';n.dispatchEvent(new InputEvent('input',{bubbles:true,isComposing:true}));})()`);
  assert.equal(await evaluate(`document.querySelector('#name').value`),'Gr日本','preedit must survive refresh');
  // Observe application state while the displayed control contains preedit.
  await observe();
  await evaluate(`(()=>{const n=document.querySelector('#name');n.value='日本🙂';n.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true,data:'日本🙂'}));})()`);await observe();
  await evaluate(`document.querySelector('#name').dispatchEvent(new InputEvent('input',{bubbles:true}))`);await observe();
  await type('#notes','Line1\nLine2');await observe();
  await evaluate(`document.querySelectorAll('button')[0].click()`);await observe();await type('#name','ignored');await observe();
  await evaluate(`document.querySelectorAll('button')[1].click()`);await observe();await type('[aria-label="Uncontrolled"]','Changed');await observe();
  assert.deepEqual(observed,expected.observations);
  assert.equal(await evaluate(`document.querySelector('[aria-label="Exact integer"]').value`),'9007199254740993');
  await type('[aria-label="Disabled"]','ignored');assert.equal(await evaluate(`document.querySelector('[aria-label="Disabled"]').value`),'Fixed');
  const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);
  for(const name of ['Name','Notes']){const node=tree.find(node=>node.role?.value==='textbox' && node.name?.value===name);assert(node,`Missing accessible textbox ${name}`);}
  const nameField=tree.find(node=>node.role?.value==='textbox' && node.name?.value==='Name');assert.equal(nameField.description?.value,'Use your full name.');assert(nameField.properties.some(p=>p.name==='readonly' && p.value.value===true));assert(nameField.properties.some(p=>p.name==='required' && p.value.value===true));
  const invalid=tree.find(node=>node.role?.value==='textbox' && node.name?.value==='Invalid');assert(invalid.properties.some(p=>p.name==='invalid' && p.value.value==='true'));
  await evaluate(`document.querySelector('label[for="notes"]').click()`);assert.equal(await evaluate('document.activeElement.id'),'notes');
  const style=await evaluate(`(()=>{const n=document.querySelector('#name');const s=getComputedStyle(n);return {padding:s.paddingLeft,radius:s.borderRadius,border:s.borderWidth};})()`);assert.deepEqual(style,{padding:'10px',radius:'12px',border:'1px'});
  const dark=await evaluate(`(()=>{const s=getComputedStyle(document.querySelector('[aria-label="Dark invalid"]'));return {background:s.backgroundColor,border:s.borderColor,opacity:s.opacity};})()`);
  const rgba=value=>{assert(/^(color\(srgb |rgba?\()/.test(value),value);const channels=value.match(/[+-]?(?:[0-9]*\.)?[0-9]+(?:e[+-]?[0-9]+)?/gi).map(Number);if(value.startsWith('rgb'))for(let i=0;i<3;i++)channels[i]/=255;if(channels.length===3)channels.push(1);return channels;};
  assert.equal(dark.opacity,'0.5');
  for(const [key,wanted] of [['background',[71/255,85/255,105/255,0.8]],['border',[185/255,28/255,28/255,0.5]]]){const actual=rgba(dark[key]);assert.equal(actual.length,4);for(let i=0;i<4;i++)assert(Math.abs(actual[i]-wanted[i])<0.000001,JSON.stringify({key,actual,wanted}));}
  await call('Emulation.setDeviceMetricsOverride',{width:800,height:1400,deviceScaleFactor:1,mobile:false});assert.equal(await evaluate(`getComputedStyle(document.querySelector('#name')).fontSize`),'14px');
  await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
  for(const file of ['input-reference.ts','../../examples/ui-inputs/src/app/page.tsrx','../../examples/ui-inputs/src/main.tsr','../../examples/ui-inputs/tsrust.config.json'])console.log(JSON.stringify({source:file,sha256:createHash('sha256').update(fs.readFileSync(path.join(here,file))).digest('hex')}));
  console.log(JSON.stringify({case:'browser-ui-editable-controls',status:'pass',observed,dark,checks:['controlled/uncontrolled edits','derived and direct state binding','caret preservation','composition commit','readonly and disabled','exact integer','labels and descriptions','required and invalid','Nova spacing and responsive text','nested dark disabled/invalid alpha','disposal'],engines:['wasm','node-tsc'],timed:false}));

} catch(error) { console.error(error);throw error; } finally {
  clearTimeout(timeout);if(socket)socket.close();browser.kill('SIGTERM');
  await Promise.race([new Promise(resolve=>browser.once('exit',resolve)),new Promise(resolve=>setTimeout(()=>{browser.kill('SIGKILL');resolve();},2000))]);
  await new Promise(resolve=>server.close(resolve));fs.rmSync(profile,{recursive:true,force:true,maxRetries:20,retryDelay:100});
}
