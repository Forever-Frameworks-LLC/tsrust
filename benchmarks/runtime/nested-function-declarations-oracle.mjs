import {observeSliceTree} from './slice-observation.mjs';
export function expectedNestedFunctionDeclarationValues(id,n,seed,input){
 if(id!=='nested_function_declarations'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='nested')throw Error('Nested declaration case');
 return Array.from({length:n},(_,i)=>{
  const value=i%5+seed%5+1;
  return [2*value,120,value%2===0?1:0,3*value,2*value,2*(value+1),3*value+4];
 });
}
export function expectedNestedFunctionDeclarationObservation(id,n,seed,input){return observeSliceTree(expectedNestedFunctionDeclarationValues(id,n,seed,input));}
