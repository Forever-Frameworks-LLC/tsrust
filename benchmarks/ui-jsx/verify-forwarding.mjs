// Untimed browser forwarding and independent Node-after-tsc oracle. Owns and closes every process.
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawn,spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const [site]=process.argv.slice(2);
assert(site,'Pass the built forwarding site');
const profile=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-forwarding-browser-'));
const here=path.dirname(fileURLToPath(import.meta.url));
function run(command,args){const result=spawnSync(command,args,{encoding:'utf8',timeout:30000});assert.equal(result.status,0,result.stderr||String(result.error));return result.stdout;}
let expected;
try {
 run(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',profile,path.join(here,'forwarding-reference.ts')]);
 expected=JSON.parse(run(process.execPath,[path.join(profile,'forwarding-reference.js')]));
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
  const observed=await evaluate(`Array.from(document.querySelectorAll('[data-size]')).map(node=>{const style=getComputedStyle(node),body=getComputedStyle(node.querySelector('[data-slot=body]')),footer=node.querySelector('[data-slot=footer]');return {size:node.dataset.size,padding:[style.paddingTop,style.paddingRight,style.paddingBottom,style.paddingLeft],bodyPadding:[body.paddingTop,body.paddingRight,body.paddingBottom,body.paddingLeft],footerTop:footer?getComputedStyle(footer).paddingTop:null,minHeight:body.minHeight};})`);
  assert.deepEqual(observed,expected);
  // Live DOM metadata proves selectors remain CSS conditions, not mount-time snapshots.
  await evaluate(`document.querySelector('[data-size=sm]').dataset.size='default'`);
  const changed=await evaluate(`(()=>{const node=document.querySelector('[data-size]');return {padding:getComputedStyle(node).paddingTop,body:getComputedStyle(node.querySelector('[data-slot=body]')).paddingLeft};})()`);
  assert.deepEqual(changed,{padding:'16px',body:'16px'});
  await evaluate(`import('/app.js').then(module=>{window.extraMounts=module.launch();})`);
  const isolated=await evaluate(`Array.from(document.querySelectorAll('[data-tsrust-mount]')).map(mount=>getComputedStyle(mount.querySelector('[data-slot=body]')).paddingLeft)`);
  assert.deepEqual(isolated,['16px','12px']);
  await evaluate(`window.extraMounts.forEach(mount=>mount.free());window.extraMounts=[]`);
  assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-mount]").length'),1);
  const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert(!tree.some(node=>node.role?.value==='button'));
  assert(tree.some(node=>node.name?.value==='Small panel'));
  await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
  for(const file of ['forwarding-reference.ts','../../examples/ui-forwarding/screen.tsrx','../../examples/ui-forwarding/main.tsr'])console.log(JSON.stringify({source:file,sha256:createHash('sha256').update(fs.readFileSync(path.join(here,file))).digest('hex')}));
  console.log(JSON.stringify({case:'browser-ui-forwarding',status:'pass',observed,checks:['owned props','data size','descendant predicates','named ancestor groups','own class conditions','live CSS metadata','mount isolation','accessibility','disposal'],engines:['wasm','node-tsc'],timed:false}));
} catch(error) { console.error(error);throw error; } finally {
  clearTimeout(timeout);if(socket)socket.close();browser.kill('SIGTERM');
  await Promise.race([new Promise(resolve=>browser.once('exit',resolve)),new Promise(resolve=>setTimeout(()=>{browser.kill('SIGKILL');resolve();},2000))]);
  await new Promise(resolve=>server.close(resolve));fs.rmSync(profile,{recursive:true,force:true,maxRetries:20,retryDelay:100});
}
