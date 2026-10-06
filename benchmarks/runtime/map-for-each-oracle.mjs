import {observeSliceTree} from './slice-observation.mjs';
export function expectedMapForEachValues(id,n,seed,input){
 if(id!=='map_for_each'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Map callback case');
 return Array.from({length:n},(_,i)=>[String(2*(i+seed)+12),'2',input==='tail'?'tail:updated|':`${input}:updated|tail:last|`,input==='tail'?'1':'2']);
}
export function expectedMapForEachObservation(id,n,seed,input){return observeSliceTree(expectedMapForEachValues(id,n,seed,input));}
