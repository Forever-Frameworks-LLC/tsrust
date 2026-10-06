import {observeSliceTree} from './slice-observation.mjs';
// Independent of bracket access: characters come from charAt and a length test.
export function expectedStringIndexingValues(id,n,seed,input){
 if(id!=='string_indexing'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||typeof input!=='string')throw Error('String indexing case');
 const letters='abcdefghij';
 return Array.from({length:n},(_,i)=>{const position=(i+seed)%10;return [letters.charAt(position),letters.charAt(position)+letters.charAt(9-position),position+5<letters.length?letters.charAt(position+5):'out',String(position),input];});
}
export function expectedStringIndexingObservation(id,n,seed,input){return observeSliceTree(expectedStringIndexingValues(id,n,seed,input));}
