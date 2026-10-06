import {observeSliceTree} from './slice-observation.mjs';
export const tupleTypesIds=['tuple_types_values'];
// Derive retained scalar cells directly; do not construct, copy or map tuple owners.
export function expectedTupleTypesValues(id,n,seed,input) {
 if(!tupleTypesIds.includes(id))throw Error('Unknown tuple types workload');
 if(!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Tuple types bounds');
 return Array.from({length:n},(_,i)=>{const base=(i*13+seed)%97,text=input.repeat(base%3+1);return [String(base+1),input,String(base),text,String(base+1),text,'2','true'];});
}
export function expectedTupleTypesObservation(id,n,seed,input){return observeSliceTree(expectedTupleTypesValues(id,n,seed,input));}
