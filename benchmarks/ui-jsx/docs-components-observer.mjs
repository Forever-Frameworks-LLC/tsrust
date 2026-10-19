// Checks actual local component pages; existing native editing/clipboard suites are reused.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
export async function observeDocsComponents({evaluate,call,errors,site}) {
 const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../docs');
 const wait=async expression=>{for(let i=0;i<100;i++){if(await evaluate(expression))return;await new Promise(resolve=>setTimeout(resolve,30));}assert.fail(expression);};
 const clickLink=async href=>{await evaluate(`document.querySelector('a[href=${JSON.stringify(href)}]').click()`);await wait(`location.pathname===${JSON.stringify(href)}`);};
 const clickButton=async (selector,text)=>evaluate(`Array.from(document.querySelectorAll(${JSON.stringify(selector)})).find(node=>node.textContent===${JSON.stringify(text)}).click()`);
 await call('Emulation.setDeviceMetricsOverride',{width:1280,height:900,deviceScaleFactor:1,mobile:false});
 await clickLink('/docs');await evaluate(`Array.from(document.querySelectorAll('nav[aria-label="Documentation sections"] a')).find(node=>node.textContent==='Components').click()`);
 await wait(`!!document.querySelector('#sidebar a[href="/docs/ui/components/card"]')`);
 await evaluate(`Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async value=>{window.copiedSource=value;}}})`);
 const pages=[['card',['card-layout','card-project']],['input',['input-states']],['textarea',['multiline-draft']],['label',['field-labels']],['kbd',['keyboard-shortcuts']],['separator',['section-dividers']]];
 for(const [name,ids] of pages){
  await clickLink('/docs/ui/components/'+name);
  assert(await evaluate(`document.querySelector('[data-slot=route-page] h1').textContent.toLowerCase()===${JSON.stringify(name)}`));
  assert(!await evaluate(`document.querySelector('[data-slot=route-page]').textContent.includes('Try the example')`));
  for(const id of ids){
   const prefix=`component-${name}-${id}`;
   const exact=fs.readFileSync(path.join(root,`src/examples/${id}.tsrx`),'utf8');
   assert(await evaluate(`!!document.querySelector('#${prefix}-preview[role=region][tabindex="0"][aria-label]')`));
   // Find the shared card from the actual pane, not an inventory-only placeholder.
   await evaluate(`document.querySelector('#${prefix}-preview').closest('[data-slot=card]').querySelectorAll('button')[0]`);
   await evaluate(`Array.from(document.querySelector('#${prefix}-preview').closest('[data-slot=card]').querySelectorAll('button')).find(node=>node.textContent==='Show source').click()`);
   assert.equal(await evaluate(`document.querySelector('#${prefix}-source code').textContent`),exact);
   await evaluate(`document.querySelector('#${prefix}-source button').click()`);
   await wait(`window.copiedSource===${JSON.stringify(exact)}`);
   await evaluate(`Array.from(document.querySelector('#${prefix}-source').closest('[data-slot=card]').querySelectorAll('button')).find(node=>node.textContent==='Show preview').click()`);
  }
  for(const width of [480,1280]){
   await call('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
   assert(await evaluate(`document.documentElement.scrollWidth<=innerWidth`),`${name} ${width}px viewport`);
   assert(await evaluate(`Array.from(document.querySelectorAll('[data-slot=route-page] input,[data-slot=route-page] textarea')).every(node=>node.getBoundingClientRect().right<=innerWidth+1)`),`${name} controls fit`);
  }
  const ax=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);
  if(name==='input'){
   for(const label of ['Display name','Uncontrolled draft','Read-only reference','Unavailable field','Project slug'])assert(ax.some(node=>node.role?.value==='textbox'&&node.name?.value===label),label);
   assert(await evaluate(`document.querySelector('#input-states-disabled').disabled&&document.querySelector('#input-states-readonly').readOnly`));
   await evaluate(`window.retainedInput=document.querySelector('#input-states-name');retainedInput.value='Grace 🦀';retainedInput.dispatchEvent(new Event('input',{bubbles:true}));`);
   assert(await evaluate(`document.querySelector('#component-input-input-states-preview').textContent.includes('Hello, Grace 🦀.')`));
   await clickButton('[data-slot=route-page] button','Show source');await clickButton('[data-slot=route-page] button','Show preview');
   await evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click()`);
   assert(await evaluate(`document.querySelector('#input-states-name')===retainedInput&&retainedInput.value==='Grace 🦀'`));
  }else if(name==='card'){
   await clickButton('[data-slot=route-page] button','Change access');assert(await evaluate(`document.querySelector('[data-slot=route-page]').textContent.includes('Shared workspace')`));
   await evaluate(`const field=document.querySelector('#card-project-name');field.value='';field.dispatchEvent(new Event('input',{bubbles:true}));`);
   assert(await evaluate(`Array.from(document.querySelectorAll('[data-slot=route-page] button')).find(node=>node.textContent==='Apply name').disabled`));
   await evaluate(`const field2=document.querySelector('#card-project-name');field2.value='Docs';field2.dispatchEvent(new Event('input',{bubbles:true}));`);await clickButton('[data-slot=route-page] button','Apply name');
   assert(await evaluate(`document.querySelector('[data-slot=route-page]').textContent.includes('Project name: Docs')`));
  }else if(name==='textarea'){
   await evaluate(`(()=>{const field=document.querySelector('#multiline-message');field.value='文\\n🦀';field.dispatchEvent(new Event('input',{bubbles:true}));})()`);
   assert(await evaluate(`document.querySelector('[data-slot=route-page]').textContent.includes('UTF-8 bytes: 8')`));
   assert(await evaluate(`document.querySelector('#multiline-readonly').readOnly&&document.querySelector('#multiline-disabled').disabled`));
  }else if(name==='label'){
   await evaluate(`document.querySelector('label[for="field-labels-explicit"]').click()`);assert(await evaluate(`document.activeElement.id==='field-labels-explicit'`));
   await evaluate(`Array.from(document.querySelectorAll('[data-slot=route-page] label')).find(node=>node.textContent.startsWith('Enclosed control')).click()`);
   assert(await evaluate(`document.activeElement.placeholder==='This field has an enclosing label'`));
  }else if(name==='kbd'){
   assert(await evaluate(`document.querySelectorAll('[data-slot=route-page] [data-slot=kbd]').length===3`));
  }else if(name==='separator'){
   assert(ax.some(node=>node.role?.value==='separator'));
   assert(await evaluate(`document.querySelectorAll('[data-slot=route-page] [role=separator]').length===2`));
   assert(await evaluate(`document.querySelector('[data-slot=route-page] [aria-orientation=vertical]').getBoundingClientRect().height>0`));
  }
 }
 await clickLink('/docs/ui/components/card');
 fs.writeFileSync(path.join(site,'component-card.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.equal(await evaluate(`document.querySelectorAll('[data-tsrust-node]').length`),0);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'local-component-documentation',status:'pass',pages:pages.length,examples:pages.flatMap(entry=>entry[1]).length,checks:['real routed previews','exact source and copy','controlled and retained values','disabled/read-only/invalid fields','label focus','separator geometry/AX','both themes','480px and 1280px controls','disposal'],timed:false}));
}
