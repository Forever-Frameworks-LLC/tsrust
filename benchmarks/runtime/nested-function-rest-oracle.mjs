import {observeSliceTree} from './slice-observation.mjs';
export function expectedNestedRestValues(id,n,seed,input) {
 if(id!=='nested_function_rest'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='rest')throw Error('Named rest case');
 return Array.from({length:n},(_,i)=>{const v=i%5+seed%5+1;return [v,v+5,v+9,v,v+2,4,4];});
}
export function expectedNestedRestObservation(id,n,seed,input){return observeSliceTree(expectedNestedRestValues(id,n,seed,input));}
