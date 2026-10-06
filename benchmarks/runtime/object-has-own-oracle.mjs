import {observeSliceTree} from './slice-observation.mjs';
export const objectHasOwnIds=['object_has_own'];
export function expectedObjectHasOwnValues(id,n,seed,input){
 if(!objectHasOwnIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Object membership case');
 const result=[];
 for(let i=0;i<n;i++){const p=(i+seed)%4;result.push(['true','false','false',String(p<2),'true','false',String(p<input.length),'true','true']);}
 return result;
}
export function expectedObjectHasOwnObservation(id,n,seed,input){return observeSliceTree(expectedObjectHasOwnValues(id,n,seed,input));}
