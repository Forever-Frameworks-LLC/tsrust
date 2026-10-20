import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
export async function observeLazyChildren({evaluate,call,errors,expected,site}){
 const observed=[];const snapshot=async()=>observed.push(await evaluate(`({parent:document.querySelector('#lazy-parent-count').textContent,child:document.querySelector('#lazy-child-count')?.textContent??null,draft:document.querySelector('#lazy-draft').value})`));
 const click=async id=>evaluate(`document.querySelector('#${id}').click()`);
 await evaluate(`window.lazyDraft=document.querySelector('#lazy-draft');window.lazyParent=document.querySelector('#lazy-parent-count')`);await snapshot();
 await evaluate(`lazyDraft.value='Kept 🦀';lazyDraft.dispatchEvent(new Event('input',{bubbles:true}))`);await click('lazy-increment-parent');await click('lazy-toggle');await snapshot();
 await evaluate(`window.firstDetail=document.querySelector('#lazy-child-count')`);await click('lazy-increment-child');await click('lazy-increment-child');await snapshot();
 await click('lazy-toggle');await snapshot();assert(await evaluate('!firstDetail.isConnected&&!document.querySelector("#lazy-content")'));
 await click('lazy-toggle');await snapshot();assert(await evaluate('document.querySelector("#lazy-child-count")!==firstDetail'));await evaluate(`window.secondDetail=document.querySelector('#lazy-child-count')`);
 await click('lazy-increment-child');await click('lazy-increment-parent');await snapshot();assert(await evaluate('document.querySelector("#lazy-child-count")===secondDetail'));
 await click('lazy-toggle');await snapshot();await evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click()`);await click('lazy-toggle');await snapshot();
 assert.deepEqual(observed,expected.steps);assert(await evaluate('document.querySelector("#lazy-draft")===lazyDraft&&document.querySelector("#lazy-parent-count")===lazyParent'));
 const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert.equal(tree.filter(node=>node.role?.value==='button'&&node.name?.value==='Increment detail').length,1);
 for(const width of [390,768,1280]){await call('Emulation.setDeviceMetricsOverride',{width,height:950,deviceScaleFactor:1,mobile:false});await evaluate('new Promise(resolve=>requestAnimationFrame(resolve))');assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));}
 await evaluate(`document.querySelector('#lazy-toggle').scrollIntoView({block:'center'})`);fs.writeFileSync(path.join(site,'lazy-children.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
 for(const source of ['lazy-children-reference.ts','lazy-children-observer.mjs','../../docs/src/examples/lazy-children.tsrx','../../crates/tsrust-compiler/src/stdlib/ui/components.rs','../../crates/tsrust-compiler/src/stdlib/ui/api.tsr'])console.log(JSON.stringify({source,sha256:createHash('sha256').update(fs.readFileSync(new URL(source,import.meta.url))).digest('hex')}));
 console.log(JSON.stringify({case:'lazy-children',status:'pass',observed,timed:false}));
}
