// Root-approved admission only: no Cargo/fetch, no public source mutation.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {waitForQuietMachine} from '../../quiet-machine.mjs';
const here=path.dirname(fileURLToPath(import.meta.url));
const candidates=['double-box','single-box','stack-four'];
const kernels=candidate=>candidate==='double-box'?['ready','yield','ready-floor','yield-floor','channel-runtime','channel-floor']:['ready','yield'];
const cases=[[0,0],[1,7],[2,19],[17,3],[2048,7]];
function expected(n,seed,reverse=false){let sum=0n;for(let i=0;i<n;i++)for(const j of reverse?[3,2,1,0]:[0,1,2,3])sum=(sum*131n+BigInt(seed+i+j))%1000000007n;return Number(sum);}
function validate(rows,candidate,allocation=false,check=false){
 assert.equal(rows.length,kernels(candidate).length*(check?cases.length:1)+(check?1:0));const seen=new Set();
 for(const row of rows){
  assert.equal(row.candidate,candidate);
  if(row.kind==='contracts'){
   assert(check);assert(!seen.has('contracts'));seen.add('contracts');assert.deepEqual(row.ordered,[11,22,33,44]);
   for(const name of ['borrowedNonSendNotUnpin','immediateDrops','cleanupAfterCancelledDrain'])assert.equal(row[name],true);
   for(const[name,value]of Object.entries({failfastOutputDrops:1,failfastSiblingDrops:1,unpolledSiblingDrops:1,nestedSiblingDrops:1,escapedJobDrops:4,cancelledGroupDrops:1}))assert.equal(row[name],value);
   continue;
  }
  assert(kernels(candidate).includes(row.kernel));assert(Number.isSafeInteger(row.n)&&row.n>=0&&Number.isSafeInteger(row.seed));
  const key=`${row.kernel}/${row.n}/${row.seed}`;assert(!seen.has(key));seen.add(key);
  if(check){assert.equal(row.kind,'correctness');assert(cases.some(([n,seed])=>row.n===n&&row.seed===seed));assert.equal(row.checksum,expected(row.n,row.seed));}
  else{
   assert.equal(row.n,2048);assert.equal(row.seed,7);
   if(allocation){assert.equal(row.kind,'allocation');assert.equal(row.checksum,expected(row.n,row.seed));assert.equal(row.retained,0,'Query must restore live bytes to its warmed runtime baseline');for(const name of ['allocations','reallocations','requested','peak'])assert(Number.isSafeInteger(row[name])&&row[name]>=0);}
   else{assert.equal(row.kind,'timing');assert.equal(row.durations_ns.length,31);assert.equal(row.checksums.length,31);for(const value of row.durations_ns)assert(Number.isSafeInteger(value)&&value>0);for(const checksum of row.checksums)assert.equal(checksum,expected(row.n,row.seed));}
  }
 }
}
function quietRecord(processIndex,candidate,load){assert(Number.isFinite(load)&&load>=0);return {processIndex,candidate,load};}
function selfCheck(){
 assert.deepEqual(JSON.parse(JSON.stringify(quietRecord(2,'stack-four',1.25))),{processIndex:2,candidate:'stack-four',load:1.25});assert.throws(()=>quietRecord(0,'double-box',NaN));
 assert.equal(expected(0,7),0);assert.equal(expected(1,7),7*131**3+8*131**2+9*131+10);assert.notEqual(expected(17,3),expected(17,3,true));
 for(const candidate of candidates){
  const contract={kind:'contracts',candidate,ordered:[11,22,33,44],borrowedNonSendNotUnpin:true,immediateDrops:true,failfastOutputDrops:1,failfastSiblingDrops:1,unpolledSiblingDrops:1,nestedSiblingDrops:1,escapedJobDrops:4,cancelledGroupDrops:1,cleanupAfterCancelledDrain:true};
  const rows=[contract,...kernels(candidate).flatMap(kernel=>cases.map(([n,seed])=>({kind:'correctness',candidate,kernel,n,seed,checksum:expected(n,seed)})))];validate(rows,candidate,false,true);
  assert.throws(()=>validate(rows.slice(1),candidate,false,true));assert.throws(()=>validate(rows.map((row,index)=>index===1?{...row,checksum:expected(row.n,row.seed,true)+1}:row),candidate,false,true));assert.throws(()=>validate([{...contract,ordered:[44,33,22,11]},...rows.slice(1)],candidate,false,true));assert.throws(()=>validate([{...contract,failfastOutputDrops:0},...rows.slice(1)],candidate,false,true));
 }
}
selfCheck();
if(process.argv.includes('--self-check')){console.log('Task-group admission oracle self-check passed');process.exit(0);}
assert.equal(process.version,'v24.11.1');
const option=name=>process.argv.find(value=>value.startsWith(`--${name}=`))?.slice(name.length+3);
const sourceRoot=option('source-root'),artifactPath=option('artifacts'),target=process.env.CARGO_TARGET_DIR;
for(const[value,name]of[[sourceRoot,'source-root'],[artifactPath,'artifacts'],[target,'CARGO_TARGET_DIR']])assert(value&&path.isAbsolute(value),`Pass approved absolute ${name}`);
assert.equal(path.resolve(target),'/Users/seancotter/Desktop/dev/learning/tsrust/target');
const hash=file=>createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const canonicalFiles=['crates/tsrust-compiler/src/stdlib/futures/runtime.rs','crates/tsrust-compiler/src/emit/futures.rs','crates/tsrust-compiler/src/lib.rs','Cargo.lock',...['task_all_ready','task_all_yield','async_channel_await'].flatMap(id=>[`benchmarks/runtime/workloads/${id}.tsr`,`benchmarks/runtime/parity/${id}.rs`])];
const localFiles=['candidates.rs','profile.rs','profile.mjs','../../probes/counting-allocator.rs','../../quiet-machine.mjs'];
const sources=()=>Object.fromEntries([...canonicalFiles.map(file=>[`canonical:${file}`,hash(path.join(sourceRoot,file))]),...localFiles.map(file=>[`admission:${file}`,hash(path.resolve(here,file))])]);const before=sources();
const runtime=fs.readFileSync(path.join(sourceRoot,canonicalFiles[0]),'utf8');assert.match(runtime,/let mut futures=futures\.into_iter\(\)\.map\(\|future\|Some\(Box::pin\(future\)\)\)\.collect::<Vec<_>>\(\);/);assert.match(runtime,/scope\.drain\(false\)\.await;result/);
const emitter=fs.readFileSync(path.join(sourceRoot,canonicalFiles[1]),'utf8');assert(emitter.includes('Box::pin(async move{{{poll}}}) as std::pin::Pin<Box<dyn std::future::Future'));
const candidateSource=fs.readFileSync(path.join(here,'candidates.rs'),'utf8');assert.equal((candidateSource.match(/let scope=Scope::group\(\);/g)||[]).length,2);assert.equal((candidateSource.match(/scope\.drain\(false\)\.await;result/g)||[]).length,2);assert(candidateSource.includes('$slot.as_mut().set(None)'));assert(!/\bunsafe\b/.test(candidateSource));
const metadata=fs.readFileSync(artifactPath,'utf8').split('\n').filter(line=>line.startsWith('{')).map(line=>JSON.parse(line));assert.equal(metadata.at(-1)?.reason,'build-finished');assert.equal(metadata.at(-1)?.success,true);
const matched=metadata.filter(row=>row.reason==='compiler-artifact'&&row.target.name==='tokio'&&row.filenames.some(file=>file.endsWith('.rlib')));assert.equal(matched.length,1);const artifact=matched[0],files=artifact.filenames.filter(file=>file.endsWith('.rlib'));assert.equal(files.length,1);assert.equal(String(artifact.profile.opt_level),'3');
assert.equal(artifact.package_id,'registry+https://github.com/rust-lang/crates.io-index#tokio@1.53.3');for(const feature of ['rt-multi-thread','sync','time','macros'])assert(artifact.features.includes(feature));
const dependency=files[0];assert(path.isAbsolute(dependency)&&dependency.startsWith(`${target}${path.sep}`));const dependencyHash=hash(dependency);
const lock=fs.readFileSync(path.join(sourceRoot,'Cargo.lock'),'utf8').split('[[package]]').find(section=>/^\s*name = "tokio"\s*$/m.test(section));assert(lock&&/^version = "1.53.3"$/m.test(lock));const lockChecksum=lock.match(/^checksum = "([a-f0-9]{64})"$/m)?.[1];assert(lockChecksum);
const compiler=fs.readFileSync(path.join(sourceRoot,'crates/tsrust-compiler/src/lib.rs'),'utf8').match(/pub const RUST_BACKEND_VERSION: &str = "([^"]+)"/)[1];
const scratch=fs.mkdtempSync(path.join(target,'task-group-attribution-'));
const generatedRuntime=path.join(scratch,'runtime.rs');const end=runtime.lastIndexOf('}');assert(end>0&&runtime.slice(end+1).trim()==='');fs.writeFileSync(generatedRuntime,runtime.slice(0,end)+`\ninclude!(${JSON.stringify(path.join(here,'candidates.rs'))});\n`+runtime.slice(end));
const floorPaths={};for(const[kernel,id]of[['READY','task_all_ready'],['YIELD','task_all_yield'],['CHANNEL','async_channel_await']]){const raw=fs.readFileSync(path.join(sourceRoot,`benchmarks/runtime/parity/${id}.rs`),'utf8');const file=path.join(scratch,`floor-${kernel.toLowerCase()}.rs`);fs.writeFileSync(file,`mod floor_${kernel.toLowerCase()} {\n${raw}\npub async fn run(n:i64,seed:i64)->i64{function0(n,seed).await}\n}\n`);floorPaths[`TSRUST_TASK_GROUP_FLOOR_${kernel}`]=file;}
const run=(command,args,build=false)=>{const child=spawnSync(command,args,{cwd:sourceRoot,encoding:'utf8',timeout:build?180000:60000,maxBuffer:16*1024*1024,env:{...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1',TSRUST_TASK_GROUP_RUNTIME:generatedRuntime,...floorPaths}});if(child.error||child.status!==0)throw Error(`${command}: ${child.error??child.stderr}`);return child.stdout;};
const records=(binary,args=[])=>run(binary,args).trim().split('\n').map(line=>JSON.parse(line));
try{
 const builds=[],binaries={};
 for(const candidate of candidates)for(const allocation of [false,true]){
  const binary=path.join(scratch,`${candidate}${allocation?'-alloc':''}`),args=['run',compiler,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on','--extern',`tokio=${dependency}`,'-L',`dependency=${path.dirname(dependency)}`];
  if(candidate==='single-box')args.push('--cfg','candidate_single');if(candidate==='stack-four')args.push('--cfg','candidate_stack');if(allocation)args.push('--cfg','allocation_probe');
  args.push(path.join(here,'profile.rs'),'-o',binary);const start=performance.now();run('rustup',args,true);builds.push({candidate,allocation,buildMs:performance.now()-start,binaryBytes:fs.statSync(binary).size,binarySha256:hash(binary)});binaries[`${candidate}/${allocation}`]=binary;
 }
 const correctness=[];for(const candidate of candidates){const rows=records(binaries[`${candidate}/false`],['--check']);validate(rows,candidate,false,true);correctness.push({candidate,records:rows});}
 const measurements=[],allocations=[],quietLoads=[];
 for(let processIndex=0;processIndex<3;processIndex++)for(const candidate of candidates){
  const counts=records(binaries[`${candidate}/true`]);validate(counts,candidate,true);allocations.push({processIndex,candidate,records:counts});
  quietLoads.push(quietRecord(processIndex,candidate,waitForQuietMachine({log:line=>process.stderr.write(`${line}\n`)})));
  const rows=records(binaries[`${candidate}/false`]);validate(rows,candidate);measurements.push({processIndex,candidate,records:rows});
 }
 assert.deepEqual(sources(),before);assert.equal(hash(dependency),dependencyHash);
 console.log(JSON.stringify({scope:'B06 task-group attribution admission, not generated API timing or an accepted optimization',sourceRoot,sourceCommit:run('git',['rev-parse','HEAD']).trim(),compiler,node:process.version,protocol:{processes:3,warmups:8,samples:31,iterations:2048,seed:7,separateAllocationBuild:true,scope:'One runtime per process, outside timing. Every candidate retains exact current Scope::group, task-local scope and drain(false). Ready/yield inputs have four distinct async wrapper types and match the owned erased literal shape. Checksums observe every ordered result. Naked Tokio floors are unchanged source, wrapped only in a module; they intentionally omit structured scope semantics.'},allocationScope:'One complete warmed 2048-group/channel invocation, including input boxes/Vec, scope registration/drain, ordered output storage/observation/drop. Runtime construction/teardown and eight warmups excluded. Live bytes must return to warmed baseline. Stack-four additionally replaces dynamic scratch Vec with fixed slots/results; it is not an allocation-only A/B.',sources:before,dependency:{path:dependency,sha256:dependencyHash,package:artifact.package_id,features:artifact.features,profile:artifact.profile,lockChecksum},builds,correctness,quietLoads,allocations,measurements}));
}finally{fs.rmSync(scratch,{recursive:true,force:true});}
