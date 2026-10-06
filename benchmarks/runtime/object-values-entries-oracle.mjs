import {observeSliceTree} from './slice-observation.mjs';
export const objectValuesIds = ['object_values_entries'];
// Independent of Object.values and Object.entries: the two constructor orders
// are written out, and each cell follows from the order of that iteration.
export function expectedObjectValuesValues(id,n,seed,input) {
 if (!objectValuesIds.includes(id)) throw Error('Unknown object values workload');
 if (!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string') throw Error('Object values case bounds');
 const values=[];
 for(let i=0;i<n;i++) {
  const base=(i*13+seed)%97,first=(i+seed)%2===0;
  values.push(first
   ?['T'+input,''+base,'true|'+base+'|'+input,''+(base-i%5),'gamma=true;alpha='+base+';beta='+input+';','2']
   :[input+'F',''+base,input+'|false|'+base,''+(base-i%5),'beta='+input+';gamma=false;alpha='+base+';','2']);
 }
 return values;
}
export function expectedObjectValuesObservation(id,n,seed,input) { return observeSliceTree(expectedObjectValuesValues(id,n,seed,input)); }
