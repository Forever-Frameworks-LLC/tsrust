import {observeSliceTree} from './slice-observation.mjs';
export function expectedGenericReplacementValues(id,n,seed,input){
 if(id!=='generic_callable_replacement'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='replacement')throw Error('Generic replacement case');
 return Array.from({length:n},(_,i)=>{
  const base=i%5+seed%5+1;
  const flag=(i+seed)%2===1;
  const callback=i%3===0?11:base+(flag?7:0);
  const captured=flag&&i%3===0?11:2*base+(flag?7:0);
  return flag?[base,base+7,11,7,10,8,0,callback,captured,base,base+7,base,base+9]:[base,base,base,3,4,4,4,callback,captured,base,base+7,base,base+9];
 });
}
export function expectedGenericReplacementObservation(id,n,seed,input){return observeSliceTree(expectedGenericReplacementValues(id,n,seed,input));}
