import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import {createRequire} from 'node:module';
import {createHash} from 'node:crypto';
import ts from 'typescript';
import {createNativeFileFixture} from './native-files-input.mjs';
import {expectedNativeFilesValues,expectedNativeFilesObservation} from './native-files-oracle.mjs';
import {assertNativeFilesSource} from './native-files-source-contract.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {measurementPlan} from './measurement-plan.mjs';
import {reportCategory} from './report.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
const read=name=>fs.readFileSync(new URL(name,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
for(const id of ['native_directory_metadata','native_file_copy_rename_remove','native_file_options_text','native_file_seek_read','native_file_sync_storage'])test(`${id} retains independent fixture observations and matched Node control`,()=>{
 const row=manifest.workloads.find(row=>row.id===id);assert.ok(row);assert.equal(reportCategory(id),'resources');
 assert.throws(()=>createNativeFileFixture({...row,fixtureSha256:'changed'}));
 for(const [kind,file] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+file);assertNativeFilesSource(source,row,kind);assert.throws(()=>assertNativeFilesSource(source,{...row,iterations:1},kind));
  const needle=id==='native_file_sync_storage'?(kind==='native'?'syncAll(file)':'fileSystem.fsyncSync(file)'):id==='native_file_seek_read'?(kind==='native'?'seek(file, offset, "start")':'fileSystem.readSync(file,bytes,0,64,position)'):id==='native_directory_metadata'?(kind==='native'?'stat(entry.path)':'fileSystem.statSync(entry.path, { bigint: true })'):id==='native_file_copy_rename_remove'?(kind==='native'?'removeFile(moved)':'fileSystem.unlinkSync(moved)'):(kind==='native'?'truncate(file, 0)':'fileSystem.ftruncateSync(file, 0)');
  const changed=replaceCodeSnippet(source,needle,'skipped()');
  const fingerprint=kind==='native'?'sourceSha256':'counterpartSha256';
  assert.throws(()=>assertNativeFilesSource(changed,{...row,[fingerprint]:createHash('sha256').update(changed).digest('hex')},kind));
 }
 if(id==='native_file_seek_read'){
  const rust=read('./parity/'+id+'.rs');assert.ok(rust.includes('Vec::with_capacity(usize::try_from(n).unwrap())'));
  const buffer=rust.indexOf('let mut bytes=[0u8;64]'),loop=rust.indexOf('for i in 0..n');assert.ok(buffer>=0&&buffer<loop);assert.ok(rust.includes('bytes[..count]'));assert.ok(!rust.includes('vec![0;64]'));
 }
 const fixture=createNativeFileFixture(row);
 try{
  if(id==='native_directory_metadata'){
   const names=fs.readdirSync(fixture.path);assert.equal(names.length,67);assert.ok(names.includes('.hidden'));assert.ok(names.includes('é-🦀.txt'));
   const files=names.map(name=>fs.statSync(path.join(fixture.path,name))).filter(value=>value.isFile());assert.equal(files.length,66);assert.equal(files.reduce((sum,value)=>sum+value.size,0),998);
  }else if(id==='native_file_copy_rename_remove'){
   const bytes=fs.readFileSync(path.join(fixture.path,'source.bin'));assert.equal(bytes.length,1048576);for(let i=0;i<bytes.length;i++)assert.equal(bytes[i],(i*17+3)%256);
  }
  const source=read('./'+row.counterpart);const js=ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
  const context=vm.createContext({require:createRequire(import.meta.url),TextEncoder,TextDecoder});vm.runInContext(js+'\nglobalThis.run=workload;',context);
  for(const [n,seed] of measurementPlan(['--verify'],manifest).correctnessCases(row.iterations)){
   const expected=expectedNativeFilesObservation(id,n,seed,row.inputText);assert.deepEqual(expectedExactResult(id,n,seed,row.inputText),expected);assert.equal(expectedChecksum(id,n,seed,row.inputText),expected.checksum);
   const actual=JSON.parse(JSON.stringify(context.run(n,seed,fixture.path)));assert.deepEqual(actual,expectedNativeFilesValues(id,n,seed,row.inputText));
   if(id==='native_file_sync_storage')assert.deepEqual(fs.readFileSync(fixture.path+'.out'),Buffer.from('ab'.repeat(n)));
  }
 }finally{fixture.cleanup();}
 assert.equal(fs.existsSync(fixture.path),false);
});
