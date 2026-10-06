import {observeSliceTree} from './slice-observation.mjs';
export function expectedArrayFromCollectionsValues(id,n,seed,input){
 if(id!=='array_from_collections'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Array.from collections case');
 return Array.from({length:n},(_,i)=>{const k=i+seed;const pairs=input==='tail'?[[input,k+1]]:[[input,k],['tail',k+1]];return [[k+2,k].join(','),[(k+2)*10,k*10+1].join(','),String(pairs.length),pairs.map(([key,value],index)=>key+':'+(value+index)).join('|'),input+':'+k];});
}
export function expectedArrayFromCollectionsObservation(id,n,seed,input){return observeSliceTree(expectedArrayFromCollectionsValues(id,n,seed,input));}
