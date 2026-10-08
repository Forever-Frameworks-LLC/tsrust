import {observeSliceTree} from './slice-observation.mjs';
export function expectedFallibleRecordCallbacksValues(id,n,seed,input){
 if(id!=='fallible_record_callbacks'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='fallible')throw Error('Fallible record callbacks case');
 return Array.from({length:n},(_,i)=>{
  const value=i%5+seed%5+1;
  return [value,value,value,value+1000,value>2?3:1];
 });
}
export function expectedFallibleRecordCallbacksObservation(id,n,seed,input){return observeSliceTree(expectedFallibleRecordCallbacksValues(id,n,seed,input));}
