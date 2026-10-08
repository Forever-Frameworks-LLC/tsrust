import {observeSliceTree} from './slice-observation.mjs';
export function expectedScopedRecordCompositionValues(id,n,seed,input) {
 if(id!=='scoped_record_composition'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='composition') throw Error('Scoped record composition case');
 return Array.from({length:n},(_,i)=>{const value=i%5+seed%5+1;return [value,value+1,value,value+1,value+2,2*value+1,value+3,2,0,value+1,3*value+16,0,3*value+28,3*value+40,3*value+42,1,1,value%2===0?2:4,value+21,value+22,value+29,value+30,value+39,value+40,value+50+(value%2===0?0:1),2*value+51+(value%2===0?0:1),value+60+(value%2===0?0:1),value%2===0?107:2*value+242,value%2===0?value+180:2*value+265,2*value+484,2*value+484,3*value+305,3*value+641,value%2===0?107:2*value+342,value%2===0?2*value+348:2*value+1242];});
}
export function expectedScopedRecordCompositionObservation(id,n,seed,input) {return observeSliceTree(expectedScopedRecordCompositionValues(id,n,seed,input));}
