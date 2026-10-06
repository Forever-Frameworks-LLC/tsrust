import {observeSliceTree} from './slice-observation.mjs';
export function expectedGenericResultValues(id,n,seed,input) {
 if(id!=='generic_function_results'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='results')throw Error('Generic result case');
 return Array.from({length:n},(_,i)=>{const v=i%5+seed%5+1;return [v,5,v+1,5,2,7];});
}
export function expectedGenericResultObservation(id,n,seed,input){return observeSliceTree(expectedGenericResultValues(id,n,seed,input));}
