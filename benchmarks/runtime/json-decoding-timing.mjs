import {measureProcessRSS} from './process-rss.mjs';
// Decoding latency and RSS: checked shapes and independent result oracles.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
import {adaptJsonAllocation,directJsonAllocation,nodeJsonAllocation} from './json-allocation-adapter.mjs';
import {jsonAllocationInput,jsonAllocationChecksum} from './json-allocation-adapter.mjs';
import {prepareNativeDependencies} from './native-dependencies.mjs';
import {compilerSourceInputs} from './compiler-source-inputs.mjs';
import {validateEncodingTiming} from './json-encoding-timing-contract.mjs';
const decodingKinds=['dynamic','typed','dynamic_strings','typed_strings','dynamic_nested','typed_nested','dynamic_selective','typed_selective','dynamic_invalid','typed_invalid','typed_mismatch','dynamic_deep'];
const selection=process.argv.slice(2);
if(new Set(selection).size!==selection.length||selection.some(arg=>!['--rss','--raw'].includes(arg)))throw Error('Usage: json-decoding-timing.mjs [--rss] [--raw]');
const rss=selection.includes('--rss'),raw=selection.includes('--raw');
const kinds=raw?['raw','raw_strings','raw_deep','raw_invalid']:decodingKinds;
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

const driver=fs.readFileSync(here+(rss?'/json-encoding-rss-driver.rs':'/json-decoding-timing-driver.rs'),'utf8');
const nodeDriver=fs.readFileSync(here+(rss?'/json-encoding-rss-driver.ts':'/json-encoding-timing-driver.ts'),'utf8');
const build=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-json-decoding-timing-'));
const sources=[],rows=[];
try {
 for(const kind of kinds){
  const id=`json_${kind}_array`,sourceId=raw?'json_raw_array':kind.startsWith('dynamic_')?'json_dynamic_array':['typed_invalid','typed_mismatch'].includes(kind)?'json_typed_strings_array':id,file=here+`/allocations/${sourceId}.tsr`;
  const source=fs.readFileSync(file,'utf8'),counterpart=fs.readFileSync(here+`/allocations/${sourceId}.ts`,'utf8');
  const emitted=run(compiler,['emit',file]);
  const dependencies=prepareNativeDependencies({emitted,compiler,source:file,build,rustVersion:pins.rust,targetDirectory:target,env});
  const generated=adaptJsonAllocation(emitted,driver,kind),direct=directJsonAllocation(driver,kind),typescript=nodeJsonAllocation(counterpart,nodeDriver,kind);
  const commands=[];
  for(const [engine,text] of [['native',generated],['directRust',direct]]){
   const stem=path.join(build,`${id}_${engine}`);fs.writeFileSync(stem+'.rs',text);
   const args=['run',pins.rust,'rustc',...dependencies.args,'--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',stem+'.rs','-o',stem];
   run('rustup',args);commands.push(['rustup',...args]);
  }
  const stem=path.join(build,id);fs.writeFileSync(stem+'.ts',typescript);fs.writeFileSync(stem+'.json',JSON.stringify({extends:here+'/tsconfig.json',files:[stem+'.ts'],compilerOptions:{outDir:build}}));
  const tsc=[here+'/node_modules/typescript/bin/tsc','--project',stem+'.json'];run(process.execPath,tsc);commands.push([process.execPath,...tsc]);
  sources.push({id,sourceId,sourceSha256:hash(source),counterpartSha256:hash(counterpart),emittedSha256:hash(emitted),instrumentedSha256:hash(generated),directRustSha256:hash(direct),instrumentedTypeScriptSha256:hash(typescript),dependencies:dependencies.evidence,commands});
  for(const n of (rss?[0,1,10000]:[0,1,1000])){
   const input=jsonAllocationInput(n,kind),expected=jsonAllocationChecksum(n,kind),samples=[];
   const engines=[['native',stem+'_native',[]],['directRust',stem+'_directRust',[]],['node',process.execPath,[stem+'.js']]];
   for(let processIndex=0;processIndex<3;processIndex++){
    for(const [engine,command,args] of [...engines.slice(processIndex),...engines.slice(0,processIndex)]){
     const resource=rss?measureProcessRSS(command,[...args,String(n),input],env):null;
     const result=JSON.parse(resource?resource.stdout:run(command,[...args,String(n),input]));
     if(rss){assert.equal(result.checksum,expected);result.peakRssBytes=resource.peakRssBytes;}else validateEncodingTiming(result,expected);
     samples.push({engine,processIndex,...result});
    }
   }
   rows.push({id,n,inputSha256:hash(input),inputBytes:Buffer.byteLength(input),expected,samples});
  }
  console.error(`Measured ${id}: three sizes, three processes per engine`);
 }
 const files=['process-rss.py','process-rss.mjs','json-encoding-rss-driver.rs','json-encoding-rss-driver.ts','json-decoding-timing.mjs','json-encoding-timing-contract.mjs','json-decoding-timing-driver.rs','json-encoding-timing-driver.ts','json-allocation-adapter.mjs','allocations/json-selective-baseline.rs'];
 console.log(JSON.stringify({schemaVersion:1,scope:raw?(rss?'json-raw-rss':'json-raw-timing'):(rss?'json-decoding-rss':'json-decoding-timing'),semantics:raw?'Native validates with resource budgets and borrows original source; direct Serde IgnoredAny validates without native budgets and returns input; Node parses and discards a dynamic tree, returning original text. Invalid inputs return a rejection marker. Input is prepared outside timing; raw source pointer identity and independent source-byte checksum are checked after timing.':'Checked decoding retains decoded output.',timestamp:new Date().toISOString(),protocol:rss?{processes:3,operationsPerProcess:1,metric:'macOS child ru_maxrss in bytes; fresh monitor per child',scope:'whole process including startup, prepared input, retained output, shape/checksum observer and runtime overhead; not incremental heap; no warmup or forced GC'}:{processes:3,warmups:8,samples:31,iterationsPerBatch:100,cold:'first decoding after input preparation; not process startup',retention:'each decoded output retained until replaced; last output retained through timer stop and shape/checksum verification; failure paths drop errors inside operation; reclamation follows each runtime',timed:'decoding and replacement/disposal; input preparation and output validation excluded',limits:'native bounded checked decoding; direct Serde lacks native depth/size validation; typed selective output excludes ignored fields while dynamic output retains them; failures return rejection marker; no RSS measurement'},machine:{cpu:os.cpus()[0]?.model,architecture:os.arch(),release:os.release()},versions:{node:process.version,rust:rustVersion,typescript:pins.typescript},sources,rows,inputs:Object.fromEntries(files.map(file=>[file,hash(fs.readFileSync(here+'/'+file))])),compilerInputs:compilerSourceInputs(root+'/crates/tsrust-compiler/src')},null,2));
}finally{fs.rmSync(build,{recursive:true,force:true});}
