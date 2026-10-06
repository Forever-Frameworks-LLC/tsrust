import {observeSliceTree} from './slice-observation.mjs';
export function expectedNestedResultValues(id,n,seed,input) {
 if(id!=='nested_function_results'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='infer')throw Error('Named result case');
 return Array.from({length:n},(_,i)=>{const v=i%5+seed%5+1;return [2*v,v+3,v+2,v,v+1,5];});
}
export function expectedNestedResultObservation(id,n,seed,input){return observeSliceTree(expectedNestedResultValues(id,n,seed,input));}
