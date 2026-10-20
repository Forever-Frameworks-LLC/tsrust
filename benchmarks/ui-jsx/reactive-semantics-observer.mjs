import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
export async function observeReactiveSemantics({evaluate,call,errors,expected,site}){
 const observed=[];
 await evaluate(`window.savedField=document.querySelector('#metadata-draft');window.savedButton=document.querySelector('#metadata-toggle');window.savedIcon=document.querySelector('#metadata-icon');savedField.focus();savedField.value='Kept 🦀';savedField.dispatchEvent(new Event('input',{bubbles:true}));savedField.setSelectionRange(2,4)`);
 for(let i=0;i<expected.steps.length;i++){
  if(i)await evaluate(`savedButton.click()`);
  const value=await evaluate(`({label:savedField.getAttribute('aria-label'),description:savedField.getAttribute('aria-describedby'),invalid:savedField.getAttribute('aria-invalid'),hidden:document.querySelector('#metadata-decoration').getAttribute('aria-hidden')==='true',toggle:savedButton.getAttribute('aria-label'),badge:document.querySelector('#metadata-badge').getAttribute('aria-label'),image:savedIcon.getAttribute('aria-label'),role:savedIcon.getAttribute('role')})`);
  observed.push(value);assert.deepEqual(value,expected.steps[i]);
  assert(await evaluate(`document.querySelector('#metadata-draft')===savedField&&document.querySelector('#metadata-toggle')===savedButton&&document.querySelector('#metadata-icon')===savedIcon&&savedField.value==='Kept 🦀'&&document.activeElement===savedField&&savedField.selectionStart===2&&savedField.selectionEnd===4`));
  const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);const field=tree.find(node=>node.role?.value==='textbox'&&node.name?.value===value.label);assert(field,'Current accessible field name');assert.equal(field.description?.value,i%2?'That name is already in use.':'Choose a unique project name.');assert.equal(tree.some(node=>node.role?.value==='image'&&node.name?.value==='Saved'),Boolean(i%2));
 }
 for(const width of [390,768,1280]){await call('Emulation.setDeviceMetricsOverride',{width,height:950,deviceScaleFactor:1,mobile:false});await evaluate('new Promise(resolve=>requestAnimationFrame(resolve))');assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));}
 await evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click();savedButton.scrollIntoView({block:'center'})`);assert.equal(await evaluate('savedField.value'),'Kept 🦀');
 fs.writeFileSync(path.join(site,'reactive-semantics.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
 for(const source of ['reactive-semantics-reference.ts','reactive-semantics-observer.mjs','../../docs/src/examples/reactive-semantics.tsrx','../../crates/tsrust-compiler/src/stdlib/ui/api.tsr','../../crates/tsrust-ui/src/lib.rs'])console.log(JSON.stringify({source,sha256:createHash('sha256').update(fs.readFileSync(new URL(source,import.meta.url))).digest('hex')}));
 console.log(JSON.stringify({case:'reactive-semantics',status:'pass',observed,timed:false}));
}
