import {observeSliceTree} from './slice-observation.mjs';
export function expectedNumericLiteralsValues(id,n,seed,input){
 if(id!=='numeric_literals'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Numeric literal case');
 return Array.from({length:n},(_,i)=>{const k=i+seed;let masked=0;for(const bit of [1,2,32,128])if(Math.floor(k/bit)%2===1)masked+=bit;return [String(masked),String(57068+k),String(1000*k),String(k+1.2),'18446744073709552000',input];});
}
export function expectedNumericLiteralsObservation(id,n,seed,input){return observeSliceTree(expectedNumericLiteralsValues(id,n,seed,input));}
