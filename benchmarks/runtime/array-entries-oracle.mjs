import {observeSliceTree} from './slice-observation.mjs';
export const arrayEntriesIds=['array_entries_string','array_entries_records'];
// Derive the retained cells without entries(), Array.from() or copying record owners.
export function expectedArrayEntriesValues(id,n,seed,input) {
 if(!arrayEntriesIds.includes(id))throw Error('Unknown array entries workload');
 if(!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Array entries bounds');
 const output=[];
 for(let i=0;i<n;i++){const base=(i*13+seed)%97,text=input.repeat(base%3+1);output.push(id==='array_entries_string'?[String(base+5),input,text,'1',input,'2','2','true']:['0','1',text,`${base},${base+1}`,input,`${base+9},${base+1}`,'2']);}
 return output;
}
export function expectedArrayEntriesObservation(id,n,seed,input){return observeSliceTree(expectedArrayEntriesValues(id,n,seed,input));}
