// Untimed conditional mounting and independent Node-after-tsc oracle. Owns every process.
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawn,spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const [site]=process.argv.slice(2);
assert(site,'Pass the built conditional-mounting site');
const profile=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-conditional-browser-'));
const here=path.dirname(fileURLToPath(import.meta.url));
function run(command,args){const result=spawnSync(command,args,{encoding:'utf8',timeout:30000});assert.equal(result.status,0,result.stderr||result.stdout||String(result.error));return result.stdout;}
let expected;
try {
 run(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',profile,path.join(here,'conditional-reference.ts')]);
 expected=JSON.parse(run(process.execPath,[path.join(profile,'conditional-reference.js')]));
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
  await evaluate(`document.querySelector('h1').dispatchEvent(new Event('input',{bubbles:true}))`);assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);
  const observed=[];const counterIds=[];
  const press=label=>evaluate(`document.querySelector('[aria-label="${label}"]').click()`);
  async function observe(label){observed.push(await evaluate(`({label:${JSON.stringify(label)},counter:document.querySelector('[data-slot=counter]')?.textContent??null,status:document.querySelector('[data-slot=status]').textContent,row:Array.from(document.querySelector('[data-slot=row]').children,node=>node.tagName)})`));}
  async function counterIdentity(){const id=await evaluate(`document.querySelector('[data-slot=counter]').getAttribute('data-tsrust-node')`);assert(!counterIds.includes(id),'retired branch identity reused');counterIds.push(id);}
  await evaluate(`window.stable=document.querySelector('#stable');window.pulse=document.querySelector('[aria-label="Pulse"]');window.oldCounter=document.querySelector('[data-slot=counter]');window.stable.focus();window.stable.setSelectionRange(1,3)`);
  assert.equal(await evaluate(`document.querySelector('[data-slot=caption]').children.length`),0,'absent scalar child creates no element or layout item');
  const stableId=await evaluate(`window.stable.getAttribute('data-tsrust-node')`);await counterIdentity();await observe('initial');
  await evaluate(`document.querySelector('[data-slot=counter]').click()`);await observe('increment');await press('Pulse');await observe('pulse');assert.equal(await evaluate(`document.querySelector('[data-slot=caption]').textContent`),'Pulse active');
  await evaluate(`window.stable.value='editing';window.stable.setSelectionRange(2,4);window.stable.dispatchEvent(new InputEvent('input',{data:'editing',inputType:'insertText'}))`);await observe('typing');
  await press('Toggle counter');await observe('hidden');
  assert(await evaluate(`document.activeElement===window.stable&&document.querySelector('#stable')===window.stable&&document.querySelector('[aria-label="Pulse"]')===window.pulse&&window.stable.selectionStart===2&&window.stable.selectionEnd===4`),'removal preserves sibling focus, selection and identity');
  await evaluate(`window.stable.dispatchEvent(new CompositionEvent('compositionstart',{bubbles:true}));window.stable.value='ed漢ing';window.stable.setSelectionRange(3,3);window.stable.dispatchEvent(new InputEvent('input',{bubbles:true,isComposing:true,data:'漢'}))`);
  await press('Toggle counter');await counterIdentity();await observe('shown during IME');
  assert(await evaluate(`document.activeElement===window.stable&&window.stable.value==='ed漢ing'&&window.stable.selectionStart===3`),'mounting preserves pending composition');
  await press('Switch counter');await counterIdentity();await observe('switched during IME');
  assert(await evaluate(`document.activeElement===window.stable&&window.stable.value==='ed漢ing'&&window.stable.selectionStart===3`),'switching preserves pending composition');
  assert.equal(await evaluate(`getComputedStyle(document.querySelector('[data-slot=counter]')).backgroundColor`),'rgb(37, 99, 235)','new branch receives project theme and current stylesheet');
  await evaluate(`window.stable.dispatchEvent(new CompositionEvent('compositionend',{bubbles:true,data:'漢'}))`);await observe('committed');
  await evaluate(`window.retired=document.querySelector('[data-slot=counter]');window.retired.click()`);await observe('second increment');
  await press('Switch counter');await counterIdentity();await evaluate('window.retired.click();window.oldCounter.click()');await observe('first restored');
  await press('Toggle counter');await press('Pulse');await observe('hidden pulse');await press('Toggle counter');await counterIdentity();await observe('remounted');
  assert.deepEqual(observed,expected.observations);assert.equal(await evaluate(`window.stable.getAttribute('data-tsrust-node')`),stableId);
  const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert(tree.some(node=>node.role?.value==='button'&&node.name?.value==='First: 1'));assert(tree.some(node=>node.role?.value==='textbox'&&node.name?.value==='Stable name'));
  await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
  for(const file of ['conditional-reference.ts','../../examples/ui-conditional-mounting/src/app/page.tsrx','../../examples/ui-conditional-mounting/src/main.tsr','../../examples/ui-conditional-mounting/tsrust.config.json'])console.log(JSON.stringify({source:file,sha256:createHash('sha256').update(fs.readFileSync(path.join(here,file))).digest('hex')}));
  console.log(JSON.stringify({case:'browser-ui-conditional-mounting',status:'pass',observed,counterIds,checks:['lazy nested branches','state persistence and remount reset','no layout placeholders','stable input focus and selection','retained IME composition','retired control events','no identity reuse','project theme on new nodes','accessibility','disposal'],engines:['wasm','node-tsc'],timed:false}));

} catch(error) { console.error(error);throw error; } finally {
  clearTimeout(timeout);if(socket)socket.close();browser.kill('SIGTERM');
  await Promise.race([new Promise(resolve=>browser.once('exit',resolve)),new Promise(resolve=>setTimeout(()=>{browser.kill('SIGKILL');resolve();},2000))]);
  await new Promise(resolve=>server.close(resolve));fs.rmSync(profile,{recursive:true,force:true,maxRetries:20,retryDelay:100});
}
