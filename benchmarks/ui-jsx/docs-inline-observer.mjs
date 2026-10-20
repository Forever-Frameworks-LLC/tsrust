// Actual DOM, system clipboard and lifecycle assertions; no timing claims.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
export async function observeDocsInline({evaluate,call,errors,site}) {
 const wait=async expression=>{for(let i=0;i<100;i++){if(await evaluate(expression))return;await new Promise(resolve=>setTimeout(resolve,50));}assert.fail(`Timeout: ${expression}`);};
 const click=async href=>{assert(await evaluate(`!!document.querySelector('a[href=${JSON.stringify(href)}]')`),href);await evaluate(`document.querySelector('a[href=${JSON.stringify(href)}]').click()`);};
 await call('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 assert(await evaluate(`document.querySelector('#landing h1').textContent.includes('Ship Native Rust.')`));
 assert(!await evaluate(`document.body.textContent.includes('Search page text, APIs and code')`));
 fs.writeFileSync(path.join(site,'landing.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 await evaluate(`document.querySelector('#home-workspace-preview button').click();Array.from(document.querySelectorAll('#landing button')).find(node=>node.textContent==='View source').click()`);assert(await evaluate(`document.querySelector('#home-workspace-source code').textContent.includes("saved.set('Saved '")`));await evaluate(`Array.from(document.querySelectorAll('#landing button')).find(node=>node.textContent==='Preview').click()`);assert(await evaluate(`document.querySelector('#home-workspace-preview').textContent.includes('Saved Fieldwork')`));
 await click('/docs');assert.equal(await evaluate(`document.querySelectorAll('#sidebar a').length`),3,'Only the current section appears');
 assert.equal(await evaluate(`document.querySelectorAll('nav[aria-label="Documentation sections"] a').length`),6);
 assert(!await evaluate(`Array.from(document.querySelectorAll('#sidebar button')).some(node=>['Back','Next'].includes(node.textContent))`));
 await click('/docs/ui/foundations/state');await click('/docs/ui/foundations/documentation');
 const article=`document.querySelector('[data-slot=route-page]')`;
 assert(await evaluate(`!!${article}.querySelector('li strong')`));
 assert(await evaluate(`${article}.querySelector('p strong').textContent==='strong emphasis'`));assert(await evaluate(`${article}.querySelector('p em').textContent==='emphasis'`));assert(await evaluate(`${article}.querySelector('p del').textContent==='deleted text'`));
 assert(await evaluate(`Array.from(${article}.querySelectorAll('p a')).some(node=>node.textContent==='state and ownership')`));
 assert(await evaluate(`getComputedStyle(${article}.querySelector('p strong')).fontWeight==='700'`));assert(await evaluate(`getComputedStyle(${article}.querySelector('p em')).fontStyle==='italic'`));
 const expected="// Unicode and whitespace stay intact when copied.\nconst greeting = 'Hello, café 文';\nconsole.log(greeting.trim());\n";
 assert.equal(await evaluate(`${article}.querySelector('code [data-slot=keyword]').textContent`),'const');
 await evaluate(`window.code=document.querySelector('[data-slot=route-page] code:has([data-slot=keyword])');window.copyButton=document.querySelector('[data-slot=route-page] [data-copy-state]')`);
 assert.equal(await evaluate('code.textContent'),expected);
 assert(await evaluate(`getComputedStyle(code.querySelector('[data-slot=keyword]')).color!==getComputedStyle(code.querySelector('[data-slot=string]')).color`));
 await evaluate(`(()=>{const selection=getSelection(),range=document.createRange();range.selectNodeContents(code);selection.removeAllRanges();selection.addRange(range);})()`);assert.equal(await evaluate('getSelection().getRangeAt(0).toString()'),expected);assert.equal(await evaluate('getSelection().toString()'),expected.slice(0,-1),'Browser visual selection omits the final empty line; Copy must preserve it');
 // Exercise a real activation and the actual platform clipboard, with browser permission scoped to this isolated profile.
 const origin=await evaluate('location.origin');await call('Browser.grantPermissions',{origin,permissions:['clipboardReadWrite','clipboardSanitizedWrite']});await call('Page.bringToFront');await call('Emulation.setFocusEmulationEnabled',{enabled:true});
 await evaluate(`(async()=>{window.restoreWrite=navigator.clipboard.writeText.bind(navigator.clipboard);const items=await navigator.clipboard.read();window.originalClipboard=await Promise.all(items.filter(item=>item.types.length).map(async item=>new ClipboardItem(Object.fromEntries(await Promise.all(item.types.map(async type=>[type,await item.getType(type)]))))));})()`);
 try {
 await evaluate('copyButton.scrollIntoView()');let box=await evaluate('(()=>{const r=copyButton.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2};})()');
 await call('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...box});await call('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...box});await wait(`copyButton.dataset.copyState==='copied'`);assert.equal(await evaluate('navigator.clipboard.readText()'),expected);
 // A controlled rejected request must show an error, preserve the root and allow retry.
 await evaluate(`window.realWrite=navigator.clipboard.writeText.bind(navigator.clipboard);navigator.clipboard.writeText=()=>Promise.reject(new Error('Clipboard permission denied'));copyButton.click()`);await wait(`copyButton.dataset.copyState==='error'`);assert(await evaluate(`copyButton.textContent.includes('Try again')&&copyButton.title.includes('permission denied')`));assert.deepEqual(await evaluate('window.ui.mounts.map(m=>m.error())'),[null]);
 // Pending completion cannot report success early and cannot retain a removed mount.
 await evaluate(`navigator.clipboard.writeText=()=>new Promise(resolve=>window.finishCopy=resolve);copyButton.click()`);assert.equal(await evaluate('copyButton.dataset.copyState'),'pending');assert(await evaluate('copyButton.disabled'));await evaluate('finishCopy()');await wait(`copyButton.dataset.copyState==='copied'`);
 } finally {
  await evaluate(`(async()=>{navigator.clipboard.writeText=restoreWrite;if(originalClipboard.length)await navigator.clipboard.write(originalClipboard);else await restoreWrite('');delete window.originalClipboard;})()`);
 }
 await evaluate(`navigator.clipboard.writeText=realWrite;document.querySelector('button[aria-label="Toggle light and dark theme"]').click()`);assert(await evaluate(`getComputedStyle(code.querySelector('[data-slot=keyword]')).color!==getComputedStyle(code.querySelector('[data-slot=string]')).color`));
 fs.writeFileSync(path.join(site,'docs.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 // Search spans sections even though the default index shows one category.
 await evaluate(`(()=>{const search=document.querySelector('input[aria-label="Search documentation"]');search.value='U+0085';search.dispatchEvent(new Event('input',{bubbles:true}));})()`);assert.equal(await evaluate(`document.querySelectorAll('#sidebar a').length`),1);await click('/docs/standard-library/strings');assert(!await evaluate(`document.querySelector('[data-slot=route-page]').textContent.includes('Benchmark evidence')`));
 await evaluate(`(()=>{const search=document.querySelector('input[aria-label="Search documentation"]');search.value='';search.dispatchEvent(new Event('input',{bubbles:true}));})()`);
 await click('/');await call('Emulation.setDeviceMetricsOverride',{width:480,height:700,deviceScaleFactor:1,mobile:false});assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));assert(await evaluate(`document.querySelector('#landing h1').getBoundingClientRect().right<=innerWidth`));
 await click('/docs/ui/foundations/state');await click('/docs/ui/foundations/documentation');assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));
 await evaluate(`window.copyButton=document.querySelector('[data-slot=route-page] [data-copy-state]');navigator.clipboard.writeText=()=>new Promise(resolve=>window.finishCopy=resolve);copyButton.click();window.ui.dispose();finishCopy()`);await new Promise(resolve=>setTimeout(resolve,50));assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
 await evaluate(`dispatchEvent(new PageTransitionEvent('pageshow',{persisted:true}));window.copyButton=document.querySelector('[data-slot=route-page] [data-copy-state]');navigator.clipboard.writeText=()=>{window.ui.dispose();return Promise.resolve();};copyButton.click()`);await new Promise(resolve=>setTimeout(resolve,50));assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'documentation-inline-and-code',status:'pass',checks:['section navigation','inline semantics and wrapping','compiler-token colors','exact platform clipboard','selection across tokens','permission failure and retry','pending completion and disposal','both modes','narrow layout','no developer placeholder copy'],timed:false}));
}
