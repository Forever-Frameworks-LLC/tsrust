// Encoding latency: exact outputs, prepared inputs, native/direct Rust/Node.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
import {nativeEncodingAllocation,directEncodingAllocation,nodeEncodingAllocation} from './json-encoding-allocation.mjs';
import {jsonAllocationInput,jsonAllocationChecksum} from './json-allocation-adapter.mjs';
import {prepareNativeDependencies} from './native-dependencies.mjs';
import {compilerSourceInputs} from './compiler-source-inputs.mjs';
import {validateEncodingTiming} from './json-encoding-timing-contract.mjs';
const kinds=['encode_typed','encode_dynamic','encode_typed_strings','encode_dynamic_strings','encode_dynamic_deep','encode_dynamic_object','encode_typed_object'];
if(process.argv.length!==2)throw Error('Usage: json-encoding-timing.mjs');
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

const driver=fs.readFileSync(here+'/json-encoding-timing-driver.rs','utf8');
const nodeDriver=fs.readFileSync(here+'/json-encoding-timing-driver.ts','utf8');
const build=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-json-encoding-timing-'));
const sources=[],rows=[];
try {
 for(const kind of kinds){
  const id=`json_${kind}_array`,sourceId=kind.startsWith('encode_dynamic_')?'json_encode_dynamic_array':id,file=here+`/allocations/${sourceId}.tsr`;
  const source=fs.readFileSync(file,'utf8'),counterpart=fs.readFileSync(here+`/allocations/${sourceId}.ts`,'utf8');
  const emitted=run(compiler,['emit',file]);
  const dependencies=prepareNativeDependencies({emitted,compiler,source:file,build,rustVersion:pins.rust,targetDirectory:target,env});
  const generated=nativeEncodingAllocation(emitted,driver,kind),direct=directEncodingAllocation(driver,kind),typescript=nodeEncodingAllocation(counterpart,nodeDriver);
  const commands=[];
  for(const [engine,text] of [['native',generated],['directRust',direct]]){
   const stem=path.join(build,`${id}_${engine}`);fs.writeFileSync(stem+'.rs',text);
   const args=['run',pins.rust,'rustc',...dependencies.args,'--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',stem+'.rs','-o',stem];
   run('rustup',args);commands.push(['rustup',...args]);
  }
  const stem=path.join(build,id);fs.writeFileSync(stem+'.ts',typescript);fs.writeFileSync(stem+'.json',JSON.stringify({extends:here+'/tsconfig.json',files:[stem+'.ts'],compilerOptions:{outDir:build}}));
  const tsc=[here+'/node_modules/typescript/bin/tsc','--project',stem+'.json'];run(process.execPath,tsc);commands.push([process.execPath,...tsc]);
  sources.push({id,sourceId,sourceSha256:hash(source),counterpartSha256:hash(counterpart),emittedSha256:hash(emitted),instrumentedSha256:hash(generated),directRustSha256:hash(direct),instrumentedTypeScriptSha256:hash(typescript),dependencies:dependencies.evidence,commands});
  for(const n of [0,1,1000]){
   const input=jsonAllocationInput(n,kind),expected=jsonAllocationChecksum(n,kind),samples=[];
   const engines=[['native',stem+'_native',[]],['directRust',stem+'_directRust',[]],['node',process.execPath,[stem+'.js']]];
   for(let processIndex=0;processIndex<3;processIndex++){
    for(const [engine,command,args] of [...engines.slice(processIndex),...engines.slice(0,processIndex)]){
     const result=JSON.parse(run(command,[...args,String(n),input]));validateEncodingTiming(result,expected);
     samples.push({engine,processIndex,...result});
    }
   }
   rows.push({id,n,inputSha256:hash(input),inputBytes:Buffer.byteLength(input),expected,samples});
  }
  console.error(`Measured ${id}: three sizes, three processes per engine`);
 }
 const files=['json-encoding-timing.mjs','json-encoding-timing-contract.mjs','json-encoding-timing-driver.rs','json-encoding-timing-driver.ts','json-encoding-allocation.mjs','json-allocation-adapter.mjs','allocations/json-object-encoding-baseline.rs'];
 console.log(JSON.stringify({schemaVersion:1,scope:'json-encoding-timing',timestamp:new Date().toISOString(),protocol:{processes:3,warmups:8,samples:31,iterationsPerBatch:100,cold:'first encoding after input preparation; not process startup',retention:'each output retained until replaced; last output retained through timer stop and exact verification; reclamation follows each runtime',timed:'encoding and replacement/disposal; input preparation and output validation excluded',limits:'native bounded checked encoding; direct Serde and Node assume valid fixture; no RSS measurement'},machine:{cpu:os.cpus()[0]?.model,architecture:os.arch(),release:os.release()},versions:{node:process.version,rust:rustVersion,typescript:pins.typescript},sources,rows,inputs:Object.fromEntries(files.map(file=>[file,hash(fs.readFileSync(here+'/'+file))])),compilerInputs:compilerSourceInputs(root+'/crates/tsrust-compiler/src')},null,2));
}finally{fs.rmSync(build,{recursive:true,force:true});}
