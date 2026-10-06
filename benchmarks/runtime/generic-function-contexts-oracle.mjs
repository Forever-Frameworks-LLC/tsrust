import {observeSliceTree} from './slice-observation.mjs';
export function expectedGenericContextValues(id,n,seed,input) {
 if(id!=='generic_function_contexts'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='context')throw Error('Generic context case');
 return Array.from({length:n},(_,i)=>{const v=i%5+seed%5+1;return [v,5,v+1,v+3,1,4,7];});
}
export function expectedGenericContextObservation(id,n,seed,input){return observeSliceTree(expectedGenericContextValues(id,n,seed,input));}
