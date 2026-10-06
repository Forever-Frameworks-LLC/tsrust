import {observeSliceTree} from './slice-observation.mjs';
export function expectedEnumsValues(id,n,seed,input){
 if(id!=='enum_values'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Enum case');
 return Array.from({length:n},(_,i)=>{const value=[0,4,5][(i+seed)%3];return [String(value),value===0?'s':'long',input+value];});
}
export function expectedEnumsObservation(id,n,seed,input){return observeSliceTree(expectedEnumsValues(id,n,seed,input));}
