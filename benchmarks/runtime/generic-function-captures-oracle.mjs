import {observeSliceTree} from './slice-observation.mjs';
export function expectedCapturedGenericValues(id,n,seed,input) {
 if(id!=='generic_function_captures'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='captures')throw Error('Captured generic case');
 return Array.from({length:n},(_,i)=>{const v=i%5+seed%5+1;return [v+1,v+11,v+2,v,5,v+3,3,v+12,8];});
}
export function expectedCapturedGenericObservation(id,n,seed,input){return observeSliceTree(expectedCapturedGenericValues(id,n,seed,input));}
