import {observeSliceTree} from './slice-observation.mjs';
export function expectedIterableCallSpreadValues(id,n,seed,input){
 if(id!=='iterable_call_spread'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Iterable call spread case');
 const chars=[];for(const character of input)chars.push(character);
 return Array.from({length:n},(_,i)=>{const k=i+seed;return [String(2*k+2),chars.join('|'),String(2*k+6),String(k+2),input+':'+k,'2','<a:b>'];});
}
export function expectedIterableCallSpreadObservation(id,n,seed,input){return observeSliceTree(expectedIterableCallSpreadValues(id,n,seed,input));}
