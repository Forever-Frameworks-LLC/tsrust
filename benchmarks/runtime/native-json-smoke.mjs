// Untimed acceptance of both JSON API surfaces using the measurement drivers.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {assertABI,adaptRustDriver,adaptTSDriver} from './adapters.mjs';
import {assertNativeJsonSource} from './native-json-contract.mjs';
import {prepareNativeDependencies} from './native-dependencies.mjs';
import {expectedChecksum} from './oracle.mjs';
const root=path.dirname(fileURLToPath(import.meta.url));
const compiler=process.env.TSRUST;
if(!compiler||!process.env.CARGO_TARGET_DIR)throw Error('Set TSRUST and the shared CARGO_TARGET_DIR');
const pins=JSON.parse(fs.readFileSync(root+'/versions.json'));
const rows=JSON.parse(fs.readFileSync(root+'/manifest.json')).workloads.filter(row=>row.family==='native-json');
const build=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-json-benchmark-'));
const run=(command,args)=>execFileSync(command,args,{encoding:'utf8',timeout:60000,maxBuffer:16*1024*1024});
try {
 for(const row of rows){
  const source=fs.readFileSync(root+'/'+row.source,'utf8');
  const counterpart=fs.readFileSync(root+'/'+row.counterpart,'utf8');
  assertNativeJsonSource(source,row,'native');assertNativeJsonSource(counterpart,row,'typescript');
  const emitted=run(compiler,['emit',root+'/'+row.source]);
  assertABI(emitted,row,{source});
  const dependencies=prepareNativeDependencies({emitted,compiler,source:root+'/'+row.source,build,rustVersion:pins.rust,targetDirectory:process.env.CARGO_TARGET_DIR});
  const base=build+'/'+row.id;
  fs.writeFileSync(base+'.rs',emitted.replace('fn main()','fn source_main()')+'\n'+adaptRustDriver(fs.readFileSync(root+'/driver.rs','utf8'),row,{emitted,source}));
  run('rustup',['run',pins.rust,'rustc',...dependencies.args,'--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',base+'.rs','-o',base]);
  fs.writeFileSync(base+'.ts',counterpart+'\n'+adaptTSDriver(fs.readFileSync(root+'/driver.ts','utf8'),row));
  fs.writeFileSync(base+'.json',JSON.stringify({extends:root+'/tsconfig.json',files:[base+'.ts'],compilerOptions:{outDir:build}}));
  run(process.execPath,[root+'/node_modules/typescript/bin/tsc','--project',base+'.json']);
  for(const [n,seed] of [[0,0],[1,7],[2,19],[17,7],[2000,19]]){
   for(const [exe,args] of [[base,[]],[process.execPath,[base+'.js']]]){
    const result=JSON.parse(run(exe,[...args,String(n),String(seed),'1','0',row.inputText]));
    assert.equal(result.checksum,expectedChecksum(row.id,n,seed,row.inputText));
    assert.deepEqual(result.durations,[]);
   }
  }
  console.log(`PASS ${row.id}: native and Node after tsc; five cases; zero timing samples`);
 }
} finally {fs.rmSync(build,{recursive:true,force:true});}
