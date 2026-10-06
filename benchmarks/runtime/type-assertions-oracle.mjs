import {observeSliceTree} from './slice-observation.mjs';
export function expectedTypeAssertionsValues(id,n,seed,input){
 if(id!=='type_assertions'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Type assertions case');
 return Array.from({length:n},(_,i)=>{const value=(i*13+seed)%97;return [String(value),String(value+0.5),String(value),input+value];});
}
export function expectedTypeAssertionsObservation(id,n,seed,input){return observeSliceTree(expectedTypeAssertionsValues(id,n,seed,input));}
