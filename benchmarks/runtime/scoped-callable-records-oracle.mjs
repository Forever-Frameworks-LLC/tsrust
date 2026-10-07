import {observeSliceTree} from './slice-observation.mjs';
export function expectedScopedCallableRecordValues(id,n,seed,input){
 if(id!=='scoped_callable_records'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='records')throw Error('Scoped callable record case');
 return Array.from({length:n},(_,i)=>{
  const value=i%5+seed%5+1;
  return [value,2*value,2*value+1,value+7,2*(value+1),2*value,value+1];
 });
}
export function expectedScopedCallableRecordObservation(id,n,seed,input){return observeSliceTree(expectedScopedCallableRecordValues(id,n,seed,input));}
