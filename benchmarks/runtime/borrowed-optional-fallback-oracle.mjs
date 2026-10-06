import {observeSliceTree} from './slice-observation.mjs';
export const borrowedOptionalFallbackIds=['borrowed_optional_fallback'];
export function expectedBorrowedOptionalFallbackValues(id,n,seed,input){
 if(!borrowedOptionalFallbackIds.includes(id)||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Borrowed optional fallback case');
 const result=[];
 for(let i=0;i<n;i++){const text=input.repeat((i+seed)%3+1);for(const value of [text,'none',''])result.push([value.toUpperCase(),value,''+i]);}
 return result;
}
export function expectedBorrowedOptionalFallbackObservation(id,n,seed,input){return observeSliceTree(expectedBorrowedOptionalFallbackValues(id,n,seed,input));}
