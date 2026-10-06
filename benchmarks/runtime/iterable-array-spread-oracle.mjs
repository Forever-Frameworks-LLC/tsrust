import {observeSliceTree} from './slice-observation.mjs';
export function expectedIterableArraySpreadValues(id,n,seed,input){
 if(id!=='iterable_array_spread'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Iterable array spread case');
 const chars=[];for(const character of input+'é🙂')chars.push(character);
 return Array.from({length:n},(_,i)=>{const k=i+seed;const pairs=input==='tail'?[[input,k+1]]:[[input,k],['tail',k+1]];return [[0,k+2,k,k+3].join(','),chars.join('|'),String(pairs.length),pairs.map(([key,value])=>key+':'+value).join('|'),input+':'+k];});
}
export function expectedIterableArraySpreadObservation(id,n,seed,input){return observeSliceTree(expectedIterableArraySpreadValues(id,n,seed,input));}
