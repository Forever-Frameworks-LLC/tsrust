import {observeSliceTree} from './slice-observation.mjs';
export function expectedRecursiveResultInferenceValues(id,n,seed,input){
 if(id!=='recursive_result_inference'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='recursive')throw Error('Recursive result inference case');
 return Array.from({length:n},(_,i)=>{
  const value=i%5+seed%5+1;
  let factorial=1;for(let k=2;k<=value;k++)factorial*=k;
  const fib=k=>k<2?k:fib(k-1)+fib(k-2);
  return [factorial,value%2===0?1:0,value,value,value+1,fib(value)];
 });
}
export function expectedRecursiveResultInferenceObservation(id,n,seed,input){return observeSliceTree(expectedRecursiveResultInferenceValues(id,n,seed,input));}
