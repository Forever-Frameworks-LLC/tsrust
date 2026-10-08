import {observeSliceTree} from './slice-observation.mjs';
export function expectedGenericReplacementValues(id,n,seed,input){
 if(id!=='generic_callable_replacement'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='replacement')throw Error('Generic replacement case');
 return Array.from({length:n},(_,i)=>{
  const base=i%5+seed%5+1;
  return (i+seed)%2===1?[base,base+7,11]:[base,base,base];
 });
}
export function expectedGenericReplacementObservation(id,n,seed,input){return observeSliceTree(expectedGenericReplacementValues(id,n,seed,input));}
