// Untimed clock acceptance through the registered native and Node-after-tsc ABI.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {adaptRustDriver,adaptTSDriver,assertABI} from './adapters.mjs';
import {monotonicTimeID,expectedMonotonicTimeObservation,assertMonotonicTimeSource} from './monotonic-time-contract.mjs';
const here=path.dirname(fileURLToPath(import.meta.url));
const row=JSON.parse(fs.readFileSync(path.join(here,'manifest.json'),'utf8')).workloads.find(w=>w.id===monotonicTimeID);
assert(process.env.TSRUST,'Set TSRUST to the current verified compiler');
function run(command,args,env=process.env){const p=spawnSync(command,args,{env,encoding:'utf8',timeout:60000,maxBuffer:16*1024*1024});assert.equal(p.status,0,p.stderr||String(p.error));return p.stdout;}
const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-time-acceptance-'));
try {
 const source=fs.readFileSync(path.join(here,row.source),'utf8'),ts=fs.readFileSync(path.join(here,row.counterpart),'utf8');
 assertMonotonicTimeSource(source,row,'native');assertMonotonicTimeSource(ts,row,'typescript');
 const emitted=run(process.env.TSRUST,['emit',path.join(here,row.source)]);assertABI(emitted,row,{source});
 const rust=path.join(directory,'env.rs'),typed=path.join(directory,'env.ts'),config=path.join(directory,'tsconfig.json');
 fs.writeFileSync(rust,emitted.replace('fn main()','fn source_main()')+'\n'+adaptRustDriver('',row,{emitted,source}));
 fs.writeFileSync(typed,ts+'\n'+adaptTSDriver('',row));
 fs.writeFileSync(config,JSON.stringify({extends:path.join(here,'tsconfig.json'),files:[typed],compilerOptions:{outDir:directory}}));
 run('rustup',['run','1.96.1','rustc','--edition=2024','-Copt-level=3','-Coverflow-checks=on','-Awarnings',rust,'-o',path.join(directory,'env')]);
 run(process.execPath,[path.join(here,'node_modules/typescript/bin/tsc'),'--project',config]);
 const env=process.env;
 for(const [n,seed] of [[0,0],[1,0],[3,7],[137,1000000],[20000,7]]) {
  const expected=expectedMonotonicTimeObservation(n,seed);
  for(const [engine,command,prefix] of [['native',path.join(directory,'env'),[]],['node-tsc',process.execPath,[path.join(directory,'env.js')]]]) {
   const result=JSON.parse(run(command,[...prefix,String(n),String(seed),'1','0'],env));
   assert.equal(result.checksum,expected.checksum);assert.equal(result.exactBits,expected.exactBits);assert.deepEqual(result.durations,[]);
   console.log(JSON.stringify({id:row.id,engine,n,seed,status:'pass',timed:false}));
  }
 }
} finally {fs.rmSync(directory,{recursive:true,force:true});}
