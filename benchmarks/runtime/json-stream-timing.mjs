import {measureProcessRSS} from './process-rss.mjs';
// Decoding latency and RSS: checked shapes and independent result oracles.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
import {nativeStreamAllocation,directStreamAllocation,nodeStreamAllocation,streamInput} from './json-stream-allocation.mjs';

import {prepareNativeDependencies} from './native-dependencies.mjs';
import {compilerSourceInputs} from './compiler-source-inputs.mjs';
import {validateEncodingTiming} from './json-encoding-timing-contract.mjs';
const kinds=['stream_owned','stream_borrowed','stream_write_owned','stream_write_borrowed'];
const rss=process.argv.length===3&&process.argv[2]==='--rss';
if(process.argv.length!==2&&!rss)throw Error('Usage: json-stream-timing.mjs [--rss]');
const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../..');
const pins=JSON.parse(fs.readFileSync(here+'/versions.json'));
const compiler=process.env.TSRUST,target=process.env.CARGO_TARGET_DIR;
if(!compiler||!target||!path.isAbsolute(compiler)||!path.isAbsolute(target))throw Error('Set absolute TSRUST and shared CARGO_TARGET_DIR');
assert.equal(process.version,pins.node,'Pinned Node version');
const env={...process.env,CARGO_INCREMENTAL:'0'};delete env.NODE_OPTIONS;
const run=(command,args)=>execFileSync(command,args,{env,encoding:'utf8',timeout:60000,maxBuffer:16*1024*1024});
const rustVersion=run('rustup',['run',pins.rust,'rustc','--version']).trim();assert.equal(rustVersion.split(' ')[1],pins.rust);
assert.equal(JSON.parse(fs.readFileSync(here+'/node_modules/typescript/package.json')).version,pins.typescript);
const hash=data=>createHash('sha256').update(data).digest('hex');

const driver=fs.readFileSync(here+(rss?'/json-encoding-rss-driver.rs':'/json-stream-timing-driver.rs'),'utf8');
const nodeDriver=fs.readFileSync(here+(rss?'/json-encoding-rss-driver.ts':'/json-stream-timing-driver.ts'),'utf8');
const build=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-json-stream-timing-'));
const sources=[],rows=[];
try {
 for(const kind of kinds){
  const writer=kind.startsWith("stream_write_");
  const id=`json_${kind}_array`,sourceId=kind==='stream_owned'?'../workloads/native_jsonl_typed':kind==='stream_borrowed'?'../workloads/native_jsonl_borrowed':id,file=here+`/allocations/${sourceId}.tsr`;
  const source=fs.readFileSync(file,'utf8'),counterpart=fs.readFileSync(here+`/allocations/${writer?"json_stream_write_owned_array":sourceId}.ts`,'utf8');
  const emitted=run(compiler,['emit',file]);
  const dependencies=prepareNativeDependencies({emitted,compiler,source:file,build,rustVersion:pins.rust,targetDirectory:target,env});
  const generated=nativeStreamAllocation(emitted,driver,writer),direct=directStreamAllocation(driver,writer),typescript=nodeStreamAllocation(counterpart,nodeDriver,writer);
  const commands=[];
  for(const [engine,text] of [['native',generated],['directRust',direct]]){
   const stem=path.join(build,`${id}_${engine}`);fs.writeFileSync(stem+'.rs',text);
   const args=['run',pins.rust,'rustc',...dependencies.args,'--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',stem+'.rs','-o',stem];
   run('rustup',args);commands.push(['rustup',...args]);
  }
  const stem=path.join(build,id);fs.writeFileSync(stem+'.ts',typescript);fs.writeFileSync(stem+'.json',JSON.stringify({extends:here+'/tsconfig.json',files:[stem+'.ts'],compilerOptions:{outDir:build}}));
  const tsc=[here+'/node_modules/typescript/bin/tsc','--project',stem+'.json'];run(process.execPath,tsc);commands.push([process.execPath,...tsc]);
  sources.push({id,sourceId,sourceSha256:hash(source),counterpartSha256:hash(counterpart),emittedSha256:hash(emitted),instrumentedSha256:hash(generated),directRustSha256:hash(direct),instrumentedTypeScriptSha256:hash(typescript),dependencies:dependencies.evidence,commands});
  for(const n of [0,1,10000]){
   const payload=writer?"":streamInput(n),input=path.join(build,`${id}-${n}.jsonl`),expected=writer?7+n:7+42*n,samples=[];
   fs.writeFileSync(input,payload);
   const engines=[['native',stem+'_native',[]],['directRust',stem+'_directRust',[]],['node',process.execPath,[stem+'.js']]];
   for(let processIndex=0;processIndex<3;processIndex++){
    for(const [engine,command,args] of [...engines.slice(processIndex),...engines.slice(0,processIndex)]){
     const resource=rss?measureProcessRSS(command,[...args,String(n),input],env):null;
     const result=JSON.parse(resource?resource.stdout:run(command,[...args,String(n),input]));
     if(rss){assert.equal(result.checksum,expected);result.peakRssBytes=resource.peakRssBytes;}else validateEncodingTiming(result,expected,1);
     if(writer)assert.equal(fs.readFileSync(input,"utf8"),'{"active":true,"name":"hello","values":[20,17]}\n'.repeat(n));
     samples.push({engine,processIndex,...result});
    }
   }
   rows.push({id,n,inputSha256:hash(input),inputBytes:Buffer.byteLength(input),payloadSha256:hash(payload),payloadBytes:Buffer.byteLength(payload),expected,samples});
  }
  console.error(`Measured ${id}: three sizes, three processes per engine`);
 }
 const files=['process-rss.py','process-rss.mjs','json-encoding-rss-driver.rs','json-encoding-rss-driver.ts','json-stream-timing.mjs','json-encoding-timing-contract.mjs','json-stream-timing-driver.rs','json-stream-timing-driver.ts','json-stream-allocation.mjs','allocations/json-stream-baseline.rs','allocations/json-writer-baseline.rs'];
 console.log(JSON.stringify({schemaVersion:1,scope:rss?'json-stream-rss':'json-stream-timing',timestamp:new Date().toISOString(),protocol:rss?{processes:3,operationsPerProcess:1,metric:'macOS child ru_maxrss in bytes; fresh monitor per child',scope:'whole process including startup, prepared input, retained output, shape/checksum observer and runtime overhead; not incremental heap; no warmup or forced GC'}:{processes:3,warmups:8,samples:31,iterationsPerBatch:1,cold:'first complete file pass; OS page cache uncontrolled; not disk cold-cache latency',retention:'read records are consumed one at a time; writer stages each record; final checksum retained; files verified after process; reclamation follows each runtime',timed:'file open, record processing, flush/close inside; fixture preparation and final output verification outside',limits:'native bounded JSONL with 8192-byte buffers; direct Rust and Node assume valid bounded fixture; no durability sync; no RSS measurement'},machine:{cpu:os.cpus()[0]?.model,architecture:os.arch(),release:os.release()},versions:{node:process.version,rust:rustVersion,typescript:pins.typescript},sources,rows,inputs:Object.fromEntries(files.map(file=>[file,hash(fs.readFileSync(here+'/'+file))])),compilerInputs:compilerSourceInputs(root+'/crates/tsrust-compiler/src')},null,2));
}finally{fs.rmSync(build,{recursive:true,force:true});}
