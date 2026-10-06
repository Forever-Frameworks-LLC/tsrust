import {observeSliceTree} from './slice-observation.mjs';
export function expectedDeferredGenericValues(id,n,seed,input) {
 if(id!=='generic_function_deferred'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='deferred')throw Error('Deferred generic case');
 return Array.from({length:n},(_,i)=>{const v=i%5+seed%5+1;return [v,5,v+1,v+2,v+4,2,8];});
}
export function expectedDeferredGenericObservation(id,n,seed,input){return observeSliceTree(expectedDeferredGenericValues(id,n,seed,input));}
