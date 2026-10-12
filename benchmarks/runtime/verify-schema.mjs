// Untimed execution through the same native and Node-after-tsc benchmark adapters.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {adaptRustDriver,adaptTSDriver,assertABI} from './adapters.mjs';
import {prepareNativeDependencies} from './native-dependencies.mjs';
import {assertSchemaSource,schemaChecksum} from './schema-contract.mjs';
const here=path.dirname(fileURLToPath(import.meta.url));
const row=JSON.parse(fs.readFileSync(path.join(here,'manifest.json'),'utf8')).workloads.find(row=>row.id===(process.argv[2]??'native_schema_string'));
const pins=JSON.parse(fs.readFileSync(path.join(here,'versions.json'),'utf8'));
assert(process.env.TSRUST,'Set TSRUST to the verified compiler');
assert(process.env.CARGO_TARGET_DIR,'Reuse the shared Cargo target');
function run(command,args){const child=spawnSync(command,args,{encoding:'utf8',timeout:60000,maxBuffer:16*1024*1024});assert.equal(child.status,0,child.stderr||child.stdout||String(child.error));return child.stdout;}
const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-schema-verify-'));
try {
 const sourcePath=path.join(here,row.source),source=fs.readFileSync(sourcePath,'utf8'),ts=fs.readFileSync(path.join(here,row.counterpart),'utf8');
 assertSchemaSource(source,row,'native');assertSchemaSource(ts,row,'typescript');
 const emitted=run(process.env.TSRUST,['emit',sourcePath]);assertABI(emitted,row,{source});
 const dependencies=prepareNativeDependencies({emitted,compiler:process.env.TSRUST,source:sourcePath,build:directory,rustVersion:pins.rust,targetDirectory:process.env.CARGO_TARGET_DIR});
 fs.writeFileSync(path.join(directory,'workload.rs'),emitted.replace('fn main()','fn source_main()')+'\n'+adaptRustDriver(fs.readFileSync(path.join(here,'driver.rs'),'utf8'),row,{emitted,source}));
 fs.writeFileSync(path.join(directory,'workload.ts'),ts+'\n'+adaptTSDriver(fs.readFileSync(path.join(here,'driver.ts'),'utf8'),row));
 fs.writeFileSync(path.join(directory,'tsconfig.json'),JSON.stringify({extends:path.join(here,'tsconfig.json'),files:[path.join(directory,'workload.ts')],compilerOptions:{outDir:directory}}));
 run('rustup',['run',pins.rust,'rustc',...dependencies.args,'--edition=2024','-Copt-level=3','-Coverflow-checks=on','-Awarnings',path.join(directory,'workload.rs'),'-o',path.join(directory,'workload')]);
 run(process.execPath,[path.join(here,'node_modules/typescript/bin/tsc'),'--project',path.join(directory,'tsconfig.json')]);
 for(const [n,seed] of [[0,0],[1,0],[17,7],[2000,1000000]])for(const [engine,command,prefix] of [['native',path.join(directory,'workload'),[]],['node-tsc',process.execPath,[path.join(directory,'workload.js')]]]){
  const result=JSON.parse(run(command,[...prefix,String(n),String(seed),'1','0',row.inputText]));
  assert.equal(result.checksum,schemaChecksum(n,seed,row.inputText));assert.deepEqual(result.durations,[]);
  console.log(JSON.stringify({id:row.id,engine,n,seed,status:'pass',timed:false}));
 }
}finally{fs.rmSync(directory,{recursive:true,force:true});}
