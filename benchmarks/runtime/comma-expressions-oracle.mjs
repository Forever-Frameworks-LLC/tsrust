import {observeSliceTree} from './slice-observation.mjs';
export function expectedCommaExpressionsValues(id,n,seed,input){
 if(id!=='comma_expressions'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Comma expression case');
 return Array.from({length:n},(_,i)=>{const j=i+seed;const value=3*j;return [String(value),String(value+1),[j,j+2,value,j+4].join(','),input,String(i),String(j)];});
}
export function expectedCommaExpressionsObservation(id,n,seed,input){return observeSliceTree(expectedCommaExpressionsValues(id,n,seed,input));}
