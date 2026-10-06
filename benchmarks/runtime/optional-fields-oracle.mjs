import {observeSliceTree} from './slice-observation.mjs';
// Independent of records: every cell is direct arithmetic.
export function expectedOptionalFieldsValues(id,n,seed,input){
 if(id!=='optional_fields'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Optional field case');
 return Array.from({length:n},(_,i)=>{const k=i+seed;return [String(k),String(k+k%7),String(1+k%3),'9',input];});
}
export function expectedOptionalFieldsObservation(id,n,seed,input){return observeSliceTree(expectedOptionalFieldsValues(id,n,seed,input));}
