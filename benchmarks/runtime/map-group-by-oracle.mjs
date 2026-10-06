import {observeSliceTree} from './slice-observation.mjs';
export function expectedMapGroupByValues(id,n,seed,input){
 if(id!=='map_group_by'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Map grouping case');
 const alpha=[];const other=[];for(const character of input){if(character==='a')alpha.push(character);else other.push(character);}
 const count=Number(alpha.length>0)+Number(other.length>0);
 return Array.from({length:n},(_,i)=>{const k=i+seed;return ['2',`${k%2},${(k+1)%2}`,`${k},${k+2}`,`${k+1},${k+3}`,'10',`${k},${k+1},${k+2},${k+3},${k+4}`,String(count),alpha.join('|'),other.join('|')];});
}
export function expectedMapGroupByObservation(id,n,seed,input){return observeSliceTree(expectedMapGroupByValues(id,n,seed,input));}
