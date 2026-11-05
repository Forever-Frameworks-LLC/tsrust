import {observeSliceTree} from './slice-observation.mjs';
export const nativeFileIDs = ['native_file_read_close','native_file_scope_drop','native_file_write','native_file_write_all','native_file_full_write_read','native_file_buffered_write_read','native_file_unbounded_full_write_read','native_directory_metadata','native_file_copy_rename_remove','native_file_options_text','native_file_seek_read','native_file_sync_storage'];
export function expectedNativeFilesValues(id,n,seed,input) {
  if (!nativeFileIDs.includes(id) || !Number.isSafeInteger(n) || n<0 || n>2048 || !Number.isSafeInteger(seed) || seed<0 || seed>1000000 || input!==({native_directory_metadata:'native-directory-metadata-v1',native_file_copy_rename_remove:'native-copy-file-v1',native_file_options_text:'native-options-text-v1'}[id]??'native-file-fixture-v1')) throw Error('Native file case');
  if(id==='native_file_sync_storage')return Array.from({length:n},(_,i)=>[2*(i+1),195+(i+seed)%17]);
  if(id==='native_file_seek_read')return Array.from({length:n},(_,i)=>{const offset=i%8===0?4096:(i*97+seed)%4096;const count=Math.min(64,4096-offset);let sum=0;for(let j=0;j<count;j++){const value=(offset+j)%16;sum+=value<10?48+value:87+value;}return [offset+count,sum];});
  if(id==='native_directory_metadata')return Array.from({length:n},(_,i)=>[67,66,998,9+(i+seed)%17]);
  if(id==='native_file_copy_rename_remove')return Array.from({length:n},(_,i)=>[1048576,1048576+(i+seed)%17]);
  if(id==='native_file_options_text')return Array.from({length:n},(_,i)=>[8,7,(i+seed)%17]);
  if(id==='native_file_unbounded_full_write_read') {
    return Array.from({length:n},(_,i)=>{const count=[65536,1048576,67108865][i%3];return [count,count*((i+seed)%256)];});
  }
  return Array.from({length:n},(_,i)=>{
    const count=(i+seed)%64+1;
    const tail=count%16, digits=Math.min(tail,10), letters=Math.max(tail-10,0);
    return [count,Math.floor(count/16)*1122+48*digits+digits*(digits-1)/2+97*letters+letters*(letters-1)/2];
  });
}
export function expectedNativeFilesObservation(id,n,seed,input) { return observeSliceTree(expectedNativeFilesValues(id,n,seed,input)); }
