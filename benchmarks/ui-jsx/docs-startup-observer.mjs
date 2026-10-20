import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
export async function observeDocsStartup({evaluate,call,errors,site}) {
 const catalog=JSON.parse(fs.readFileSync(new URL('../../docs/target/docs-content/catalog.json',import.meta.url),'utf8'));
 const wait=async expression=>{for(let i=0;i<100;i++){if(await evaluate(expression))return;await new Promise(r=>setTimeout(r,30));}assert.fail(expression);};
 const click=async href=>{await evaluate(`document.querySelector('a[href=${JSON.stringify(href)}]').click()`);await wait(`location.pathname===${JSON.stringify(href)}`);};
 await call('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 assert.equal(await evaluate(`document.querySelectorAll('[data-slot=route-page]').length`),0);
 await evaluate(`window.homeCounter=document.querySelector('#home-counter-preview');homeCounter.querySelector('button').click()`);
 await call('Input.dispatchMouseEvent',{type:'mouseWheel',x:700,y:400,deltaX:0,deltaY:450});await wait(`document.querySelector('#landing').scrollTop>0`);
 await click('/docs');assert.equal(await evaluate(`document.querySelector('[data-slot=route-page] h1').textContent`),'Build with Tsrust');
 await click('/docs/getting-started/installation');await evaluate(`window.savedArticle=document.querySelector('[data-slot=route-page]');savedArticle.scrollTop=90`);
 await evaluate('history.back()');await wait(`location.pathname==='/docs'`);await evaluate('history.forward()');await wait(`location.pathname==='/docs/getting-started/installation'`);assert(await evaluate(`document.querySelector('[data-slot=route-page]')===savedArticle&&savedArticle.scrollTop===90`));
 await evaluate(`Array.from(document.querySelectorAll('nav[aria-label="Documentation sections"] a')).find(node=>node.textContent==='Components').click()`);
 await click('/docs/ui/components/input');await evaluate(`window.savedInput=document.querySelector('#input-states-name');savedInput.value='Retained 🦀';savedInput.dispatchEvent(new Event('input',{bubbles:true}));`);
 await click('/');assert(await evaluate(`document.querySelector('#home-counter-preview')===homeCounter&&homeCounter.textContent.includes('Count: 1')`));
 await click('/docs');await evaluate(`Array.from(document.querySelectorAll('nav[aria-label="Documentation sections"] a')).find(node=>node.textContent==='Components').click()`);await click('/docs/ui/components/input');assert(await evaluate(`document.querySelector('#input-states-name')===savedInput&&savedInput.value==='Retained 🦀'`));
 await evaluate(`(()=>{const input=document.querySelector('input[aria-label="Search documentation"]');input.value='U+0085';input.dispatchEvent(new Event('input',{bubbles:true}));})()`);
 assert.deepEqual(await evaluate(`Array.from(document.querySelectorAll('#sidebar a')).map(a=>a.getAttribute('href'))`),['/docs/standard-library/strings']);
 await evaluate(`(()=>{const input=document.querySelector('input[aria-label="Search documentation"]');input.value='aria-describedby';input.dispatchEvent(new Event('input',{bubbles:true}));})()`);assert(await evaluate(`!!document.querySelector('#sidebar a[href="/docs/ui/components/button"]')`));
 await evaluate(`(()=>{const input=document.querySelector('input[aria-label="Search documentation"]');input.value='';input.dispatchEvent(new Event('input',{bubbles:true}));document.querySelector('button[aria-label="Toggle light and dark theme"]').click();})()`);assert(await evaluate(`document.querySelector('#input-states-name')===savedInput&&savedInput.value==='Retained 🦀'`));
 const base=await evaluate('location.origin');await call('Page.navigate',{url:base+'/docs/ui/components/aspect-ratio'});await wait(`Boolean(window.ui)&&document.querySelector('[data-slot=route-page] h1')?.textContent==='Aspect Ratio'`);assert.equal(await evaluate(`document.querySelector('#landing')===null`),true);
 assert.deepEqual(await evaluate('window.ui.mounts.map(m=>m.error())'),[null]);fs.writeFileSync(path.join(site,'lazy-docs.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));await evaluate('window.ui.dispose()');assert.equal(await evaluate(`document.querySelectorAll('[data-tsrust-node]').length`),0);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'lazy-documentation-startup',status:'pass',pages:catalog.pages.length,checks:['no landing article construction','immediate actual wheel scroll','first route','retained article DOM/scroll','retained field DOM/value','retained home counter','lazy full-text/code/API search','theme retention','direct deep entry','no eager home on deep entry','disposal'],timed:false}));
}
