import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
export async function observeAspectRatio({evaluate,call,errors,site,expected}) {
 const exact=fs.readFileSync(new URL('../../docs/src/examples/aspect-ratio-layout.tsrx',import.meta.url),'utf8');
 await call('Emulation.setDeviceMetricsOverride',{width:1440,height:1000,deviceScaleFactor:1,mobile:false});
 await evaluate(`document.querySelector('a[href="/docs"]').click();Array.from(document.querySelectorAll('nav[aria-label="Documentation sections"] a')).find(node=>node.textContent==='Components').click();document.querySelector('#sidebar a[href="/docs/ui/components/aspect-ratio"]').click()`);
 assert.equal(await evaluate(`document.querySelector('[data-slot=route-page] h1').textContent`),'Aspect Ratio');
 for(const box of expected.cases){
  const actual=await evaluate(`(()=>{const node=document.querySelector('#aspect-${box.name}-example');node.style.width=${JSON.stringify(box.width+'px')};const r=node.getBoundingClientRect();return {width:r.width,height:r.height,focus:node.hasAttribute('tabindex'),role:node.getAttribute('role')};})()`);
  assert(Math.abs(actual.width-box.width)<.02,JSON.stringify({box,actual}));assert(Math.abs(actual.height-box.height)<.02,JSON.stringify({box,actual}));assert.equal(actual.focus,false);assert.equal(actual.role,null);
  await evaluate(`document.querySelector('#aspect-${box.name}-example').style.removeProperty('width')`);
 }
 const card='#component-aspect-ratio-aspect-ratio-layout-preview';
 await evaluate(`Array.from(document.querySelectorAll('[data-slot=route-page] button')).find(node=>node.textContent==='Show source').click()`);
 assert.equal(await evaluate(`document.querySelector('#component-aspect-ratio-aspect-ratio-layout-source code').textContent`),exact);
 await evaluate(`Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async value=>{window.copiedSource=value;}}});document.querySelector('#component-aspect-ratio-aspect-ratio-layout-source button').click()`);
 for(let i=0;i<100&&!await evaluate(`window.copiedSource===${JSON.stringify(exact)}`);i++)await new Promise(resolve=>setTimeout(resolve,30));
 assert.equal(await evaluate('window.copiedSource'),exact);
 await evaluate(`Array.from(document.querySelectorAll('[data-slot=route-page] button')).find(node=>node.textContent==='Show preview').click()`);
 for(const width of [480,768,1440]){
  await call('Emulation.setDeviceMetricsOverride',{width,height:1000,deviceScaleFactor:1,mobile:false});
  assert(await evaluate(`document.documentElement.scrollWidth<=innerWidth&&document.querySelector(${JSON.stringify(card)}).scrollWidth<=document.querySelector(${JSON.stringify(card)}).clientWidth+1`),`Overflow at ${width}`);
 }
 await evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click();document.querySelector('#aspect-square-example').scrollIntoView({block:'center'})`);
 assert.deepEqual(await evaluate(`window.ui.mounts.map(mount=>mount.error())`),[null]);
 fs.writeFileSync(path.join(site,'aspect-ratio.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 await evaluate('window.ui.dispose()');assert.equal(await evaluate(`document.querySelectorAll('[data-tsrust-node]').length`),0);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'paired-aspect-ratio',status:'pass',cases:expected.cases.length,checks:['independent Node-after-tsc geometry','preferred ratios','caller override','explicit/min/max heights','fractional width','source preview','480–1440px layout','theme','no extra role/focus','disposal'],timed:false}));
}
