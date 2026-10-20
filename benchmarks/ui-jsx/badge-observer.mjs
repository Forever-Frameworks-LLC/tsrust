import assert from 'node:assert/strict';
export async function observeBadge({evaluate,call,errors,expected}){
 await evaluate(`window.badges=document.querySelector('[data-slot=badge-gallery]');window.liveBadge=document.querySelector('#badge-live')`);
 assert(await evaluate('!!badges&&!!liveBadge'));
 assert.deepEqual(await evaluate(`Array.from(badges.querySelectorAll('[data-slot=badge]')).slice(0,6).map(node=>node.dataset.variant)`),expected.variants);
 const observed=[];const snapshot=async()=>observed.push(await evaluate(`({variant:liveBadge.getAttribute('data-variant'),padding:parseFloat(getComputedStyle(liveBadge).paddingLeft),label:liveBadge.textContent})`));
 const click=async label=>evaluate(`Array.from(badges.querySelectorAll('button')).find(node=>node.textContent===${JSON.stringify(label)}).click()`);
 await snapshot();await click('Change badge palette');await snapshot();await click('Add build');await snapshot();await click('Toggle neutral badge');await snapshot();await click('Change badge palette');await snapshot();await click('Toggle neutral badge');await snapshot();assert.deepEqual(observed,expected.steps);
 assert(await evaluate(`document.querySelector('#badge-live')===liveBadge&&badges.querySelectorAll('button').length===3&&badges.querySelectorAll('a').length===1`));
 for(const width of [390,768,1280]){await call('Emulation.setDeviceMetricsOverride',{width,height:850,deviceScaleFactor:1,mobile:false});await evaluate('new Promise(resolve=>requestAnimationFrame(resolve))');assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'));}
 const tree=(await call('Accessibility.getFullAXTree')).nodes.filter(node=>!node.ignored);assert(tree.some(node=>node.role?.value==='button'&&node.name?.value==='Add build'));assert(tree.some(node=>node.role?.value==='link'&&node.name?.value==='Read the Badge API'));
 assert.deepEqual(await evaluate('window.ui.mounts.map(mount=>mount.error())'),[null]);await evaluate('window.ui.dispose()');assert.deepEqual(errors,[]);console.log(JSON.stringify({case:'badge',status:'pass',observed,timed:false}));
}
