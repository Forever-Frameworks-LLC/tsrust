import {observeSliceTree} from './slice-observation.mjs';
export function expectedNamedFunctionExpressionValues(id,n,seed,input){
 if(id!=='named_function_expressions'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='named')throw Error('Named function expression case');
 return Array.from({length:n},(_,i)=>{
  const value=i%5+seed%5+1;let factorial=1;for(let k=2;k<=value;k++)factorial*=k;
  return [factorial,6+3*value,value+2,value+3,value+2,value,1];
 });
}
export function expectedNamedFunctionExpressionObservation(id,n,seed,input){return observeSliceTree(expectedNamedFunctionExpressionValues(id,n,seed,input));}
