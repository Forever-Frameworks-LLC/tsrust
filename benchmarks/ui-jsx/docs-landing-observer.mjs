// Untimed observation of the shared landing and rectangle-visibility contract.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

export const observerTracking = `
window.trackedObservers=[];
for(const name of ['IntersectionObserver','ResizeObserver']){
 const Native=window[name];
 window[name]=class extends Native {
  constructor(callback,options){super(callback,options);this.targets=new Set();window.trackedObservers.push(this);}
  observe(target,options){super.observe(target,options);this.targets.add(target);}
  unobserve(target){super.unobserve(target);this.targets.delete(target);}
  disconnect(){super.disconnect();this.targets.clear();}
 };
}`;

export async function observeDocsLanding({evaluate,call,errors,site,expected}) {
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
 await evaluate(`document.querySelector('#home-workspace-preview button').click();Array.from(document.querySelectorAll('#landing button')).find(node=>node.textContent==='View source').click()`);
 assert(await evaluate(`document.querySelector('#home-workspace-source code').textContent.includes("saved.set('Saved '")`));
 await evaluate(`Array.from(document.querySelectorAll('#landing button')).find(node=>node.textContent==='Preview').click()`);
 assert(await evaluate(`document.querySelector('#home-workspace-preview').textContent.includes('Saved Fieldwork')`));
 for(const width of [320,480,768,1024,1440]){
  await call('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
  await evaluate(`document.querySelector('#landing').scrollTop=0`);
  const fits=await evaluate(`document.documentElement.scrollWidth<=innerWidth&&document.querySelector('#landing').scrollWidth<=document.querySelector('#landing').clientWidth`);
  if(!fits){console.log(JSON.stringify(await evaluate(`({width:innerWidth,document:document.documentElement.scrollWidth,landing:{width:document.querySelector('#landing').clientWidth,scroll:document.querySelector('#landing').scrollWidth},overflow:[...document.querySelectorAll('#landing .ui,header .ui')].map(node=>({tag:node.tagName,id:node.id,classes:node.className,text:node.textContent.slice(0,70),left:node.getBoundingClientRect().left,right:node.getBoundingClientRect().right,width:node.getBoundingClientRect().width})).filter(node=>node.width&&node.right>innerWidth+1).slice(0,20)})`)));fs.writeFileSync(path.join(site,`overflow-${width}.png`),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));}
  assert(fits,`Horizontal overflow at ${width}`);
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
 await wait(`document.querySelector('#visibility-state').textContent===${JSON.stringify(expected.steps[0].label)}`);
 await evaluate(`document.querySelector('#visibility-pane').scrollTop=96`);
 await wait(`document.querySelector('#visibility-state').textContent===${JSON.stringify(expected.steps[1].label)}`);
 await evaluate(`document.querySelector('#visibility-pane').scrollTop=100`);await new Promise(resolve=>setTimeout(resolve,75));
 assert.equal(await evaluate(`document.querySelector('#visibility-state').textContent`),expected.steps[2].label);
 await evaluate(`Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Toggle observed view').click()`);
 await wait(`document.querySelector('#visibility-state').textContent===${JSON.stringify(expected.steps[3].label)}`);
 await evaluate(`Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Toggle observed view').click()`);
 await wait(`document.querySelector('#visibility-state').textContent===${JSON.stringify(expected.steps[4].label)}`);
 await evaluate(`document.querySelector('#visibility-target').style.cssText='width:0px;height:0px;padding:0;border:0'`);
 await wait(`document.querySelector('#visibility-state').textContent===${JSON.stringify(expected.steps[5].label)}`);
 await evaluate(`document.querySelector('#visibility-target').style.cssText=''`);
 await wait(`document.querySelector('#visibility-state').textContent===${JSON.stringify(expected.steps[6].label)}`);
 await evaluate(`window.retiredVisibilityTarget=document.querySelector('#visibility-target');Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Remove observed view').click()`);
 await wait(`!document.querySelector('#visibility-target')`);
 assert(await evaluate(`trackedObservers.every(observer=>!observer.targets.has(retiredVisibilityTarget))`),'Removed views must release both observer registrations');
 assert.equal(await evaluate(`document.querySelector('#visibility-state').textContent`),expected.steps[6].label,'Removal must not invoke a disposed handler');
 await evaluate(`Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Mount observed view').click()`);
 await wait(`document.querySelector('#visibility-state').textContent===${JSON.stringify(expected.steps[7].label)}`);
 assert(await evaluate(`document.querySelector('#visibility-target')!==retiredVisibilityTarget`));
 await click('/');await wait(`!!document.querySelector('#home-workspace-preview')`);
 assert(await evaluate(`document.querySelector('#home-workspace-preview').textContent.includes('Saved Fieldwork')`));
 assert.deepEqual(await evaluate(`window.ui.mounts.map(mount=>mount.error())`),[null]);
 await evaluate(`document.querySelector('#landing').scrollTop=500;window.ui.dispose()`);await new Promise(resolve=>setTimeout(resolve,75));
 assert.equal(await evaluate(`document.querySelectorAll('[data-tsrust-node]').length`),0);assert.deepEqual(errors,[]);
 assert(await evaluate(`trackedObservers.length>=2&&trackedObservers.every(observer=>observer.targets.size===0)`),'Disposal must release all platform observer targets');
 console.log(JSON.stringify({case:'paired-landing-and-visibility',status:'pass',checks:['real code and Rust binding','chapter visibility','sticky header','320–1440px layout','both themes','reduced-motion readability','retained counter','nested clipping','zero area','initial notification and deduplication','hidden restoration','conditional removal and fresh identity','independent observer teardown','pending observer disposal','independent Node-after-tsc notification oracle'],timed:false}));
}
