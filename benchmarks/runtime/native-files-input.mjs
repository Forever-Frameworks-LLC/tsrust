import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {createHash} from 'node:crypto';
export function createNativeFileFixture(row) {
  if(row.family!=='native-files'||row.inputText!=='native-file-fixture-v1') throw Error('Native file fixture contract');
  const bytes=Buffer.from('0123456789abcdef'.repeat(256),'ascii');
  if(createHash('sha256').update(bytes).digest('hex')!==row.fixtureSha256) throw Error('Native file fixture fingerprint');
  const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-file-workload-'));
  const file=path.join(directory,'input.bin');
  try { fs.writeFileSync(file,bytes,{flag:'wx',mode:0o600}); if(['native_file_write','native_file_write_all'].includes(row.id))fs.writeFileSync(file+'.out',bytes,{flag:'wx',mode:0o600}); }
  catch(error) { fs.rmSync(directory,{recursive:true,force:true});throw error; }
  return {path:file,cleanup:()=>fs.rmSync(directory,{recursive:true,force:true})};
}
const fixtures=new Map();
function cleanup(){for(const fixture of fixtures.values())fixture.cleanup();fixtures.clear();}
export function nativeFileInput(row) {
  if(row?.family!=='native-files')return row?.inputText;
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
