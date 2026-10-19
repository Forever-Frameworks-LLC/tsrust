// Only the catalog/example-card integration is new; foundation acceptance is reused.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
export async function observeDocsExample({evaluate,call,errors,site}) {
 const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),'../../docs');
 const catalog=JSON.parse(fs.readFileSync(path.join(root,'target/docs-content/catalog.json'),'utf8'));
 const exact=fs.readFileSync(path.join(root,'src/examples/counter.tsrx'),'utf8');
 const intro=catalog.pages.find(page=>page.metadata.id==='introduction');
 await call('Emulation.setDeviceMetricsOverride',{width:1100,height:780,deviceScaleFactor:1,mobile:false});
 assert.equal(await evaluate('document.querySelector("h1").textContent'),intro.document.title);
 assert(await evaluate(`document.body.textContent.includes(${JSON.stringify(catalog.pages.length+' page destinations')})`));
 const click=async text=>evaluate(`Array.from(document.querySelectorAll('button')).find(button=>button.textContent===${JSON.stringify(text)}).click()`);
 await click('Increment');
 await evaluate(`window.exampleDraft=document.querySelector('#example-draft');exampleDraft.value='Retained 🦀 draft';exampleDraft.dispatchEvent(new Event('input',{bubbles:true}));`);
 await click('Show source');assert.equal(await evaluate(`document.querySelector('#counter-preview')`),null);
 const shown=await evaluate(`(()=>{window.exampleSource=document.querySelector('#counter-source textarea');exampleSource.focus();exampleSource.setSelectionRange(7,35);return {value:exampleSource.value,readonly:exampleSource.readOnly,label:exampleSource.getAttribute('aria-label')};})()`);
 assert.equal(shown.value,exact);assert.equal(shown.readonly,true);assert.equal(shown.label,'State stays with the example source');
 await call('Input.insertText',{text:'must not edit'});assert.equal(await evaluate('exampleSource.value'),exact);
 const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);
 assert(tree.some(node=>node.name?.value===shown.label&&node.role?.value==='textbox'),'Source has an accessible text-control role and name');
 assert(!tree.some(node=>node.role?.value==='button'&&node.name?.value==='Increment'),'Hidden preview leaves accessibility');
 await click('Light / dark');assert(await evaluate(`document.querySelector('#counter-source textarea')===exampleSource&&exampleSource.selectionStart===7&&exampleSource.selectionEnd===35`),'Theme retains source DOM/selection');
 await click('Show preview');assert.equal(await evaluate(`document.querySelector('#counter-source')`),null);
 assert(await evaluate(`document.querySelector('#example-draft')===exampleDraft&&exampleDraft.value==='Retained 🦀 draft'&&document.querySelector('#example-count').textContent==='Count: 1'`),'Preview switch retains the actual component instance');
 await click('Show source');assert(await evaluate(`document.querySelector('#counter-source textarea')===exampleSource`));
 await call('Emulation.setDeviceMetricsOverride',{width:480,height:520,deviceScaleFactor:2,mobile:false});
 assert(await evaluate(`document.querySelector('#counter-source textarea').getBoundingClientRect().right<=innerWidth`),'Source fits narrow article');
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);
 await evaluate(`document.querySelector('#article').scrollTop=0`);
 const screenshot=await call('Page.captureScreenshot',{format:'png'});fs.writeFileSync(path.join(site,'preview.png'),Buffer.from(screenshot.data,'base64'));
 await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'browser-docs-example-card',status:'pass',pages:catalog.pages.length,examples:catalog.examples.length,checks:['checked introduction/catalog','exact readonly source','actual component retention','source DOM/selection retention','AX source and hidden preview','narrow source layout','disposal'],timed:false}));
}
