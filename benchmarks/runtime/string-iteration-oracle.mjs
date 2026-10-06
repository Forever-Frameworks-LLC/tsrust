import {observeSliceTree} from './slice-observation.mjs';
export function expectedStringIterationValues(id,n,seed,input){
 if(id!=='string_iteration'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('String iteration case');
 const characters=Array.from(input);const trace=characters.map(c=>c+'|').join('');const filtered=characters.filter(c=>c!=='é').join('');const nested=characters.map(c=>c+'a'+c+'b').join('');
 return Array.from({length:n},(_,i)=>[String(i+seed)+':'+trace,String(characters.length),filtered,nested]);
}
export function expectedStringIterationObservation(id,n,seed,input){return observeSliceTree(expectedStringIterationValues(id,n,seed,input));}
