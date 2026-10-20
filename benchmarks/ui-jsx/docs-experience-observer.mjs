import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';

export async function observeDocsExperience({evaluate,call,errors,site}) {
 const wait=async expression=>{for(let i=0;i<120;i++){if(await evaluate(expression))return;await new Promise(resolve=>setTimeout(resolve,30));}assert.fail(expression);};
 const click=async href=>{await evaluate(`document.querySelector('a[href=${JSON.stringify(href)}]').click()`);await wait(`location.pathname===${JSON.stringify(href.split('#')[0])}&&location.hash===${JSON.stringify(href.includes('#')?'#'+href.split('#')[1]:'')}`);};
 const group=async name=>{await evaluate(`document.querySelector('nav[aria-label="Documentation sections"] a[href=${JSON.stringify('/docs/'+name)}]').click()`);await wait(`location.pathname===${JSON.stringify('/docs/'+name)}`);};
 const theme=()=>evaluate(`document.querySelector('button[aria-label="Toggle light and dark theme"]').click()`);
 const modeColors=()=>evaluate(`Object.fromEntries(['keyword','string','number','type'].map(kind=>[kind,Array.from(document.querySelectorAll('[data-slot="'+kind+'"]')).filter(x=>x.getClientRects().length).map(x=>getComputedStyle(x).color)[0]??null]))`);
 const contrast=()=>evaluate(`(()=>{const canvas=document.createElement('canvas');canvas.width=canvas.height=1;const context=canvas.getContext('2d',{willReadFrequently:true});function rgba(color){context.clearRect(0,0,1,1);context.fillStyle=color;context.fillRect(0,0,1,1);return [...context.getImageData(0,0,1,1).data].map((value,index)=>index===3?value/255:value);}function over(top,bottom){return top.slice(0,3).map((value,index)=>value*top[3]+bottom[index]*(1-top[3]));}function luminance(rgb){return rgb.map(value=>{value/=255;return value<=.04045?value/12.92:((value+.055)/1.055)**2.4;}).reduce((sum,value,index)=>sum+value*[.2126,.7152,.0722][index],0);}return Array.from(document.querySelectorAll('[data-slot=keyword],[data-slot=string],[data-slot=number],[data-slot=type]')).filter(node=>node.getClientRects().length).slice(0,100).map(node=>{let ancestors=[];for(let parent=node;parent;parent=parent.parentElement)ancestors.push(parent);let background=[255,255,255];for(const parent of ancestors.reverse())background=over(rgba(getComputedStyle(parent).backgroundColor),background);const a=luminance(background),b=luminance(over(rgba(getComputedStyle(node).color),background));return (Math.max(a,b)+.05)/(Math.min(a,b)+.05);});})()`);
 const state=()=>evaluate('window.ui.mounts.map(m=>m.error())');
 await call('Emulation.setDeviceMetricsOverride',{width:1440,height:1000,deviceScaleFactor:1,mobile:false});
 await evaluate(`window.home=document.querySelector('#home-workspace-preview');window.field=home.querySelector('input');field.value='Workbench';field.dispatchEvent(new Event('input',{bubbles:true}));home.querySelector('button').click()`);
 assert(await evaluate(`home.textContent.includes('Saved Workbench')`));
 assert.equal(await evaluate(`Array.from(home.querySelectorAll('*')).some(x=>getComputedStyle(x).overflowY==='auto')`),false,'Ordinary previews must not introduce scroll panes');
 await call('Input.dispatchMouseEvent',{type:'mouseWheel',x:1100,y:700,deltaX:0,deltaY:900});await wait(`document.querySelector('#landing').scrollTop>500`);const landingScroll=await evaluate(`document.querySelector('#landing').scrollTop`);
 await click('/docs');await group('ui/components');
 await evaluate(`window.componentNav=document.querySelector('[aria-label="Components pages"]');componentNav.scrollTop=componentNav.scrollHeight`);const sidebarScroll=await evaluate('componentNav.scrollTop');assert(sidebarScroll>500);
 await click('/docs/ui/components/textarea');
 assert.equal(await evaluate(`document.querySelector('nav[aria-label="Documentation sections"] a[href="/docs/ui/components"]').getAttribute('aria-current')`),'location');
 assert.equal(await evaluate(`document.querySelector('[aria-label="Components pages"] a[href="/docs/ui/components/textarea"]').getAttribute('aria-current')`),'page');
 const selected=await evaluate(`(()=>{const a=document.querySelector('[aria-label="Components pages"] a[aria-current=page]');return {color:getComputedStyle(a).color,primary:getComputedStyle(document.querySelector('#site')).getPropertyValue('--ui-primary')};})()`);assert(selected.color.includes('169, 56, 25'));
 await group('ui/themes');await group('ui/components');assert.equal(await evaluate('componentNav.scrollTop'),sidebarScroll,'Each section retains its sidebar offset');
 await group('standard-library');await click('/docs/standard-library/date');
 await evaluate(`window.article=document.querySelector('[data-slot=route-page]');article.scrollTop=1800`);assert.equal(await evaluate('article.scrollTop'),1800);
 await click('/docs/standard-library/decimal');await evaluate(`document.querySelector('[data-slot=route-page]').scrollTop=600`);
 await evaluate('history.back()');await wait(`location.pathname==='/docs/standard-library/date'`);assert(await evaluate(`document.querySelector('[data-slot=route-page]')===article&&article.scrollTop===1800`));
 await evaluate('history.forward()');await wait(`location.pathname==='/docs/standard-library/decimal'`);assert.equal(await evaluate(`document.querySelector('[data-slot=route-page]').scrollTop`),600);
 await group('ui/foundations');await click('/docs/ui/foundations/state');
 await evaluate(`window.stateArticle=document.querySelector('[data-slot=route-page]');stateArticle.scrollTop=150;window.fragment=document.querySelector('nav[aria-label="On this page"] a:last-child').getAttribute('href');document.querySelector('nav[aria-label="On this page"] a:last-child').click()`);await wait('location.hash.length>0');
 await evaluate(`stateArticle.scrollTop=Math.min(400,stateArticle.scrollHeight-stateArticle.clientHeight)`);const fragmentScroll=await evaluate('stateArticle.scrollTop');assert(fragmentScroll>100);
 await click('/docs/ui/foundations/theming');await evaluate('history.back()');await wait('location.hash.length>0');assert.equal(await evaluate('stateArticle.scrollTop'),fragmentScroll,'Returning to a fragment restores the reading position instead of jumping to its heading');
 await click('/');assert(await evaluate(`document.querySelector('#home-workspace-preview')===home&&home.querySelector('input')===field&&field.value==='Workbench'`));assert.equal(await evaluate(`document.querySelector('#landing').scrollTop`),landingScroll);
 await evaluate(`Array.from(document.querySelectorAll('#landing button')).find(x=>x.textContent==='View source').click()`);
 const light=await modeColors();assert(light.keyword&&light.string&&light.number);assert(new Set(Object.values(light).filter(Boolean)).size>=3);const lightContrast=await contrast();assert(lightContrast.length&&lightContrast.every(value=>value>=4.5));
 await theme();const dark=await modeColors();assert(dark.keyword!==light.keyword&&dark.string!==light.string);const darkContrast=await contrast();assert(darkContrast.length&&darkContrast.every(value=>value>=4.5));await theme();assert.deepEqual(await modeColors(),light);
 await click('/docs');const prose=await evaluate(`Array.from(document.querySelectorAll('[data-slot=route-page] p a')).map(x=>getComputedStyle(x).color)`);assert(prose.length>0&&prose.every(color=>color==='rgb(169, 56, 25)'));
 await theme();const darkLinks=await evaluate(`Array.from(document.querySelectorAll('[data-slot=route-page] p a')).map(x=>getComputedStyle(x).color)`);assert(darkLinks.every(color=>color==='rgb(255, 173, 138)'));await theme();
 for(const width of [390,768,1440]){await call('Emulation.setDeviceMetricsOverride',{width,height:900,deviceScaleFactor:1,mobile:false});assert(await evaluate(`document.documentElement.scrollWidth<=innerWidth`),`Outer page overflow at ${width}`);assert.deepEqual(await state(),[null]);}
 await call('Emulation.setDeviceMetricsOverride',{width:1440,height:1000,deviceScaleFactor:1,mobile:false});
 fs.writeFileSync(path.join(site,'experience-light.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));await theme();fs.writeFileSync(path.join(site,'experience-dark.png'),Buffer.from((await call('Page.captureScreenshot',{format:'png'})).data,'base64'));
 const map=JSON.parse(fs.readFileSync(new URL('../../docs/plans/documentation-reference-coverage.json',import.meta.url)));const catalog=JSON.parse(fs.readFileSync(new URL('../../docs/target/docs-content/catalog.json',import.meta.url)));
 for(const entry of Object.values(map.receivers)){assert(catalog.pages.some(page=>'/docs'+page.route===entry.route));}
 assert.deepEqual(await state(),[null]);await evaluate('window.ui.dispose()');assert.equal(await evaluate(`document.querySelectorAll('[data-tsrust-node]').length`),0);assert.deepEqual(errors,[]);
 console.log(JSON.stringify({case:'documentation-experience',status:'pass',pages:catalog.pages.length,publicReceivers:Object.keys(map.receivers).length,checks:['long article and landing restoration','back/forward and fragment restoration','independent section scroll','ancestor and exact selection','live prose/link and syntax colors','AA syntax contrast in both modes','unbounded component previews','retained workspace state','390/768/1440 viewports','disposal'],light,dark,minimumSyntaxContrast:{light:Math.min(...lightContrast),dark:Math.min(...darkContrast)},timed:false}));
}
