import {observeSliceTree} from './slice-observation.mjs';
export function expectedSetForEachValues(id,n,seed,input){
 if(id!=='set_for_each'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Set callback case');
 return Array.from({length:n},(_,i)=>[String(4*(i+seed)+6),'2',input==='tail'?'tail|':`${input}|tail|`,input==='tail'?'1':'2']);
}
export function expectedSetForEachObservation(id,n,seed,input){return observeSliceTree(expectedSetForEachValues(id,n,seed,input));}
