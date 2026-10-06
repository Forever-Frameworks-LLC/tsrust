import {observeSliceTree} from './slice-observation.mjs';
export function expectedNestedDefaultValues(id,n,seed,input) {
 if(id!=='nested_function_defaults'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='defaults')throw Error('Named default case');
 return Array.from({length:n},(_,i)=>{const v=i%5+seed%5+1;return [v+2,v+5,v+7,2,v+4,v+1,5,8];});
}
export function expectedNestedDefaultObservation(id,n,seed,input){return observeSliceTree(expectedNestedDefaultValues(id,n,seed,input));}
