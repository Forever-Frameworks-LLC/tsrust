import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
export function createNativeFileFixture(row) {
  if(['native_directory_metadata','native_file_copy_rename_remove','native_file_options_text'].includes(row.id))return createUtilityFixture(row);
  const writer=['native_jsonl_writer','native_jsonl_borrowed_writer'].includes(row.id)&&row.family==='native-json'&&row.inputText==='native-jsonl-writer-v1';
  const jsonl=['native_jsonl_typed','native_jsonl_borrowed'].includes(row.id)&&row.family==='native-json'&&row.inputText==='native-jsonl-typed-v1';
  if(!writer&&!jsonl&&(row.family!=='native-files'||row.inputText!=='native-file-fixture-v1')) throw Error('Native file fixture contract');
  const bytes=Buffer.from(writer?'':jsonl?'{"name":"hello","values":[20,17],"active":true}\n{"name":"hello","values":[20,17],"active":true}\n':'0123456789abcdef'.repeat(256),'ascii');
  if(createHash('sha256').update(bytes).digest('hex')!==row.fixtureSha256) throw Error('Native file fixture fingerprint');
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-file-workload-'));
  const file=path.join(directory,'input.bin');
  try { fs.writeFileSync(file,bytes,{flag:'wx',mode:0o600}); if(['native_file_write','native_file_write_all','native_file_full_write_read','native_file_buffered_write_read','native_file_unbounded_full_write_read','native_file_sync_storage'].includes(row.id))fs.writeFileSync(file+'.out',bytes,{flag:'wx',mode:0o600}); }
  catch(error) { fs.rmSync(directory,{recursive:true,force:true});throw error; }
  return {path:file,cleanup:()=>fs.rmSync(directory,{recursive:true,force:true})};
}
const fixtures=new Map();
function cleanup(){for(const fixture of fixtures.values())fixture.cleanup();fixtures.clear();}
export function nativeFileInput(row) {
  if(row?.family!=='native-files'&&!['native_jsonl_typed','native_jsonl_borrowed'].includes(row?.id)&&!['native_jsonl_writer','native_jsonl_borrowed_writer'].includes(row?.id))return row?.inputText;
  if(!fixtures.has(row.id)) {
    if(fixtures.size===0) {
      process.once('exit',cleanup);
      process.once('SIGINT',()=>{cleanup();process.exit(130);});
      process.once('SIGTERM',()=>{cleanup();process.exit(143);});
    }
    fixtures.set(row.id,createNativeFileFixture(row));
  }
  return fixtures.get(row.id).path;
}

function utilityContents(id) {
  if(id==='native_directory_metadata')return [...Array.from({length:64},(_,i)=>[`entry-${String(i).padStart(2,'0')}.bin`,Buffer.alloc(i%31+1,i)]),['.hidden',Buffer.alloc(0)],['é-🦀.txt',Buffer.from([0,255,7])],['nested',null]];
  if(id==='native_file_copy_rename_remove')return [['source.bin',Buffer.from(Array.from({length:1048576},(_,i)=>(i*17+3)%256))]];
  if(id==='native_file_options_text')return [['input.bin',Buffer.alloc(0)]];
  throw Error('Native utility fixture');
}
function createUtilityFixture(row) {
  const markers={native_directory_metadata:'native-directory-metadata-v1',native_file_copy_rename_remove:'native-copy-file-v1',native_file_options_text:'native-options-text-v1'};
  if(row.family!=='native-files'||row.inputText!==markers[row.id])throw Error('Native utility fixture contract');
  const contents=utilityContents(row.id);
  const bytes=JSON.stringify(contents.map(([name,bytes])=>[name,bytes===null?null:bytes.toString('hex')]));
  if(createHash('sha256').update(bytes).digest('hex')!==row.fixtureSha256)throw Error('Native utility fixture fingerprint');
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-fs-utility-'));
  try {for(const [name,bytes] of contents) {const file=path.join(directory,name);if(bytes===null)fs.mkdirSync(file);else fs.writeFileSync(file,bytes,{flag:'wx'});}}
  catch(error) {fs.rmSync(directory,{recursive:true,force:true});throw error;}
  return {path:row.id==='native_file_options_text'?path.join(directory,'input.bin'):directory,cleanup:()=>fs.rmSync(directory,{recursive:true,force:true})};
}
