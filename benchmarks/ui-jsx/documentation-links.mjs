import assert from 'node:assert/strict';
import fs from 'node:fs';

// Model a reader selecting the appropriate section before choosing a page.
export function documentationLink(evaluate) {
 const catalog=JSON.parse(fs.readFileSync(new URL('../../docs/target/docs-content/catalog.json',import.meta.url)));
 return async href=>{
  const exists=()=>evaluate(`!!document.querySelector('a[href=${JSON.stringify(href)}]')`);
  if(!await exists()){
   await evaluate(`(()=>{const search=document.querySelector('input[aria-label="Search documentation"]');if(search?.value){search.value='';search.dispatchEvent(new Event('input',{bubbles:true}));}})()`);
   const page=catalog.pages.find(page=>(page.route==='/'?'/docs':'/docs'+page.route)===href.split('#')[0]);assert(page,href);
   const group=page.metadata.group;const section='/docs/'+(['foundations','components','recipes','blocks','charts','themes'].includes(group)?'ui/':'')+group;
   await evaluate(`document.querySelector('nav[aria-label="Documentation sections"] a[href=${JSON.stringify(section)}]').click()`);
  }
  assert(await exists(),href);await evaluate(`document.querySelector('a[href=${JSON.stringify(href)}]').click()`);
 };
}
