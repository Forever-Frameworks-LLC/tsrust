import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {textContrasts} from './text-contrast.mjs';
import {documentationLink} from './documentation-links.mjs';

export async function observeDocsAlert({evaluate,call,errors,site,expected}) {
 const wait=async expression=>{for(let i=0;i<120;i++){if(await evaluate(expression))return;await new Promise(resolve=>setTimeout(resolve,30));}assert.fail(expression);};
 const settle=()=>evaluate('new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))');
 await call('Emulation.setDeviceMetricsOverride',{width:1280,height:1000,deviceScaleFactor:1,mobile:false});await settle();
 await evaluate(`document.querySelector('a[href="/docs"]').click()`);await wait(`location.pathname==='/docs'`);const open=documentationLink(evaluate);await open('/docs/ui/components/alert');
 await wait(`!!document.querySelector('#alert-action-example')`);
 assert.equal(await evaluate(`document.querySelector('#alert-destructive').getAttribute('data-variant')`),'destructive');
 assert.equal(await evaluate(`document.querySelector('#alert-default').getAttribute('role')`),'alert');
 assert.equal(await evaluate(`document.querySelector('#alert-validation-message').getAttribute('role')`),'status');
 await evaluate(`window.notice=document.querySelector('#alert-action-example');window.retry=notice.querySelector('button');retry.click()`);assert.equal(await evaluate(`document.querySelector('#alert-retries').textContent`),'Retry requests: 1');
 await evaluate(`Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Hide notice').click()`);assert(await evaluate(`!notice.isConnected`));
 await evaluate(`Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Show notice').click()`);assert(await evaluate(`notice===document.querySelector('#alert-action-example')&&notice.querySelector('button')===retry`));
 assert.equal(await evaluate(`document.querySelector('#alert-retries').textContent`),'Retry requests: 1');
 await evaluate(`window.field=document.querySelector('#alert-project-name');field.focus();field.value='Workbench';field.dispatchEvent(new Event('input',{bubbles:true}));`);assert(await evaluate(`document.activeElement===field&&!document.querySelector('#alert-validation-message')`));
 await evaluate(`field.value='  ';field.dispatchEvent(new Event('input',{bubbles:true}));`);assert(await evaluate(`document.activeElement===field&&!!document.querySelector('#alert-validation-message')`));
 const local=await evaluate(`getComputedStyle(document.querySelector('#alert-theme-scope')).backgroundColor`);await evaluate(`Array.from(document.querySelectorAll('button')).find(node=>node.textContent==='Toggle example theme').click()`);assert.notEqual(await evaluate(`getComputedStyle(document.querySelector('#alert-theme-scope')).backgroundColor`),local);assert(await evaluate(`document.querySelector('#alert-local-mode').textContent.includes('dark')`));
 await evaluate(`(()=>{const probe=document.createElement('div');probe.id='contrast-calibration';probe.style.cssText='background:#000;color:#fff';probe.innerHTML='<span>Probe</span>';document.body.append(probe);})()`);const positive=await textContrasts(evaluate,'#contrast-calibration span');assert(Math.abs(positive[0]-21)<0.001);await evaluate(`document.querySelector('#contrast-calibration').style.cssText='background:#fff;color:#aaa'`);const negative=await textContrasts(evaluate,'#contrast-calibration span');assert(negative[0]<4.5);await evaluate(`document.querySelector('#contrast-calibration').remove()`);
 const lightContrast=await textContrasts(evaluate,'[data-slot=alert] [data-tsrust-text]');assert(lightContrast.length>0&&lightContrast.every(value=>value>=4.5),JSON.stringify(lightContrast));
 const alertColor=await evaluate(`getComputedStyle(document.querySelector('#alert-destructive [data-slot="alert-description"]')).color`);assert(alertColor.includes('185')||alertColor.startsWith('color('),alertColor);
 // Positioning is measured against independent CSS border/content box equations from Node-after-tsc.
 await open('/docs/ui/foundations/positioning');await wait(`!!document.querySelector('#position-frame-0')`);const observed=[];
 for(let index=0;index<4;index++)observed.push(await evaluate(`(()=>{const parent=document.querySelector('#position-frame-${index}').getBoundingClientRect();const child=document.querySelector('#position-child-${index}').getBoundingClientRect();return {index:${index},x:child.x-parent.x,y:child.y-parent.y,width:child.width,height:child.height};})()`));
 for(let index=0;index<4;index++)for(const key of ['x','y','width','height'])assert(Math.abs(observed[index][key]-expected.geometry[index][key])<0.05,JSON.stringify({index,key,observed,expected}));
 await open('/docs/ui/components/alert');await wait(`!!document.querySelector('#alert-action-example')`);assert(await evaluate(`document.querySelector('#alert-action-example')===notice&&notice.querySelector('button')===retry`));
 await evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click()`);assert(await evaluate(`document.querySelector('#alert-theme-scope')&&document.querySelector('#alert-local-mode').textContent.includes('dark')`));
 const darkContrast=await textContrasts(evaluate,'[data-slot=alert] [data-tsrust-text]');assert(darkContrast.length>0&&darkContrast.every(value=>value>=4.5),JSON.stringify(darkContrast));
 for(const width of [390,768,1280]){await call('Emulation.setDeviceMetricsOverride',{width,height:1000,deviceScaleFactor:1,mobile:false});await settle();assert(await evaluate(`document.documentElement.scrollWidth<=innerWidth`));assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);}
 const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert(tree.some(node=>node.role?.value==='alert'));assert(tree.some(node=>node.role?.value==='status'));
 fs.writeFileSync(path.join(site,'alerts.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.equal(await evaluate(`document.querySelectorAll('[data-tsrust-node]').length`),0);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'documentation-alerts',status:'pass',checks:['four subcomponents','variant metadata','live retries','retained hide/show','polite validation without focus loss','independent scoped palette','CSS/native box-model oracle','retained route state','responsive layouts','accessibility roles','disposal'],observed,engines:['wasm','node-tsc'],minimumTextContrast:{light:Math.min(...lightContrast),dark:Math.min(...darkContrast)},timed:false}));
}
