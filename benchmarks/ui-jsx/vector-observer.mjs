import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
export async function observeVectors({evaluate,call,errors,expected,site}){
 assert(await evaluate(`document.querySelector('#vector-chart')?.namespaceURI==='http://www.w3.org/2000/svg'`));
 await evaluate(`window.series=document.querySelector('#vector-series');window.area=document.querySelector('#vector-area');window.chart=document.querySelector('#vector-chart')`);
 const observed=[];const snapshot=async()=>observed.push(await evaluate(`({line:series.getAttribute('d'),area:area.getAttribute('d'),text:document.querySelector('#vector-values').textContent})`));await snapshot();
 await evaluate(`document.querySelector('#vector-change-series').click()`);await snapshot();await evaluate(`document.querySelector('#vector-change-series').click()`);await snapshot();assert.deepEqual(observed,expected.steps);
 assert(await evaluate(`series===document.querySelector('#vector-series')&&area===document.querySelector('#vector-area')&&chart===document.querySelector('#vector-chart')`));
 for(const entry of expected.browser){const actual=await evaluate(`(()=>{const svg=document.querySelector('#vector-${entry.id}');const matrix=svg.getScreenCTM(),rect=svg.getBoundingClientRect();const a=new DOMPoint(10,20).matrixTransform(matrix),b=new DOMPoint(50,40).matrixTransform(matrix);return {corners:[a.x-rect.x,a.y-rect.y,b.x-rect.x,b.y-rect.y],namespace:svg.firstElementChild.namespaceURI};})()`);assert.equal(actual.namespace,'http://www.w3.org/2000/svg');for(let i=0;i<4;i++)assert(Math.abs(actual.corners[i]-entry.corners[i])<.02,JSON.stringify({entry,actual}));}
 await evaluate(`document.querySelector('#vector-save').focus();document.querySelector('#vector-save').click()`);assert.equal(await evaluate(`document.querySelector('#vector-save-status').textContent`),'Saved');assert(await evaluate(`document.activeElement===document.querySelector('#vector-save')`));
 const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert(tree.some(node=>node.role?.value==='image'&&node.name?.value==='Weekly successful builds'));assert(tree.some(node=>node.role?.value==='button'&&node.name?.value==='Save'));assert(!tree.some(node=>node.role?.value==='image'&&!node.name?.value));
 for(const width of [390,768,1280]){await call('Emulation.setDeviceMetricsOverride',{width,height:950,deviceScaleFactor:1,mobile:false});await evaluate('new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))');assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));}
 const before=await evaluate('getComputedStyle(series).stroke');await evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click()`);assert.notEqual(await evaluate('getComputedStyle(series).stroke'),before);
 await evaluate(`chart.scrollIntoView({block:'center'})`);fs.writeFileSync(path.join(site,'vectors.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.equal(await evaluate('document.querySelectorAll("[data-tsrust-node]").length'),0);assert.deepEqual(errors,[]);console.log(JSON.stringify({case:'vectors',status:'pass',observed,viewports:expected.browser,timed:false}));
}
