import {expectedValues} from './call-spread-values-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
export const arrayLiteralSpreadIds = ['dynamic_int','multiple_mixed_int','boolean_pack','string_owned','nested_owned','record_owned','nullish_packs','number','callable','source_order'].map(value=>'array_literal_spread_'+value);
export function expectedArrayLiteralSpreadObservation(id,n,seed,input) {
 if (!arrayLiteralSpreadIds.includes(id)) throw Error('Unknown array-literal spread workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000) throw Error('Array-literal spread case bounds');
 const variant=id.slice('array_literal_spread_'.length);
 let values=[];
 if (!['number','callable','source_order'].includes(variant)) values=expectedValues('call_spread_'+variant,n,seed,input);
 else for(let i=0;i<n;i++) {
  const base=(i+seed)%97;
  if(variant==='number') {const count=(i+seed)%3;values.push(count===0?[1.5,-2.5]:count===1?[1.5,0,-2.5]:[1.5,0,-0,-2.5]);}
  if(variant==='callable') values.push([base,base+1,base+2]);
  if(variant==='source_order') values.push([base,base+1,3,base,base+1,9],[base,base+1,9]);
 }
 return observeSliceTree(values,{numbers:variant==='number'});
}
