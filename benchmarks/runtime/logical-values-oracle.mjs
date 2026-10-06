import {observeSliceTree} from './slice-observation.mjs';
export function expectedLogicalValueValues(id,n,seed) {
 if(id!=='logical_nullable_string_values'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000)throw Error('Bounded logical value case required');
 const output=[];let calls=0;
 for(let i=0;i<n;i++){
  const mode=(i+seed)%4;
  let first;
  if(mode<2){calls++;first='Guest';}else{first=mode===2?'é':'😀';}
  output.push(first,String(calls));
  let second;
  if(mode===0)second='undefined';else if(mode===1)second='';else{calls++;second='YES';}
  output.push(second,String(calls));
 }
 return output;
}
export function expectedLogicalValueObservation(id,n,seed){return observeSliceTree(expectedLogicalValueValues(id,n,seed));}
