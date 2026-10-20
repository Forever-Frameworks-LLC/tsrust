import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
export async function observeDisclosure({evaluate,call,errors,expected,site}){
 const observed=[];await call('Page.bringToFront');
 await evaluate(`window.triggers=[...document.querySelectorAll('#disclosure-example [data-slot="disclosure-trigger"]')];window.panels=[null,null];window.panelIds=[null,null];window.drafts=[null,null];`);
 for(let index=0;index<expected.steps.length;index++){
  if(index===1||index===3||index===4){await evaluate('triggers[0].focus()');assert(await evaluate('document.activeElement===triggers[0]'));await call('Input.dispatchKeyEvent',{type:'keyDown',key:'Enter',code:'Enter',text:'\r',unmodifiedText:'\r',windowsVirtualKeyCode:13});await call('Input.dispatchKeyEvent',{type:'keyUp',key:'Enter',code:'Enter',windowsVirtualKeyCode:13});}
  if(index===2||index===5){await evaluate('triggers[1].focus()');assert(await evaluate('document.activeElement===triggers[1]'));await call('Input.dispatchKeyEvent',{type:'keyDown',key:' ',code:'Space',text:' ',unmodifiedText:' ',windowsVirtualKeyCode:32});await call('Input.dispatchKeyEvent',{type:'keyUp',key:' ',code:'Space',windowsVirtualKeyCode:32});}
  await evaluate(`triggers.forEach((button,i)=>{const id=button.getAttribute('aria-controls');if(id&&!panels[i]){panels[i]=document.getElementById(id);panelIds[i]=id;drafts[i]=panels[i].querySelector('input');}})`);
  if(index===2){assert.equal(await evaluate('new Set(panelIds).size'),2);assert(await evaluate('panelIds.every(id=>id&&id.length>0)'));}
  const state=await evaluate(`({expanded:triggers.map(button=>button.getAttribute('aria-expanded')==='true'),visible:document.querySelectorAll('#disclosure-example [data-slot=\"disclosure-panel\"]').length})`);observed.push(state);assert.deepEqual(state,expected.steps[index]);
  assert(await evaluate(`panels.every((panel,i)=>!panel||(panel.id===panelIds[i]&&(!panel.isConnected||(document.getElementById(panelIds[i])===panel&&panel.querySelector('input')===drafts[i]))))&&triggers.every((button,i)=>button.getAttribute('aria-controls')===(panels[i]?.isConnected?panelIds[i]:null))`),JSON.stringify({index,state}));
  if(index===1)await evaluate(`drafts[0].value='Kept 🦀';drafts[0].dispatchEvent(new Event('input',{bubbles:true}))`);
  if(index>=2)assert.equal(await evaluate('drafts[0].value'),'Kept 🦀');
  const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);for(const [i,name]of ['Project settings','Workspace settings'].entries()){const trigger=tree.find(node=>node.role?.value==='button'&&node.name?.value===name);assert(trigger);assert.equal(trigger.properties.find(property=>property.name==='expanded')?.value.value,state.expanded[i]);}
 }
 await evaluate(`document.querySelector('#disclosure-disabled').click()`);assert.equal(await evaluate(`document.querySelector('#disclosure-disabled').getAttribute('aria-expanded')`),'false');
 for(const width of [390,768,1280]){await call('Emulation.setDeviceMetricsOverride',{width,height:950,deviceScaleFactor:1,mobile:false});await evaluate('new Promise(resolve=>requestAnimationFrame(resolve))');assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));}
 await evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click();triggers[0].scrollIntoView({block:'center'})`);assert.equal(await evaluate('drafts[0].value'),'Kept 🦀');
 fs.writeFileSync(path.join(site,'disclosure.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
 for(const source of ['disclosure-reference.ts','disclosure-observer.mjs','../../docs/src/examples/disclosure-semantics.tsrx','../../crates/tsrust-compiler/src/stdlib/ui/api.tsr'])console.log(JSON.stringify({source,sha256:createHash('sha256').update(fs.readFileSync(new URL(source,import.meta.url))).digest('hex')}));
 console.log(JSON.stringify({case:'disclosure-semantics',status:'pass',observed,timed:false}));
}
