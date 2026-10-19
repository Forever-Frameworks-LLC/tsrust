// Untimed observation of the shared landing and rectangle-visibility contract.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

export async function observeDocsLanding({evaluate,call,errors,site}) {
 const wait=async expression=>{for(let i=0;i<100;i++){if(await evaluate(expression))return;await new Promise(resolve=>setTimeout(resolve,50));}assert.fail(`Timeout: ${expression}`);};
 const click=async href=>{assert(await evaluate(`!!document.querySelector('a[href=${JSON.stringify(href)}]')`),href);await evaluate(`document.querySelector('a[href=${JSON.stringify(href)}]').click()`);};
 await call('Emulation.setDeviceMetricsOverride',{width:1440,height:960,deviceScaleFactor:1,mobile:false});
 assert.equal(await evaluate(`document.querySelector('#landing h1').textContent`),'Write TypeScript.\nShip Native Rust.');
 assert.equal(await evaluate(`document.querySelectorAll('nav[aria-label="Main navigation"] a[href^="https://github.com/Forever-Frameworks-LLC/tsrust"]').length`),2);
 assert(!await evaluate(`document.querySelector('#landing').textContent.includes('2.1MB')||document.body.textContent.includes('curl -fsSL')||document.body.textContent.includes('Discord')`));
 const initialHeader=await evaluate(`document.querySelector('header').getBoundingClientRect().top`);
 for(const id of ['familiar','rust','fabric']){
  await evaluate(`document.getElementById(${JSON.stringify(id)}).scrollIntoView()`);
  await wait(`!!document.querySelector('#${id} [data-slot=chapter-result]')`);
 }
 assert.equal(await evaluate(`document.querySelector('header').getBoundingClientRect().top`),initialHeader);
 assert(await evaluate(`document.querySelector('#rust').textContent.includes('Hello from Rust')`));
 await evaluate(`document.querySelector('#home-counter-preview button').click();Array.from(document.querySelectorAll('#landing button')).find(node=>node.textContent==='Show source').click()`);
 assert(await evaluate(`document.querySelector('#home-counter-source code').textContent.includes('count.set(count.get() + 1)')`));
 await evaluate(`Array.from(document.querySelectorAll('#landing button')).find(node=>node.textContent==='Show preview').click()`);
 assert(await evaluate(`document.querySelector('#home-counter-preview').textContent.includes('Count: 1')`));
 for(const width of [320,480,768,1024,1440]){
  await call('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
  await evaluate(`document.querySelector('#landing').scrollTop=0`);
  assert(await evaluate(`document.documentElement.scrollWidth<=innerWidth&&document.querySelector('#landing').scrollWidth<=document.querySelector('#landing').clientWidth`),`Horizontal overflow at ${width}`);
  assert(await evaluate(`Array.from(document.querySelectorAll('nav[aria-label="Main navigation"] a,nav[aria-label="Main navigation"] button')).every(node=>{const r=node.getBoundingClientRect();return r.left>=0&&r.right<=innerWidth;})`),`Header at ${width}`);
 }
 await call('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 await call('Emulation.setEmulatedMedia',{features:[{name:'prefers-reduced-motion',value:'reduce'}]});
 await evaluate(`document.querySelector('#rust').scrollIntoView()`);assert(await evaluate(`!!document.querySelector('#rust code')`));
 fs.writeFileSync(path.join(site,'landing-light.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 await evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click()`);
 assert.equal(await evaluate(`getComputedStyle(document.querySelector('#site')).backgroundColor`),'rgb(9, 9, 11)');
 await evaluate(`document.querySelector('#landing').scrollTop=0`);
 fs.writeFileSync(path.join(site,'landing-dark.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 await click('/docs');await click('/docs/ui/foundations/state');await click('/docs/ui/foundations/visibility');
 await wait(`!!document.querySelector('#visibility-pane')`);
 await evaluate(`document.querySelector('#visibility-pane').scrollIntoView()`);
 await wait(`document.querySelector('#visibility-state').textContent==='Visible: no; notifications: 1'`);
 await evaluate(`document.querySelector('#visibility-pane').scrollTop=96`);
 await wait(`document.querySelector('#visibility-state').textContent==='Visible: yes; notifications: 2'`);
 await evaluate(`document.querySelector('#visibility-pane').scrollTop=100`);await new Promise(resolve=>setTimeout(resolve,75));
 assert.equal(await evaluate(`document.querySelector('#visibility-state').textContent`),'Visible: yes; notifications: 2');
 await evaluate(`Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Toggle observed view').click()`);
 await wait(`document.querySelector('#visibility-state').textContent==='Visible: no; notifications: 3'`);
 await evaluate(`Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Toggle observed view').click()`);
 await wait(`document.querySelector('#visibility-state').textContent==='Visible: yes; notifications: 4'`);
 await evaluate(`document.querySelector('#visibility-target').style.width='0px'`);
 await wait(`document.querySelector('#visibility-state').textContent==='Visible: no; notifications: 5'`);
 await evaluate(`document.querySelector('#visibility-target').style.width=''`);
 await wait(`document.querySelector('#visibility-state').textContent==='Visible: yes; notifications: 6'`);
 await click('/');await wait(`!!document.querySelector('#home-counter-preview')`);
 assert(await evaluate(`document.querySelector('#home-counter-preview').textContent.includes('Count: 1')`));
 assert.deepEqual(await evaluate(`window.ui.mounts.map(mount=>mount.error())`),[null]);
 await evaluate(`document.querySelector('#landing').scrollTop=500;window.ui.dispose()`);await new Promise(resolve=>setTimeout(resolve,75));
 assert.equal(await evaluate(`document.querySelectorAll('[data-tsrust-node]').length`),0);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'paired-landing-and-visibility',status:'pass',checks:['real code and Rust binding','chapter visibility','sticky header','320–1440px layout','both themes','reduced-motion readability','retained counter','nested clipping','zero area','initial notification and deduplication','hidden restoration','pending observer disposal'],timed:false}));
}
