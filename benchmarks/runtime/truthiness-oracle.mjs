import {observeSliceTree} from './slice-observation.mjs';
export function expectedTruthinessValues(id,n,seed,input){
 if(id!=='truthiness'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='truthiness')throw Error('Owned assignment values case');
 return Array.from({length:n},(_,i)=>{
  const value=i%5+seed%5+1;
  return [value+1,6,7,value,0,1];
 });
}
export function expectedTruthinessObservation(id,n,seed,input){return observeSliceTree(expectedTruthinessValues(id,n,seed,input));}
