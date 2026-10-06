import {observeSliceTree} from './slice-observation.mjs';
export function expectedMapsValues(id,n,seed,input){
 if(id!=='map_values'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Map case');
 return Array.from({length:n},()=>['3','1','4','true','true','false','0','2',input]);
}
export function expectedMapsObservation(id,n,seed,input){return observeSliceTree(expectedMapsValues(id,n,seed,input));}
