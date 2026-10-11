import {observeSliceTree} from './slice-observation.mjs';
export const exceptionCompletionIds=['exception_completion'];
// Closed-form arithmetic is independent of exception execution.
export function expectedExceptionCompletionValues(id,n,seed,input){
 if(!exceptionCompletionIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Exception completion case');
 const output=[];
 for(let i=0;i<n;i++){
  const value=(i*13+seed)%97;
  output.push([String(value+(value%3===0?2:1)),String(value+4),String(value)+",7,11,"+(value%2===0?"4":"32,8")+(value%4<2?",20":",20,21")+(value%3===0?",31":value%3===1?",30,31":",29,31"),value%2===0&&value%3!==0?input+String(value):"alternate"]);
 }
 return output;
}
export function expectedExceptionCompletionObservation(id,n,seed,input){return observeSliceTree(expectedExceptionCompletionValues(id,n,seed,input));}
