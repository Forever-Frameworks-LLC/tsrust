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
const profile=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-conditional-failure-browser-'));
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
  assert.equal(await evaluate('document.querySelector("[data-slot=healthy]").textContent'),'10');
  await evaluate(`document.querySelector('[aria-label="Break branch"]').click()`);
  const observed=await evaluate('window.ui.mounts.map(mount=>mount.error())');assert.deepEqual(observed,[expected.error,null]);
  await evaluate(`document.querySelector('[aria-label="Healthy branch"]').click()`);assert.equal(await evaluate('document.querySelector("[data-slot=healthy]").textContent'),'11');
  await evaluate(`document.querySelector('[aria-label="Break branch"]').click()`);assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),observed);
  await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
  for(const file of ['conditional-reference.ts','../../examples/ui-conditional-mounting/src/failures.tsrx'])console.log(JSON.stringify({source:file,sha256:createHash('sha256').update(fs.readFileSync(path.join(here,file))).digest('hex')}));
  console.log(JSON.stringify({case:'browser-ui-conditional-mounting-failures',status:'pass',observed,checks:['preserved lazy branch error','independent healthy mount','stopped failed events','disposal'],timed:false}));

} catch(error) { console.error(error);throw error; } finally {
  clearTimeout(timeout);if(socket)socket.close();browser.kill('SIGTERM');
  await Promise.race([new Promise(resolve=>browser.once('exit',resolve)),new Promise(resolve=>setTimeout(()=>{browser.kill('SIGKILL');resolve();},2000))]);
  await new Promise(resolve=>server.close(resolve));fs.rmSync(profile,{recursive:true,force:true,maxRetries:20,retryDelay:100});
}
