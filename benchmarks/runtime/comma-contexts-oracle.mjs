import {observeSliceTree} from './slice-observation.mjs';
export function expectedCommaContextsValues(id,n,seed,input){
 if(id!=='comma_contexts'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Comma context case');
 return Array.from({length:n},(_,i)=>{const j=seed+i;return [[j+2,8,j].join(','),'1,2,3,4,5,6,7',String(i%2===0?j+1:0),i%2===0?'YES':'number','true',input];});
}
export function expectedCommaContextsObservation(id,n,seed,input){return observeSliceTree(expectedCommaContextsValues(id,n,seed,input));}
