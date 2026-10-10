import {observeSliceTree} from './slice-observation.mjs';
export function expectedCallerBorrowedRecordStoreValues(id,n,seed,input) {
 if(id!=='caller_borrowed_record_stores'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='borrowed-inputs')throw Error('Caller-borrowed record store case');
 return Array.from({length:n},(_,i)=>{
  const value=i%5+seed%5+1;
  return [value,value+2,2*value+2,value,value+9,value+2,value+3,value+4,value+5,value+6,value+6,9,value+9,value+12,value,value+6,value+9,value+17,value+1,value+2,1,value+1,value+7,value+10,value+1,value+2,value+5,value+6,value+2,value+5,value+6,2,2,value+(i%2===0?1:2),value+1,value+(i%2===0?6:5),329+2*value,value+1,value+1,value+6,329+2*value,value+3,value+3,value+8,333+2*value,value+4,value+4,value+8,238+2*value,value+8,value+9,value+9,value+10,value+13,value+15,value+15,value+19,256+2*value,value+21,value+22,value+23,2,value+31,value+32,value+40,value+41,value+45,2,value+50,value+51,value+60,2,value+80,value%2===0?value+80:value+70];
 });
}
export function expectedCallerBorrowedRecordStoreObservation(id,n,seed,input) {
 return observeSliceTree(expectedCallerBorrowedRecordStoreValues(id,n,seed,input));
}
