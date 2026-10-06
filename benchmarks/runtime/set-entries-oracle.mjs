import {observeSliceTree} from './slice-observation.mjs';
export function expectedSetEntriesValues(id,n,seed,input){
 if(id!=='set_entries'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Set entry case');
 return Array.from({length:n},(_,i)=>[String(i+seed+1),String(i+seed+1),String(i+seed),String(i+seed),input,input,input==='tail'?'1':'2','0']);
}
export function expectedSetEntriesObservation(id,n,seed,input){return observeSliceTree(expectedSetEntriesValues(id,n,seed,input));}
