import {observeSliceTree} from './slice-observation.mjs';
export function expectedArrayCallbackErrorsValues(id,n,seed,input){
 if(id!=='array_callback_errors'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Array callback error case');
 return Array.from({length:n},(_,i)=>{const v=i+seed;return [input,'1',`${v+2},${v},${v+1}`,`${v},${v+1},${v+2}`];});
}
export function expectedArrayCallbackErrorsObservation(id,n,seed,input){return observeSliceTree(expectedArrayCallbackErrorsValues(id,n,seed,input));}
