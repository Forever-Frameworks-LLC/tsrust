// Untimed acceptance through the same ABI adapters used by the benchmark runner.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {adaptRustDriver,adaptTSDriver,assertABI} from './adapters.mjs';
import {assertNativeFilesSource} from './native-files-source-contract.mjs';
import {createNativeFileFixture} from './native-files-input.mjs';
import {expectedNativeFilesObservation} from './native-files-oracle.mjs';
const here=path.dirname(fileURLToPath(import.meta.url));
const manifest=JSON.parse(fs.readFileSync(path.join(here,'manifest.json'),'utf8'));
const compiler=process.env.TSRUST;
if(!compiler)throw Error('Set TSRUST to the current verified compiler');
function run(command,args){const p=spawnSync(command,args,{encoding:'utf8',timeout:60000,maxBuffer:16*1024*1024});assert.equal(p.status,0,p.stderr||String(p.error));return p.stdout;}
const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-write-acceptance-'));
try {
 for(const id of ['native_file_write','native_file_write_all','native_file_full_write_read','native_file_buffered_write_read']) {
  const row=manifest.workloads.find(row=>row.id===id);
  const source=fs.readFileSync(path.join(here,row.source),'utf8'),ts=fs.readFileSync(path.join(here,row.counterpart),'utf8');
  assertNativeFilesSource(source,row,'native');assertNativeFilesSource(ts,row,'typescript');
  const emitted=run(compiler,['emit',path.join(here,row.source)]);assertABI(emitted,row,{source});
  const rust=path.join(directory,id+'.rs'),typed=path.join(directory,id+'.ts'),config=path.join(directory,id+'.json');
  fs.writeFileSync(rust,emitted.replace('fn main()','fn source_main()')+'\n'+adaptRustDriver('',row,{emitted,source}));
  fs.writeFileSync(typed,ts+'\n'+adaptTSDriver('',row));
  fs.writeFileSync(config,JSON.stringify({extends:path.join(here,'tsconfig.json'),files:[typed],compilerOptions:{outDir:directory}}));
  run('rustup',['run','1.96.1','rustc','--edition=2024','-Copt-level=3','-Coverflow-checks=on','-Awarnings',rust,'-o',path.join(directory,id)]);
  run(process.execPath,[path.join(here,'node_modules/typescript/bin/tsc'),'--project',config]);
  const fixture=createNativeFileFixture(row);
  try {
   for(const [n,seed] of [[0,0],[1,0],[65,7],[137,1000000]]) {
    const args=[String(n),String(seed),'1','0',fixture.path];
    const expected=expectedNativeFilesObservation(id,n,seed,row.inputText);
    for(const [engine,command,prefix] of [['native',path.join(directory,id),[]],['node-tsc',process.execPath,[path.join(directory,id+'.js')]]]) {
      fs.writeFileSync(fixture.path+'.out',Buffer.alloc(4096,255));
      const result=JSON.parse(run(command,[...prefix,...args]));
      assert.equal(result.checksum,expected.checksum);assert.equal(result.exactBits,expected.exactBits);assert.deepEqual(result.durations,[]);
      if(n>0){const count=(n-1+seed)%64+1;assert.deepEqual(fs.readFileSync(fixture.path+'.out'),Buffer.from('0123456789abcdef'.repeat(4)).subarray(0,count));}
      console.log(JSON.stringify({id,engine,n,seed,status:'pass',timed:false}));
    }
   }
  } finally {fixture.cleanup();}
 }
} finally {fs.rmSync(directory,{recursive:true,force:true});}
