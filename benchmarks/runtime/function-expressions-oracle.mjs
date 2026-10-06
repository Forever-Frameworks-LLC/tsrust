import {observeSliceTree} from './slice-observation.mjs';
export const functionExpressionsIds=['function_expressions'];
// Independent of function values and array callbacks. The smallest of three
// texts is found by direct code-unit comparison, not by sorting.
function before(left,right){
 const shared=Math.min(left.length,right.length);
 for(let i=0;i<shared;i++){const a=left.charCodeAt(i),b=right.charCodeAt(i);if(a!==b)return a<b;}
 return left.length<right.length;
}
export function expectedFunctionExpressionsValues(id,n,seed,input){
 if(!functionExpressionsIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Function expression case');
 const result=[];for(let i=0;i<n;i++){const base=(i*13+seed)%97,digits=''+base;
  let least=input;if(before(digits,least))least=digits;if(before('z',least))least='z';
  result.push([''+((i%5)*3+base),''+(digits.length+4),least,'a0','b2']);}
 return result;
}
export function expectedFunctionExpressionsObservation(id,n,seed,input){return observeSliceTree(expectedFunctionExpressionsValues(id,n,seed,input));}
