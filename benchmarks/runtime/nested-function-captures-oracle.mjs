import {observeSliceTree} from './slice-observation.mjs';
export function expectedNestedFunctionCaptureValues(id,n,seed,input){
 if(id!=='nested_function_captures'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='captures')throw Error('Nested capture case');
 let total=0;
 return Array.from({length:n},(_,i)=>{
  const value=i%5+seed%5+1;total+=2*value;
  return [total,2*(i+1),value+1,value+2,value+4,2*(value+3),2*value];
 });
}
export function expectedNestedFunctionCaptureObservation(id,n,seed,input){return observeSliceTree(expectedNestedFunctionCaptureValues(id,n,seed,input));}
