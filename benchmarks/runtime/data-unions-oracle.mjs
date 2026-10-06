import {observeSliceTree} from './slice-observation.mjs';
export const dataUnionsIds=['data_union_values'];
// Derive retained cells from mode/base arithmetic without union values or cloning.
export function expectedDataUnionsValues(id,n,seed,input){
 if(!dataUnionsIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Data unions bounds');
 const output=[];
 for(let i=0;i<n;i++){const base=(i*13+seed)%97,mode=(i+seed)%4,word=input.repeat(base%3+1),owner=mode%2===0?input:`${base},${base+1}`,value=mode===0?String(base):mode===1?word:mode===2?'false':'null',label=mode===0?'number':mode===1?'string':mode===2?'boolean':'object',description=mode===1?word.toUpperCase():mode===3?'missing':value;output.push([value,label,owner,String(base+5),description,owner]);}
 return output;
}
export function expectedDataUnionsObservation(id,n,seed,input){return observeSliceTree(expectedDataUnionsValues(id,n,seed,input));}
