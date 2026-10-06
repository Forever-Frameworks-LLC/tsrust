import {observeSliceTree} from './slice-observation.mjs';
export const arrayFromIds = ['copy_int','map_int','string_characters','map_string'].map(value=>'array_from_'+value);
// Independent of Array.from: integer ranges are generated directly and string
// characters come from code-point iteration by spread.
export function expectedArrayFromValues(id,n,seed,input) {
 if (!arrayFromIds.includes(id)) throw Error('Unknown Array.from workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string') throw Error('Array.from case bounds');
 const variant=id.slice('array_from_'.length),values=[];
 for(let i=0;i<n;i++) {
  const mode=(i+seed)%3,base=(i*13+seed)%97,count=mode===0?0:mode===1?1:5;
  if(variant==='copy_int'){const row=[];for(let at=0;at<count;at++)row.push(base+at);values.push(row);}
  else if(variant==='map_int'){const row=[];for(let at=0;at<count;at++)row.push((base+at)*2+at);values.push(row);}
  else {let text='';for(let at=0;at<mode;at++)text+=input;const letters=[...text];values.push(variant==='string_characters'?letters:letters.map((letter,at)=>letter+at));}
 }
 return values;
}
export function expectedArrayFromObservation(id,n,seed,input) { return observeSliceTree(expectedArrayFromValues(id,n,seed,input)); }
