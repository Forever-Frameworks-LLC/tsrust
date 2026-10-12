import {streamInput,nativeStreamAllocation,directStreamAllocation,nodeStreamAllocation} from './json-stream-allocation.mjs';
import {nativeEncodingAllocation,directEncodingAllocation,nodeEncodingAllocation} from './json-encoding-allocation.mjs';
// Focused numeric and Unicode-array allocation evidence; no elapsed-time measurements.
// Native requested-byte counters and Node post-GC heap deltas are distinct metrics.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import assert from 'node:assert/strict';
import {adaptJsonAllocation,directJsonAllocation,nodeJsonAllocation,jsonAllocationInput,jsonAllocationChecksum,jsonAllocationKinds} from './json-allocation-adapter.mjs';
import {prepareNativeDependencies} from './native-dependencies.mjs';
import {compilerSourceInputs} from './compiler-source-inputs.mjs';
const selection=process.argv.slice(2);
if(selection.length>1||selection.length===1&&!selection[0].startsWith('--only='))throw Error('Usage: json-allocations.mjs [--only=kind,kind]');
const kinds=selection.length?selection[0].slice(7).split(','):jsonAllocationKinds;
if(new Set(kinds).size!==kinds.length||kinds.some(kind=>!jsonAllocationKinds.includes(kind)))throw Error('Unknown or duplicate JSON allocation kind');
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
const driver=fs.readFileSync(here+'/allocation-driver.rs','utf8');
const nodeDriver=fs.readFileSync(here+'/allocation-driver.ts','utf8');
const build=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-json-allocations-'));
const sources=[],rows=[];
try {
 for(const kind of kinds){
  const writer=kind.startsWith('stream_write_');
  const id=`json_${kind}_array`,sourceId=kind==='raw_selective'?id:kind.startsWith('raw_')?'json_raw_array':kind.startsWith('encode_dynamic_')?'json_encode_dynamic_array':kind==='stream_owned'?'../workloads/native_jsonl_typed':kind==='stream_borrowed'?'../workloads/native_jsonl_borrowed':kind.startsWith('dynamic_')?'json_dynamic_array':['typed_invalid','typed_mismatch'].includes(kind)?'json_typed_strings_array':id,file=here+`/allocations/${sourceId}.tsr`;
  const streaming=kind.startsWith('stream_');
  const sizes=streaming?[0,1,10000]:kind.includes('_')?[0,1,1000]:[0,1,10000];
  const source=fs.readFileSync(file,'utf8'),counterpart=fs.readFileSync(here+`/allocations/${writer?'json_stream_write_owned_array':sourceId}.ts`,'utf8');
  const emitted=run(compiler,['emit',file]);
  const dependencies=prepareNativeDependencies({emitted,compiler,source:file,build,rustVersion:pins.rust,targetDirectory:target,env});
  const encoding=kind.startsWith('encode_');
  const generated=streaming?nativeStreamAllocation(emitted,driver,writer):encoding?nativeEncodingAllocation(emitted,driver,kind):adaptJsonAllocation(emitted,driver,kind),direct=streaming?directStreamAllocation(driver,writer):encoding?directEncodingAllocation(driver,kind):directJsonAllocation(driver,kind),typescript=streaming?nodeStreamAllocation(counterpart,nodeDriver,writer):encoding?nodeEncodingAllocation(counterpart,nodeDriver):nodeJsonAllocation(counterpart,nodeDriver,kind);
  const commands=[];
  for(const [engine,text] of [['native',generated],['directRust',direct]]){
   const stem=path.join(build,`${id}_${engine}`);fs.writeFileSync(stem+'.rs',text);
   const args=['run',pins.rust,'rustc',...dependencies.args,'--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',stem+'.rs','-o',stem];
   run('rustup',args);commands.push(['rustup',...args]);
  }
  const stem=path.join(build,id);fs.writeFileSync(stem+'.ts',typescript);fs.writeFileSync(stem+'.json',JSON.stringify({extends:here+'/tsconfig.json',files:[stem+'.ts'],compilerOptions:{outDir:build}}));
  const tsc=[here+'/node_modules/typescript/bin/tsc','--project',stem+'.json'];run(process.execPath,tsc);commands.push([process.execPath,...tsc]);
  sources.push({id,sourceId,sizes,sourceSha256:hash(source),counterpartSha256:hash(counterpart),emittedSha256:hash(emitted),instrumentedSha256:hash(generated),directRustSha256:hash(direct),instrumentedTypeScriptSha256:hash(typescript),dependencies:dependencies.evidence,commands});
  for(const n of sizes){
   const payload=writer?'':streaming?streamInput(n):jsonAllocationInput(n,kind);
   const input=streaming?path.join(build,`${id}-${n}.jsonl`):payload,expected=writer?7+n:streaming?7+42*n:jsonAllocationChecksum(n,kind),samples=[];
   if(streaming)fs.writeFileSync(input,payload);
   for(let sample=0;sample<3;sample++){
    const verifyFile=()=>{if(writer)assert.equal(fs.readFileSync(input,'utf8'),'{"active":true,"name":"hello","values":[20,17]}\n'.repeat(n));};
    const native=JSON.parse(run(stem+'_native',[String(n),input]));verifyFile();
    const directRust=JSON.parse(run(stem+'_directRust',[String(n),input]));verifyFile();
    const node=JSON.parse(run(process.execPath,['--expose-gc',stem+'.js',String(n),input]));verifyFile();
    for(const result of [native,directRust]){assert.equal(result.checksum,expected);assert.equal(result.liveBytesAfterOutputDrop,0);assert.equal(result.liveBytesAfterDrop,0);if(!encoding)assert.equal(result.inputRequestedBytes,Buffer.byteLength(input));if(streaming||kind.endsWith('_invalid')||kind.endsWith('_mismatch'))assert.equal(result.retainedRequestedBytes,0);}
    assert.equal(node.checksum,expected);samples.push({native,directRust,node});
   }
   for(const engine of ['native','directRust'])for(const key of ['allocations','reallocations','deallocationsDuringWork','requestedBytes','retainedRequestedBytes','peakRequestedBytes'])assert.equal(new Set(samples.map(row=>row[engine][key])).size,1,`${id}/${n}/${engine}/${key}`);
   rows.push({id,n,payloadSha256:hash(payload),payloadBytes:Buffer.byteLength(payload),inputSha256:hash(input),inputBytes:Buffer.byteLength(input),expected,samples});
  }
  console.error(`Verified ${id}: three sizes, three fresh-process samples, native/direct Rust/Node`);
 }
 const data={schemaVersion:1,scope:'json-allocation',timestamp:new Date().toISOString(),timingSamples:0,samplesPerCase:3,kinds,semantics:'Selective raw cases retain only the numeric payload slice from a fixed envelope; ignored strings are still validated. Node uses JSON.parse plus fixed-envelope source slicing; this is not a general raw-field parser. Raw cases validate numeric arrays, Unicode and escaped string arrays, or 32-level nested arrays and retain only the original source: native pointer identity is checked, direct Serde IgnoredAny validates the same finite fixture and returns its input slice, Node JSON.parse discards its tree and returns the original text. Direct validation does not enforce native resource budgets. Raw checksum independently counts source bytes; zero output heap excludes preallocated source and does not imply zero stack storage. Valid arrays of exact integers 7 through n+6, or n pairs of Unicode and escaped strings (12 decoded UTF-8 bytes per pair), nested integer pairs, or records selecting nested items while ignoring a payload of strings and objects. Nested checksums sum 2n integers starting at 7. Deep inputs have 31 singleton wrappers around a numeric array, totaling 32 container levels; each wrapper shape is checked after sampling. Failure inputs contain n strings followed by invalid syntax or a number incompatible with string[]. Failure probes return 1 only on rejection and drop errors before sampling counters; no output heap bytes may remain. Native bounded decoding; direct serde_json baseline without native validation limits; Node dynamic parse or checked typed array. Encoding cases prepare numeric/Unicode arrays, mixed-field records or deeply nested dynamic values outside counters and compare exact JSON output bytes after sampling; inputRequestedBytes measures owned input heap storage. JSONL streaming cases open/read/close once over n records, use 8192-byte file buffers, and return 7+42n; inputRequestedBytes counts only the path, while file buffers and record allocation are inside counters. Writer cases open once and stage each record before buffered output; exact file bytes from each engine are verified outside counters. Native lines enforce limits; direct Rust assumes this valid bounded fixture. Decoding input is retained outside operation counters. Native counts requested bytes; Node reports post-GC heap deltas, not comparable allocation counts.',machine:{cpu:os.cpus()[0]?.model,architecture:os.arch(),release:os.release()},versions:{node:process.version,rust:rustVersion,typescript:pins.typescript,components:process.versions},sources,rows,driverSha256:hash(driver),nodeDriverSha256:hash(nodeDriver),adapterSha256:hash(fs.readFileSync(here+'/json-allocation-adapter.mjs')),streamAdapterSha256:hash(fs.readFileSync(here+'/json-stream-allocation.mjs')),encodingAdapterSha256:hash(fs.readFileSync(here+'/json-encoding-allocation.mjs')),harnessSha256:hash(fs.readFileSync(fileURLToPath(import.meta.url))),compilerInputs:compilerSourceInputs(root+'/crates/tsrust-compiler/src')};
 console.log(JSON.stringify(data,null,2));
} finally {fs.rmSync(build,{recursive:true,force:true});}
