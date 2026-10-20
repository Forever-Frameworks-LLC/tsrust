import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
export async function observeReactiveProps({evaluate,call,errors,expected}) {
 const observed=[];
 const snapshot=async()=>{observed.push(await evaluate(`(()=>{const node=document.querySelector('[data-slot=notice]');return {tone:node.dataset.variant,surface:node.className.includes('bg-muted')?'bg-muted':'bg-background',text:node.textContent,disabled:document.querySelector('[data-slot=replacement]').disabled};})()`));};
 assert.equal(await evaluate(`document.querySelectorAll('button').length`),expected.buttons);
 await evaluate(`window.savedNotice=document.querySelector('[data-slot=notice]');window.savedIncrement=document.querySelector('[data-slot=replacement]');window.savedIds=Array.from(document.querySelectorAll('[data-tsrust-node]'),node=>node.dataset.tsrustNode)`);
 await snapshot();await evaluate(`Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Toggle tone').click()`);await snapshot();
 await evaluate(`savedIncrement.focus();savedIncrement.click()`);await snapshot();
 assert(await evaluate(`document.activeElement===savedIncrement`));
 await evaluate(`Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Toggle tone').click()`);await snapshot();await evaluate('savedIncrement.click()');await snapshot();
 assert.deepEqual(observed,expected.steps);await evaluate('savedIncrement.click()');assert.equal(await evaluate('savedNotice.textContent'),expected.steps.at(-1).text);assert(await evaluate('savedIncrement.disabled'));
 assert(await evaluate(`document.querySelector('[data-slot=notice]')===savedNotice&&document.querySelector('[data-slot=replacement]')===savedIncrement`));
 assert.deepEqual(await evaluate(`Array.from(document.querySelectorAll('[data-tsrust-node]'),node=>node.dataset.tsrustNode)`),await evaluate('savedIds'));
 for(const width of [390,768,1280]){await call('Emulation.setDeviceMetricsOverride',{width,height:800,deviceScaleFactor:1,mobile:false});await evaluate('new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))');assert(await evaluate(`document.documentElement.scrollWidth<=innerWidth`));}
 const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert.equal(tree.filter(node=>node.role?.value==='button').length,2);
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.equal(await evaluate(`document.querySelectorAll('[data-tsrust-node]').length`),0);assert.deepEqual(errors,[]);
 for(const source of ['reactive-props-reference.ts','../../examples/ui-reactive-props/src/app/page.tsrx','../../examples/ui-reactive-props/src/main.tsr','../../examples/ui-reactive-props/tsrust.config.json'])console.log(JSON.stringify({source,sha256:createHash('sha256').update(fs.readFileSync(new URL(source,import.meta.url))).digest('hex')}));
 console.log(JSON.stringify({case:'reactive-props-composition',status:'pass',observed,checks:['live literal props','live classes and data-variant','derived exact scalar values','forwarded owned handler and children','no wrapper or remount','retained focus','responsive layout','accessibility','disposal'],engines:['wasm','node-tsc'],timed:false}));
}
