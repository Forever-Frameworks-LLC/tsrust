import {observeSliceTree} from './slice-observation.mjs';
export function expectedSetRelationsValues(id,n,seed,input){
 if(id!=='set_relations'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Set relation case');
 return Array.from({length:n},()=>['true','true','true','false',input==='tail'?'false':'true',input==='tail'?'true':'false']);
}
export function expectedSetRelationsObservation(id,n,seed,input){return observeSliceTree(expectedSetRelationsValues(id,n,seed,input));}
