import {observeSliceTree} from './slice-observation.mjs';
export function expectedCallerBorrowedRecordStoreValues(id,n,seed,input) {
 if(id!=='caller_borrowed_record_stores'||!Number.isSafeInteger(n)||n<0||n>2048||!Number.isSafeInteger(seed)||seed<0||seed>1000000||input!=='borrowed-inputs')throw Error('Caller-borrowed record store case');
 return Array.from({length:n},(_,i)=>{
  const value=i%5+seed%5+1;
  return [value,value+2,2*value+2,value,value+9,value+2,value+3,value+4,value+5,value+6,value+6,9,value+9,value+12];
 });
}
export function expectedCallerBorrowedRecordStoreObservation(id,n,seed,input) {
 return observeSliceTree(expectedCallerBorrowedRecordStoreValues(id,n,seed,input));
}
