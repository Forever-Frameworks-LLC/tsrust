import {observeSliceTree} from './slice-observation.mjs';
export const nativeFileIDs = ['native_file_read_close','native_file_scope_drop','native_file_write','native_file_write_all','native_file_full_write_read','native_file_buffered_write_read'];
export function expectedNativeFilesValues(id,n,seed,input) {
  if (!nativeFileIDs.includes(id) || !Number.isSafeInteger(n) || n<0 || n>2048 || !Number.isSafeInteger(seed) || seed<0 || seed>1000000 || input!=='native-file-fixture-v1') throw Error('Native file case');
  return Array.from({length:n},(_,i)=>{
    const count=(i+seed)%64+1;
    const tail=count%16, digits=Math.min(tail,10), letters=Math.max(tail-10,0);
    return [count,Math.floor(count/16)*1122+48*digits+digits*(digits-1)/2+97*letters+letters*(letters-1)/2];
  });
}
export function expectedNativeFilesObservation(id,n,seed,input) { return observeSliceTree(expectedNativeFilesValues(id,n,seed,input)); }
