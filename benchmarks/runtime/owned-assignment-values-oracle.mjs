import {observeSliceTree} from './slice-observation.mjs';
export function expectedOwnedAssignmentValuesValues(id,n,seed,input){
 if(id!=='owned_assignment_values'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='assignment')throw Error('Owned assignment values case');
 return Array.from({length:n},(_,i)=>{
  const value=i%5+seed%5+1;
  return [value,5,3,value+2,value*2,value+1,value+1,value,5];
 });
}
export function expectedOwnedAssignmentValuesObservation(id,n,seed,input){return observeSliceTree(expectedOwnedAssignmentValuesValues(id,n,seed,input));}
