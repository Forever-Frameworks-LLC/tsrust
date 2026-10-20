import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
export async function observeResponsiveNavigation({evaluate,call,errors,site}) {
 const frame=()=>evaluate('new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)))');
 const visible=selector=>evaluate(`(()=>{const n=document.querySelector(${JSON.stringify(selector)});return !!n&&n.getBoundingClientRect().width>0&&n.getBoundingClientRect().height>0;})()`);
 const click=async selector=>{
  assert(await visible(selector),`Visible click target: ${selector}`);
  const point=await evaluate(`(()=>{const n=document.querySelector(${JSON.stringify(selector)});n.scrollIntoView({block:'nearest'});const r=n.getBoundingClientRect();return {x:r.x+r.width/2,y:r.y+r.height/2};})()`);
  await call('Input.dispatchMouseEvent',{type:'mousePressed',button:'left',clickCount:1,...point});await call('Input.dispatchMouseEvent',{type:'mouseReleased',button:'left',clickCount:1,...point});await frame();
 };
 const origin=await evaluate('location.origin');
 await call('Page.navigate',{url:origin+'/docs/getting-started/project-structure'});
 for(let i=0;i<200&&!await evaluate(`Boolean(window.ui)&&!!document.querySelector('#sidebar')`);i++)await new Promise(r=>setTimeout(r,50));
 await call('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false});await frame();
 const selection=await evaluate(`Array.from(document.querySelectorAll('#sidebar a')).filter(a=>getComputedStyle(a).backgroundColor!=='rgba(0, 0, 0, 0)').map(a=>({href:a.getAttribute('href'),color:getComputedStyle(a).color}))`);
 assert.deepEqual(selection.map(x=>x.href),['/docs/getting-started/project-structure']);
 assert(selection[0].color==='rgb(169, 56, 25)');
 assert.equal(await evaluate(`Array.from(document.querySelectorAll('nav[aria-label="Documentation sections"] a')).filter(a=>getComputedStyle(a).backgroundColor!=='rgba(0, 0, 0, 0)').map(a=>a.textContent).join(',')`),'Getting started');
 for(const width of [390,768,1023,1024,1440]) {
  await call('Emulation.setDeviceMetricsOverride',{width,height:800,deviceScaleFactor:1,mobile:false});await frame();
  assert(await evaluate('document.documentElement.scrollWidth<=innerWidth'),`Viewport overflow ${width}`);
  if(width<1024){
   assert(!await visible('#sidebar'));assert(await visible('[data-slot=route-page]'));
   const rect=await evaluate(`(()=>{const r=document.querySelector('[data-slot=route-page]').getBoundingClientRect();return {width:r.width,height:r.height,top:r.top}})()`);
   assert(rect.width>=width-2&&rect.height>=600&&rect.top<=130,JSON.stringify({width,rect}));
   await click('button[aria-label="Browse documentation"]');assert(await visible('#sidebar'));assert(!await visible('[data-slot=route-page]'));
   assert.equal(await evaluate(`document.querySelector('button[aria-label="Browse documentation"]').getAttribute('aria-expanded')`),'true');
   await click('button[aria-label="Browse documentation"]');assert(!await visible('#sidebar'));assert(await visible('[data-slot=route-page]'));
  }else{assert(await visible('#sidebar'));assert(!await visible('button[aria-label="Browse documentation"]'));}
  fs.writeFileSync(path.join(site,`navigation-${width}.png`),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 }
 await call('Emulation.setDeviceMetricsOverride',{width:390,height:800,deviceScaleFactor:1,mobile:false});await frame();
 await click('button[aria-label="Browse documentation"]');await click('#sidebar a[href="/docs/getting-started/installation"]');
 assert.equal(await evaluate('location.pathname'),'/docs/getting-started/installation');assert(!await visible('#sidebar'));assert(await visible('[data-slot=route-page]'));
 await evaluate('history.back()');await frame();
 await click('button[aria-label="Toggle light and dark theme"]');
 await call('Emulation.setDeviceMetricsOverride',{width:1440,height:900,deviceScaleFactor:1,mobile:false});await frame();
 const dark=await evaluate(`Array.from(document.querySelectorAll('#sidebar a')).filter(a=>getComputedStyle(a).backgroundColor!=='rgba(0, 0, 0, 0)').map(a=>({href:a.getAttribute('href'),color:getComputedStyle(a).color}))`);
 assert.deepEqual(dark.map(x=>x.href),['/docs/getting-started/project-structure']);assert.equal(dark[0].color,'rgb(255, 173, 138)');
 assert.deepEqual(await evaluate('window.ui.mounts.map(m=>m.error())'),[null]);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'docs-responsive-navigation',status:'pass',widths:[390,768,1023,1024,1440],checks:['exact sidebar selection','separate active section','light/dark selection','visible mouse navigation','collapsed mobile controls','mobile disclosure and auto-close','full-width article','resize restoration','no viewport overflow','no mount errors'],timed:false}));
}
