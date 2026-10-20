// Untimed paired application witness. This driver owns and reaps its browser/server.
import fs from 'node:fs';
import http from 'node:http';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {spawn,spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
const [site,scenario,liveURL]=process.argv.slice(2);assert(site,'Pass the built ui-application site');
const here=path.dirname(fileURLToPath(import.meta.url));
const profile=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-application-browser-'));
let browser,socket,timer,server;
try {
 let expected;
 if(!['docs-example','docs-navigation','docs-inline','docs-components','docs-startup','docs-experience','landing-reference'].includes(scenario)){
 const referenceName=scenario==='docs-reactive-semantics'?'reactive-semantics-reference':scenario==='docs-lazy-children'?'lazy-children-reference':scenario==='docs-vectors'?'vector-reference':scenario==='docs-badge'?'badge-reference':['reactive-props','docs-reactive-props'].includes(scenario)?'reactive-props-reference':scenario==='docs-alert'?'alert-reference':scenario==='docs-aspect'?'aspect-ratio-reference':scenario==='docs-landing'?'visibility-reference':'application-reference';
 const compiler=spawnSync(process.execPath,[path.join(here,'../runtime/node_modules/typescript/bin/tsc'),'--target','es2022','--lib','es2022','--skipLibCheck','--outDir',profile,path.join(here,referenceName+'.ts')],{encoding:'utf8',timeout:30000});assert.equal(compiler.status,0,compiler.stderr||compiler.stdout);
 const reference=spawnSync(process.execPath,[path.join(profile,referenceName+'.js')],{encoding:'utf8',timeout:30000});assert.equal(reference.status,0,reference.stderr);expected=JSON.parse(reference.stdout);
 }
 if(!liveURL)server=http.createServer((request,response)=>{let name=request.url==='/'?'index.html':request.url.slice(1);if(!['index.html','app.js','app_bg.wasm'].includes(name)){if(['docs-navigation','docs-inline','docs-landing','landing-reference'].includes(scenario)&&request.headers.accept?.includes('text/html'))name='index.html';else{response.writeHead(404).end();return;}}response.setHeader('Content-Type',name.endsWith('.wasm')?'application/wasm':name.endsWith('.js')?'text/javascript':'text/html');response.end(fs.readFileSync(path.join(site,name)));});
 if(!liveURL)await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 browser=spawn(process.env.CHROME||'/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',['--headless=new','--disable-extensions','--no-first-run','--no-default-browser-check','--remote-debugging-port=0',`--user-data-dir=${profile}`,'about:blank'],{stdio:'ignore'});
 const portFile=path.join(profile,'DevToolsActivePort');for(let i=0;!fs.existsSync(portFile)&&i<100;i++)await new Promise(resolve=>setTimeout(resolve,100));assert(fs.existsSync(portFile),'Chrome did not start');
 const port=fs.readFileSync(portFile,'utf8').split('\n')[0];const page=await(await fetch(`http://127.0.0.1:${port}/json/new?about:blank`,{method:'PUT'})).json();socket=new WebSocket(page.webSocketDebuggerUrl);await new Promise((resolve,reject)=>{socket.addEventListener('open',resolve,{once:true});socket.addEventListener('error',reject,{once:true});});
 let sequence=0;const pending=new Map(),errors=[];
 socket.addEventListener('message',event=>{const message=JSON.parse(event.data);if(message.id){const slot=pending.get(message.id);pending.delete(message.id);message.error?slot.reject(message.error):slot.resolve(message.result);}if(message.method==='Runtime.exceptionThrown')errors.push(message.params.exceptionDetails);});
 function call(method,params={}){return new Promise((resolve,reject)=>{const id=++sequence;pending.set(id,{resolve,reject});socket.send(JSON.stringify({id,method,params}));});}
 async function evaluate(expression){const result=await call('Runtime.evaluate',{expression,returnByValue:true,awaitPromise:true});assert(!result.exceptionDetails,JSON.stringify(result.exceptionDetails));return result.result.value;}
 timer=setTimeout(()=>browser.kill('SIGKILL'),45000);await call('Runtime.enable');await call('Page.enable');if(scenario==='docs-landing'){const {observerTracking}=await import('./docs-landing-observer.mjs');await call('Page.addScriptToEvaluateOnNewDocument',{source:observerTracking});}await call('Page.navigate',{url:liveURL||`http://127.0.0.1:${server.address().port}/`});
 const ready=scenario==='landing-reference'?"!!document.querySelector('#hero-title')":'Boolean(window.ui)';for(let i=0;i<100&&!await evaluate(ready);i++)await new Promise(resolve=>setTimeout(resolve,100));assert(await evaluate(ready),JSON.stringify(errors));
 if(scenario==='landing-reference'){
  const {observeLandingReference}=await import('./landing-reference-observer.mjs');await observeLandingReference({evaluate,call,errors,site});
 } else if(scenario==='docs-landing'){
  const {observeDocsLanding}=await import('./docs-landing-observer.mjs');await observeDocsLanding({evaluate,call,errors,site,expected});
 } else if(['reactive-props','docs-reactive-props'].includes(scenario)){
  const {observeReactiveProps}=await import('./reactive-props-observer.mjs');await observeReactiveProps({evaluate,call,errors,expected});
 } else if(scenario==='docs-reactive-semantics'){
  const {observeReactiveSemantics}=await import('./reactive-semantics-observer.mjs');await observeReactiveSemantics({evaluate,call,errors,expected,site});
 } else if(scenario==='docs-lazy-children'){
  const {observeLazyChildren}=await import('./lazy-children-observer.mjs');await observeLazyChildren({evaluate,call,errors,expected,site});
} else if(scenario==='docs-vectors'){
  const {observeVectors}=await import('./vector-observer.mjs');await observeVectors({evaluate,call,errors,expected,site});
 } else if(scenario==='docs-badge'){
  const {observeBadge}=await import('./badge-observer.mjs');await observeBadge({evaluate,call,errors,expected,site});
 } else if(scenario==='docs-alert'){
  const {observeDocsAlert}=await import('./docs-alert-observer.mjs');await observeDocsAlert({evaluate,call,errors,site,expected});
 } else if(scenario==='docs-experience'){
  const {observeDocsExperience}=await import('./docs-experience-observer.mjs');await observeDocsExperience({evaluate,call,errors,site});
 } else if(scenario==='docs-startup'){
  const {observeDocsStartup}=await import('./docs-startup-observer.mjs');await observeDocsStartup({evaluate,call,errors,site});
 } else if(scenario==='docs-aspect'){
  const {observeAspectRatio}=await import('./aspect-ratio-observer.mjs');await observeAspectRatio({evaluate,call,errors,site,expected});
 } else if(scenario==='docs-components'){
  const {observeDocsComponents}=await import('./docs-components-observer.mjs');await observeDocsComponents({evaluate,call,errors,site});
 } else if(scenario==='docs-inline'){
  const {observeDocsInline}=await import('./docs-inline-observer.mjs');await observeDocsInline({evaluate,call,errors,site});
 } else if(scenario==='docs-navigation'){
  const {observeDocsNavigation}=await import('./docs-navigation-observer.mjs');await observeDocsNavigation({evaluate,call,errors,site});
 } else if(scenario==='docs-example'){
  const {observeDocsExample}=await import('./docs-example-observer.mjs');await observeDocsExample({evaluate,call,errors,site});
 } else if(scenario==='docs') {
  const {observeDocs}=await import('./docs-observer.mjs');await observeDocs({evaluate,call,errors,site,expected});
 } else {
 const observed=[];
 async function snapshot(index){const value=await evaluate(`({count:document.querySelector('#count')?.textContent??null,draft:document.querySelector('#draft')?.value??null,mode:document.querySelector('#mode').textContent,source:!!document.querySelector('#source'),background:getComputedStyle(document.querySelector('#application')).backgroundColor})`);observed.push(value);const wanted={...expected.steps[index]};if(wanted.source){wanted.count=null;wanted.draft=null;}assert.deepEqual(value,wanted);assert.equal(await evaluate(`getComputedStyle(document.querySelector('#nested')).backgroundColor`),expected.nested);}
 await snapshot(0);
 await evaluate(`window.savedDraft=document.querySelector('#draft');window.savedTheme=document.querySelector('#theme');document.querySelector('#increment').click();savedDraft.focus();savedDraft.value='Edited 🦀';savedDraft.dispatchEvent(new Event('input',{bubbles:true}));document.querySelector('#theme').click()`);
 await snapshot(1);assert(await evaluate(`document.activeElement===savedDraft&&document.querySelector('#draft')===savedDraft&&document.querySelector('#theme')===savedTheme`),'Theme change must preserve input DOM/focus');
 await evaluate(`document.querySelector('#show-code').click()`);await snapshot(2);assert.equal(await evaluate(`document.querySelector('#source').readOnly`),true);
 const hiddenTree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert(!hiddenTree.some(node=>node.name?.value==='Increment'),'Hidden controls must leave accessibility');
 await evaluate(`document.querySelector('#theme').click()`);await snapshot(3);await evaluate(`document.querySelector('#show-code').click()`);await snapshot(4);
 assert.equal(await evaluate(`document.querySelector('#count').textContent`),'Count: 1');assert.equal(await evaluate(`document.querySelector('#draft').value`),'Edited 🦀');
 await call('Emulation.setDeviceMetricsOverride',{width:480,height:640,deviceScaleFactor:2,mobile:false});assert(await evaluate(`document.querySelector('#theme').getBoundingClientRect().right<=innerWidth`),'Header control must fit a narrow viewport');
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
 for(const file of ['application-reference.ts','../../examples/ui-application-state/src/app/page.tsrx','../../examples/ui-application-state/src/main.tsr','../../examples/ui-application-state/tsrust.config.json'])console.log(JSON.stringify({source:file,sha256:createHash('sha256').update(fs.readFileSync(path.join(here,file))).digest('hex')}));
 console.log(JSON.stringify({case:'browser-application',status:'pass',observed,checks:['live modes','independent theme scope','retained field DOM/focus','hidden state preservation','hidden accessibility','responsive header','readonly source','disposal'],engines:['wasm','node-tsc'],timed:false}));
 }
} finally {
 clearTimeout(timer);socket?.close();
 if(browser){const exited=new Promise(resolve=>browser.once('exit',resolve));browser.kill('SIGTERM');await Promise.race([exited,new Promise(resolve=>setTimeout(()=>{browser.kill('SIGKILL');resolve();},2000))]);}
 if(server)await new Promise(resolve=>server.close(resolve));fs.rmSync(profile,{recursive:true,force:true,maxRetries:20,retryDelay:100});
}
