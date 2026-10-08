import {observeSliceTree} from './slice-observation.mjs';
export function expectedScopedRecordCompositionValues(id,n,seed,input) {
 if(id!=='scoped_record_composition'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='composition') throw Error('Scoped record composition case');
 return Array.from({length:n},(_,i)=>{const value=i%5+seed%5+1;return [value,value+1,value,value+1,value+2,2*value+1,2,0,value+1];});
}
export function expectedScopedRecordCompositionObservation(id,n,seed,input) {return observeSliceTree(expectedScopedRecordCompositionValues(id,n,seed,input));}
