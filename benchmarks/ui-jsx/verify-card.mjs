// Untimed browser card and independent Node-after-tsc oracle. Owns and closes every process.
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawn,spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const [site]=process.argv.slice(2);
assert(site,'Pass the built card site');
const profile=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-card-browser-'));
const here=path.dirname(fileURLToPath(import.meta.url));
function run(command,args){const result=spawnSync(command,args,{encoding:'utf8',timeout:30000});assert.equal(result.status,0,result.stderr||String(result.error));return result.stdout;}
let expected;
try {
 run(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',profile,path.join(here,'card-reference.ts')]);
 expected=JSON.parse(run(process.execPath,[path.join(profile,'card-reference.js')]));
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
  const observed=await evaluate(`(()=>{const get=(id,slot)=>document.querySelector('#'+id+(slot?' [data-slot='+slot+']':'')),style=(id,slot)=>getComputedStyle(get(id,slot)),card=id=>{const c=style(id),h=style(id,'card-header'),t=style(id,'card-title');return {paddingTop:c.paddingTop,paddingBottom:c.paddingBottom,gap:c.gap,radius:c.borderTopLeftRadius,headerPadding:h.paddingLeft,titleSize:t.fontSize,titleLeading:t.lineHeight,titleWeight:t.fontWeight};},d=card('default-card'),s=card('small-card'),c=card('custom-card');return {default:{...d,footerPadding:style('default-card','card-footer').paddingTop,footerTop:style('default-card','card-footer').borderTopWidth},small:s,custom:{paddingTop:c.paddingTop,paddingBottom:c.paddingBottom,gap:c.gap,radius:c.radius,headerPadding:c.headerPadding,headerBottom:style('custom-card','card-header').paddingBottom,footerPadding:style('custom-card','card-footer').paddingTop},missing:style('missing-variable').paddingTop};})()`);
  assert.deepEqual(observed,expected);
  const geometry=await evaluate(`(()=>{const h=document.querySelector('#default-card [data-slot=card-header]'),t=h.querySelector('[data-slot=card-title]').getBoundingClientRect(),a=h.querySelector('[data-slot=card-action]').getBoundingClientRect(),d=h.querySelector('[data-slot=card-description]').getBoundingClientRect();return {titleX:t.x,actionX:a.x,titleY:t.y,descriptionY:d.y};})()`);
  assert(geometry.actionX>geometry.titleX && geometry.descriptionY>geometry.titleY,JSON.stringify(geometry));
  const clip=await evaluate(`(()=>{const r=document.querySelector('#clip-card').getBoundingClientRect();return {x:r.x,y:r.y};})()`);
  async function click(x,y){await call('Input.dispatchMouseEvent',{type:'mousePressed',x,y,button:'left',clickCount:1});await call('Input.dispatchMouseEvent',{type:'mouseReleased',x,y,button:'left',clickCount:1});}
  const presses=()=>evaluate(`document.querySelector('[data-slot=press-count]').textContent`);
  await click(clip.x+80,clip.y+20);await click(clip.x+1,clip.y+1);assert.equal(await presses(),'Presses: 0');
  await click(clip.x+32,clip.y+20);assert.equal(await presses(),'Presses: 1');
  const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert(tree.some(node=>node.role?.value==='region' && node.name?.value==='Project settings'));
  assert(tree.some(node=>node.role?.value==='heading' && node.name?.value==='Project settings'));
  for(const name of ['More','Save','Clipped action'])assert(tree.some(node=>node.role?.value==='button' && node.name?.value===name),name);
  await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
  for(const file of ['card-reference.ts','../../examples/ui-card/src/screen.tsrx','../../examples/ui-card/src/main.tsr','../../examples/ui-card/src/components/ui/card.tsrx','../../examples/ui-card/tsrust.config.json'])console.log(JSON.stringify({source:file,sha256:createHash('sha256').update(fs.readFileSync(path.join(here,file))).digest('hex')}));
  console.log(JSON.stringify({case:'browser-ui-card',status:'pass',observed,checks:['seven real content parts','default/small spacing','scoped custom spacing','caller overrides','edge/corner styles','header action layout','typography','rounded pointer clipping','missing spacing fallback','accessible heading/region/buttons','disposal'],engines:['wasm','node-tsc'],timed:false}));
} catch(error) { console.error(error);throw error; } finally {
  clearTimeout(timeout);if(socket)socket.close();browser.kill('SIGTERM');
  await Promise.race([new Promise(resolve=>browser.once('exit',resolve)),new Promise(resolve=>setTimeout(()=>{browser.kill('SIGKILL');resolve();},2000))]);
  await new Promise(resolve=>server.close(resolve));fs.rmSync(profile,{recursive:true,force:true,maxRetries:20,retryDelay:100});
}
