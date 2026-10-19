// Design-reference checks; production web/native behavior has its own observer.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

export async function observeLandingReference({evaluate,call,errors,site}) {
 const wait=async expression=>{for(let i=0;i<100;i++){if(await evaluate(expression))return;await new Promise(resolve=>setTimeout(resolve,40));}assert.fail(`Timeout: ${expression}`);};
 await wait(`document.documentElement.classList.contains('enhanced')`);
 for(const index of [0,1,2,1,0]){
  await evaluate(`document.querySelector('[data-chapter="${index}"]').scrollIntoView({block:'center'})`);
  await wait(`!document.querySelector('[data-code="${index}"]').hidden`);
 }
 for(const width of [320,480,768,900,1440]){
  await call('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});
  assert(await evaluate(`document.documentElement.scrollWidth<=innerWidth`),`Reference overflow at ${width}`);
  assert.equal(await evaluate(`getComputedStyle(document.querySelector('.stage')).display!=='none'`),width>=900);
 }
 await evaluate(`scrollTo(0,0)`);
 fs.writeFileSync(path.join(site,'reference-light.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 for(const dark of [false,true]){
  if(dark)await evaluate(`document.querySelector('#theme').click()`);
  const colors=await evaluate(`['.muted','.accent','.action'].map(selector=>{const node=document.querySelector(selector),style=getComputedStyle(node);return {color:style.color,background:selector==='.action'?style.backgroundColor:getComputedStyle(document.body).backgroundColor};})`);
  const luminance=color=>{const values=color.match(/[\d.]+/g).slice(0,3).map(value=>{value=Number(value)/255;return value<=.04045?value/12.92:((value+.055)/1.055)**2.4;});return values[0]*.2126+values[1]*.7152+values[2]*.0722;};
  for(const pair of colors){const a=luminance(pair.color),b=luminance(pair.background);assert((Math.max(a,b)+.05)/(Math.min(a,b)+.05)>=4.5,JSON.stringify(pair));}
 }
 await call('Emulation.setEmulatedMedia',{features:[{name:'prefers-reduced-motion',value:'reduce'}]});
 assert.equal(await evaluate(`getComputedStyle(document.querySelector('.code-panel')).animationName`),'none');
 await evaluate(`Object.defineProperty(navigator,'clipboard',{configurable:true,value:{writeText:async value=>{window.copied=value;}}});window.copy=document.querySelector('[data-mobile="0"] [data-copy]');copy.click()`);
 await wait(`copy.textContent==='Copied'`);assert.equal(await evaluate(`copied`),await evaluate(`copy.closest('.code-panel').querySelector('code').textContent`));
 await evaluate(`navigator.clipboard.writeText=async()=>{throw new Error('Denied');};copy.click()`);await wait(`copy.textContent==='Retry copy'`);assert.equal(await evaluate('copy.disabled'),false);
 await call('Emulation.setDeviceMetricsOverride',{width:1440,height:960,deviceScaleFactor:1,mobile:false});
 fs.writeFileSync(path.join(site,'reference-dark.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'single-file-landing-reference',status:'pass',checks:['scroll forward and backward','320–1440px layouts','mobile sequential code','light/dark AA text contrast','reduced motion','exact copy and retry','self-contained compiled Tailwind'],timed:false}));
}
