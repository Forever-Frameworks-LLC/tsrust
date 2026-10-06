import {observeSliceTree} from './slice-observation.mjs';
// Independent of type declarations: every cell is a direct expression.
export function expectedTypeAliasesValues(id,n,seed,input){
 if(id!=='type_aliases'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('Type alias case');
 return Array.from({length:n},(_,i)=>{const k=i+seed;return ['number','text',String(k*2),String(k),'2'];});
}
export function expectedTypeAliasesObservation(id,n,seed,input){return observeSliceTree(expectedTypeAliasesValues(id,n,seed,input));}
