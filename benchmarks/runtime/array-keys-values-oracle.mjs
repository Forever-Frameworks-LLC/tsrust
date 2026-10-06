import {observeSliceTree} from './slice-observation.mjs';
export const arrayKeysValuesIds = ['array_keys_int','array_values_int'];
// Independent of keys()/values(): indices and elements are generated directly.
export function expectedArrayKeysValuesValues(id,n,seed) {
 if (!arrayKeysValuesIds.includes(id)) throw Error('Unknown array keys/values workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000) throw Error('Array keys/values case bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const mode=(i+seed)%3,base=(i*13+seed)%97,count=mode===0?0:mode===1?1:5,row=[];
  for(let at=0;at<count;at++)row.push(id==='array_keys_int'?at:base+at);
  values.push(row);
 }
 return values;
}
export function expectedArrayKeysValuesObservation(id,n,seed) { return observeSliceTree(expectedArrayKeysValuesValues(id,n,seed)); }
