import {observeSliceTree} from './slice-observation.mjs';
export function expectedNestedOwnedValues(id,n,seed,input) {
 if(id!=='nested_function_owned_escape'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='owned')throw Error('Owned nested case');
 return Array.from({length:n},(_,i)=>{const v=i%5+seed%5+1;return [v+1,v+11,v+2,v,v+1,v+20,1,5];});
}
export function expectedNestedOwnedObservation(id,n,seed,input){return observeSliceTree(expectedNestedOwnedValues(id,n,seed,input));}
