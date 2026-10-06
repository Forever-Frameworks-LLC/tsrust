import {observeSliceTree} from './slice-observation.mjs';
export function expectedStringConcatenationValues(id,n,seed,input){
 if(id!=='string_concatenation'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Concatenation case');
 return Array.from({length:n},(_,i)=>{const k=i+seed;const digits=String(k);const flag=k%2===0?'true':'false';return [input.concat(digits),digits.concat(input),'fraction:'.concat(String(k%4/2)),'flag:'.concat(flag),'absent:nullundefined','union:'.concat(k%2===0?digits:'undefined'),input.concat(digits,flag)];});
}
export function expectedStringConcatenationObservation(id,n,seed,input){return observeSliceTree(expectedStringConcatenationValues(id,n,seed,input));}
