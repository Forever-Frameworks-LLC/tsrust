import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {textContrasts} from './text-contrast.mjs';
export async function observeBadge({evaluate,call,errors,expected,site}){
 await evaluate(`window.badges=document.querySelector('[data-slot=badge-gallery]');window.liveBadge=document.querySelector('#badge-live')`);
 assert(await evaluate('!!badges&&!!liveBadge'));
 assert.deepEqual(await evaluate(`Array.from(badges.querySelectorAll('[data-slot=badge]')).slice(0,6).map(node=>node.dataset.variant)`),expected.variants);
 const observed=[];const snapshot=async()=>observed.push(await evaluate(`({variant:liveBadge.getAttribute('data-variant'),padding:parseFloat(getComputedStyle(liveBadge).paddingLeft),label:liveBadge.textContent})`));
 const click=async label=>evaluate(`Array.from(badges.querySelectorAll('button')).find(node=>node.textContent===${JSON.stringify(label)}).click()`);
 await snapshot();await click('Change badge palette');await snapshot();await click('Add build');await snapshot();await click('Toggle neutral badge');await snapshot();await click('Change badge palette');await snapshot();await click('Toggle neutral badge');await snapshot();assert.deepEqual(observed,expected.steps);
 assert(await evaluate(`document.querySelector('#badge-live')===liveBadge&&badges.querySelectorAll('button').length===3&&badges.querySelectorAll('a').length===1`));
 for(const width of [390,768,1280]){await call('Emulation.setDeviceMetricsOverride',{width,height:850,deviceScaleFactor:1,mobile:false});await evaluate('new Promise(resolve=>requestAnimationFrame(resolve))');assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));}
 const contrasts=[];
 for(let mode=0;mode<2;mode++){const ratios=await textContrasts(evaluate,'[data-slot=badge-gallery] [data-slot=badge] [data-tsrust-text]');assert(ratios.length>=6&&ratios.every(value=>value>=4.5),JSON.stringify(ratios));contrasts.push(ratios);await evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click()`);}
 await evaluate(`window.badgeLink=badges.querySelector('a');badgeLink.scrollIntoView({block:'center'});window.badgeLinkBackground=getComputedStyle(badgeLink).backgroundColor`);
 const point=await evaluate(`(()=>{const r=badgeLink.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2};})()`);await call('Input.dispatchMouseEvent',{type:'mouseMoved',...point});assert(await evaluate(`badgeLink.matches(':hover')`));assert.equal(await evaluate('getComputedStyle(badgeLink).backgroundColor'),await evaluate(`(()=>{const probe=document.createElement('span');probe.style.backgroundColor=getComputedStyle(badgeLink).getPropertyValue('--ui-muted');document.body.append(probe);const color=getComputedStyle(probe).backgroundColor;probe.remove();return color;})()`));const hoverContrast=await textContrasts(evaluate,'[data-slot=badge-gallery] a [data-tsrust-text]');assert(hoverContrast.length>0&&hoverContrast.every(value=>value>=4.5),JSON.stringify(hoverContrast));
 await evaluate(`window.actionBadge=Array.from(badges.querySelectorAll('button')).find(node=>node.textContent==='Add build');actionBadge.focus()`);assert(await evaluate(`document.activeElement===actionBadge&&getComputedStyle(actionBadge).boxShadow!=='none'`));
 const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert(tree.some(node=>node.role?.value==='button'&&node.name?.value==='Add build'));assert(tree.some(node=>node.role?.value==='link'&&node.name?.value==='Read the Badge API'));
 fs.writeFileSync(path.join(site,'badge.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.deepEqual(errors,[]);console.log(JSON.stringify({case:'badge',status:'pass',observed,contrasts,timed:false}));
}
