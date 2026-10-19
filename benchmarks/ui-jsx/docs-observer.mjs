// Untimed docs foundation observer; shared driver owns and reaps server/browser.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
export async function observeDocs({evaluate,call,errors,site}) {
 await call('Emulation.setDeviceMetricsOverride',{width:800,height:480,deviceScaleFactor:2,mobile:false});
 const click=async text=>evaluate(`Array.from(document.querySelectorAll('button')).find(button=>button.textContent===${JSON.stringify(text)}).click()`);
 assert.equal(await evaluate(`document.querySelector('#example-count').textContent`),'Count: 0');
 await click('Increment');
 await evaluate(`window.draft=document.querySelector('#example-draft');draft.value='Edited 🦀';draft.dispatchEvent(new Event('input',{bubbles:true}));draft.focus();document.querySelector('#preview').scrollTop=35;window.previewOffset=document.querySelector('#preview').scrollTop;`);
 assert(await evaluate('previewOffset>0'),'Example must have actual nested overflow');
 await click('Light / dark');
 assert(await evaluate(`document.querySelector('#example-draft')===draft&&document.activeElement===draft`),'Theme preserves DOM/focus');
 assert.equal(await evaluate(`getComputedStyle(document.querySelector('#docs')).backgroundColor`),'rgb(9, 9, 11)');
 await click('Hide / show preview');assert.equal(await evaluate(`document.querySelector('#preview')!==null`),false);
 await click('Hide / show preview');
 assert(await evaluate(`document.querySelector('#preview').scrollTop===previewOffset&&document.querySelector('#example-draft')===draft`),'Retained preview keeps DOM and scroll position');
 assert.equal(await evaluate(`document.querySelector('#example-draft').value`),'Edited 🦀');
 assert.equal(await evaluate(`document.querySelector('#example-count').textContent`),'Count: 1');
 const geometry=await evaluate(`(()=>{const article=document.querySelector('#article'),sidebar=document.querySelector('#sidebar');article.scrollTop=140;return {article:article.scrollTop,sidebar:sidebar.scrollTop,window:scrollY,height:document.querySelector('#docs').getBoundingClientRect().height,tab:article.tabIndex};})()`);
 assert.equal(geometry.article,140);assert.equal(geometry.sidebar,0);assert.equal(geometry.window,0);assert.equal(geometry.height,480);assert.equal(geometry.tab,0);
 // Real wheel input targets the article; sidebar remains independent.
 const point=await evaluate(`(()=>{const r=document.querySelector('#article').getBoundingClientRect();return {x:r.right-12,y:r.y+12};})()`);
 await call('Input.dispatchMouseEvent',{type:'mouseWheel',...point,deltaX:0,deltaY:80});
 for(let i=0;i<30&&await evaluate(`document.querySelector('#article').scrollTop`)<=140;i++)await new Promise(resolve=>setTimeout(resolve,50));
 assert(await evaluate(`document.querySelector('#article').scrollTop>140&&document.querySelector('#sidebar').scrollTop===0`),'Wheel scrolls article independently');
 const before=await evaluate(`document.querySelector('#article').scrollTop`);
 await evaluate(`document.querySelector('#article').focus()`);
 await call('Input.dispatchKeyEvent',{type:'keyDown',key:'PageDown',code:'PageDown',windowsVirtualKeyCode:34});
 await call('Input.dispatchKeyEvent',{type:'keyUp',key:'PageDown',code:'PageDown',windowsVirtualKeyCode:34});
 for(let i=0;i<30&&await evaluate(`document.querySelector('#article').scrollTop`)<=before;i++)await new Promise(resolve=>setTimeout(resolve,50));
 assert(await evaluate(`document.querySelector('#article').scrollTop>${before}`),'Focused pane supports PageDown');
 await call('Emulation.setDeviceMetricsOverride',{width:480,height:420,deviceScaleFactor:1,mobile:false});
 assert(await evaluate(`document.querySelector('#article').getBoundingClientRect().right<=innerWidth&&document.querySelector('#docs').getBoundingClientRect().height===innerHeight`),'Narrow layout stays within viewport');
 const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);
 for(const name of ['Documentation index','Documentation article'])assert(tree.some(node=>node.role?.value==='region'&&node.name?.value===name),name);
 assert(tree.some(node=>node.role?.value==='separator'),'Separator exposed');
 assert(await evaluate(`document.querySelector('[data-slot=kbd]').tagName==='KBD'`),'Real keyboard text semantics');
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);
 const screenshot=await call('Page.captureScreenshot',{format:'png'});fs.writeFileSync(path.join(site,'preview.png'),Buffer.from(screenshot.data,'base64'));
 await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'browser-docs-foundation',status:'pass',checks:['retained DOM/focus/draft/counter','retained pane offset','independent wheel panes','pane keyboard scrolling','viewport sizing','narrow layout','named AX regions','kbd/separator semantics','disposal'],timed:false}));
}
