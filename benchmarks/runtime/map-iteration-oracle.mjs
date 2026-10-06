import {observeSliceTree} from './slice-observation.mjs';
export function expectedMapIterationValues(id,n,seed,input){
 if(id!=='map_iteration'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Map iteration case');
 return Array.from({length:n},(_,i)=>{const k=i+seed;const pairs=input==='tail'?[[input,k+1]]:[[input,k],['tail',k+1]];const text=pairs.map(([key,value])=>key+':'+value+'|').join('');return[text,text,String(pairs.length+(input==='last'?0:1))];});
}
export function expectedMapIterationObservation(id,n,seed,input){return observeSliceTree(expectedMapIterationValues(id,n,seed,input));}
