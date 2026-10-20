import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
export async function observeEmpty({evaluate,call,errors,expected,site}){
 const observed=[];await call('Page.bringToFront');
 await evaluate(`window.emptyMedia=document.querySelector('#empty-live-media');window.emptyDefault=document.querySelector('#empty-default-icon');window.emptyExplicit=document.querySelector('#empty-explicit-icon')`);
 for(let i=0;i<expected.steps.length;i++){
  if([1,3,4].includes(i))await evaluate(`document.querySelector('#empty-cycle').click()`);if(i===2)await evaluate(`document.querySelector('#empty-size').click()`);
  const value=await evaluate(`({variant:emptyMedia.getAttribute('data-variant'),defaultSize:emptyDefault.getBoundingClientRect().width,explicitSize:emptyExplicit.getBoundingClientRect().width})`);observed.push(value);assert.deepEqual(value,expected.steps[i]);
  assert(await evaluate(`document.querySelector('#empty-live-media')===emptyMedia&&document.querySelector('#empty-default-icon')===emptyDefault&&document.querySelector('#empty-explicit-icon')===emptyExplicit`));
  assert.equal(await evaluate('getComputedStyle(emptyDefault).pointerEvents'),'none');
 }
 const text=await evaluate(`(()=>{const node=document.querySelector('#empty-description');const link=document.querySelector('#empty-guide');const title=document.querySelector('[data-slot="empty-title"]');const style=getComputedStyle(node);return {font:style.fontSize,leading:style.lineHeight,tracking:getComputedStyle(title).letterSpacing,decoration:getComputedStyle(link).textDecorationLine,offset:getComputedStyle(link).textUnderlineOffset,border:getComputedStyle(document.querySelector('#empty-live')).borderTopStyle};})()`);assert.deepEqual(text,{font:'14px',leading:'22.75px',tracking:'-0.35px',decoration:'underline',offset:'4px',border:'dashed'});
 await evaluate(`document.querySelector('#empty-create').click()`);assert(await evaluate(`document.querySelector('#empty-projects').textContent.includes('Your workspace is ready.')`));await evaluate(`document.querySelector('#empty-reset').click()`);assert(await evaluate(`document.querySelector('#empty-projects [data-slot="empty"]')!==null`));
 const scoped=await evaluate(`(()=>{const title=document.querySelector('#empty-themed-title');const node=title.closest('[data-slot="empty"]');return {family:getComputedStyle(title).fontFamily,size:getComputedStyle(title).fontSize,padding:getComputedStyle(node).paddingTop};})()`);assert.equal(scoped.family,'serif');assert.equal(scoped.size,'17.5px');assert.equal(scoped.padding,'30px');
 await evaluate(`document.querySelector('#empty-local-theme').click()`);assert.equal(await evaluate(`getComputedStyle(document.querySelector('#empty-themed-title')).fontFamily`),'serif');
 for(const width of [390,768,1280]){await call('Emulation.setDeviceMetricsOverride',{width,height:1000,deviceScaleFactor:1,mobile:false});await evaluate('new Promise(resolve=>requestAnimationFrame(resolve))');assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));}
 await evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click();document.querySelector('#empty-media-states').scrollIntoView({block:'center'})`);
 fs.writeFileSync(path.join(site,'empty.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert(tree.some(node=>node.role?.value==='button'&&node.name?.value==='Create project'));assert(!tree.some(node=>node.role?.value==='image'&&['','undefined'].includes(node.name?.value)));
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
 for(const source of ['empty-reference.ts','empty-observer.mjs','../../docs/src/examples/empty-states.tsrx','../../crates/tsrust-compiler/src/ui-catalog/empty-1.0.0.json'])console.log(JSON.stringify({source,sha256:createHash('sha256').update(fs.readFileSync(new URL(source,import.meta.url))).digest('hex')}));
 console.log(JSON.stringify({case:'empty-component',status:'pass',observed,text,scoped,timed:false}));
}
