import {observeSliceTree} from './slice-observation.mjs';
export function expectedSetsValues(id,n,seed,input){
 if(id!=='set_values'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Set case');
 return Array.from({length:n},(_,i)=>['2','true',`${i+seed+1},${i+seed}`,input==='tail'?'1':'2','0',`${i+seed+1}:${i+seed}:`,input==='tail'?'tail':`${input}|tail`]);
}
export function expectedSetsObservation(id,n,seed,input){return observeSliceTree(expectedSetsValues(id,n,seed,input));}
