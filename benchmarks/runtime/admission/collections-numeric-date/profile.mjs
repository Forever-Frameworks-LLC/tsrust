// Root grants the build/profile lease. This driver never invokes Cargo or fetch.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {waitForQuietMachine} from '../../quiet-machine.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../../../..');
const names=['endpoints','set-snapshot','deque-snapshot','heap-sorted-vec','heap-sort-snapshot','heap-sorted-vec-seeded','heap-sort-snapshot-seeded','heap-sorted-vec-reverse','heap-sort-snapshot-reverse','heap-sorted-vec-duplicates','heap-sort-snapshot-duplicates','float-parse','radix-parse','checked','date-time'];
const sizes=[0,1,8,1024];
const digest=values=>values.reduce((sum,value)=>BigInt.asUintN(64,sum*31n+BigInt(value)),7n).toString();
const bits=value=>{const buffer=Buffer.alloc(8);buffer.writeDoubleLE(value);return buffer.readBigUInt64LE();};
function expected(name,size){
 const values=Array.from({length:size},(_,index)=>index);
 if(name.startsWith('heap-')&&['seeded','reverse','duplicates'].some(pattern=>name.endsWith('-'+pattern))){const pattern=name.split('-').at(-1);return digest(values.map(i=>pattern==='seeded'?(i*73+19)%size:pattern==='duplicates'?(i*73+19)%7:i).sort((a,b)=>a-b));}
 switch(name){
  case 'endpoints':return digest([size?0:-1,size?size-1:-1,size?0:-1,size?size-1:-1]);
  case 'set-snapshot':case 'heap-sorted-vec':case 'heap-sort-snapshot':return digest(values);
  case 'deque-snapshot':return digest(size?[...values.slice(1),0]:[]);
  case 'float-parse':return digest([1.5,-0,Infinity,-0].map(bits));
  case 'radix-parse':return digest([255,-4,35]);
  case 'checked':return digest([130,116,861,17,4]);
  case 'date-time':return digest([4,1,366,23,59,59,999999999]);
  default:throw Error(`Unknown kernel ${name}`);
 }
}
function validate(rows,allocation){
 assert.equal(rows.length,names.length*sizes.length*(allocation?1:31));
 const seen=new Set();
 for(const row of rows){
  assert(names.includes(row.name)&&sizes.includes(row.size));
  assert.equal(row.kind,allocation?'allocation':'timing');assert.equal(row.checksum,expected(row.name,row.size));
  const key=`${row.name}/${row.size}/${allocation?'allocation':row.sample}`;assert(!seen.has(key));seen.add(key);
  if(allocation)for(const name of ['allocations','reallocations','bytes','peak'])assert(Number.isSafeInteger(row[name])&&row[name]>=0);
  else {assert(Number.isInteger(row.sample)&&row.sample>=0&&row.sample<31);assert.equal(row.operations,1024);assert(Number.isSafeInteger(row.ns)&&row.ns>0);}
 }
}
function selfCheck(){
 for(const name of names)for(const size of sizes){assert.match(expected(name,size),/^\d+$/);assert.equal(expected('heap-sorted-vec',size),expected('heap-sort-snapshot',size));for(const pattern of ['seeded','reverse','duplicates'])assert.equal(expected('heap-sorted-vec-'+pattern,size),expected('heap-sort-snapshot-'+pattern,size));}
 for(const allocation of [false,true]){
  const rows=names.flatMap(name=>sizes.flatMap(size=>Array.from({length:allocation?1:31},(_,sample)=>({kind:allocation?'allocation':'timing',name,size,sample,operations:1024,ns:1,allocations:0,reallocations:0,bytes:0,peak:0,checksum:expected(name,size)}))));
  validate(rows,allocation);assert.throws(()=>validate(rows.slice(1),allocation));assert.throws(()=>validate(rows.map((row,index)=>index?row:{...row,checksum:'bad'}),allocation));assert.throws(()=>validate([...rows.slice(0,-1),rows[0]],allocation));
 }
}
selfCheck();
if(process.argv.includes('--self-check')){console.log('B14 admission oracle self-check passed');process.exit(0);}
assert.equal(process.version,'v24.11.1','Use pinned Node24.11.1');
const artifactArgument=process.argv.find(value=>value.startsWith('--artifacts='))?.slice(12);
assert(artifactArgument&&path.isAbsolute(artifactArgument),'Pass approved absolute Cargo JSON metadata');
const target=process.env.CARGO_TARGET_DIR;
assert(target&&path.isAbsolute(target),'Pass coordinated absolute CARGO_TARGET_DIR');
assert.equal(path.resolve(target),'/Users/seancotter/Desktop/dev/learning/tsrust/target','Use the single shared target');
const metadata=fs.readFileSync(artifactArgument,'utf8').split('\n').filter(line=>line.startsWith('{')).map(line=>JSON.parse(line));
assert.equal(metadata.at(-1)?.reason,'build-finished','Require terminal Cargo build-finished metadata');assert.equal(metadata.at(-1)?.success,true,'Require successful terminal Cargo result');
const artifacts=metadata.filter(row=>row.reason==='compiler-artifact'&&row.target.name==='chrono'&&row.filenames.some(file=>file.endsWith('.rlib')));
assert.equal(artifacts.length,1,'Require one admitted chrono artifact');
const artifact=artifacts[0],files=artifact.filenames.filter(file=>file.endsWith('.rlib'));assert.equal(files.length,1);assert.equal(String(artifact.profile.opt_level),'3');
const dependency=files[0];assert(path.isAbsolute(dependency)&&fs.statSync(dependency).isFile());assert(dependency.startsWith(`${target}${path.sep}`));
const lock=fs.readFileSync(path.join(root,'Cargo.lock'),'utf8').split('[[package]]').find(section=>/^\s*name = "chrono"\s*$/m.test(section));
assert(lock&&/^version = "0.4.45"$/m.test(lock));const checksum=lock.match(/^checksum = "([a-f0-9]{64})"$/m)?.[1];assert(checksum);
assert.equal(artifact.package_id,'registry+https://github.com/rust-lang/crates.io-index#chrono@0.4.45');
const compiler=fs.readFileSync(path.join(root,'crates/tsrust-compiler/src/lib.rs'),'utf8').match(/pub const RUST_BACKEND_VERSION: &str = "([^"]+)"/)[1];
const hash=file=>createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const sourceFiles=['reference.rs','profile.rs','profile.mjs','../../probes/counting-allocator.rs','../../quiet-machine.mjs','../../../../crates/tsrust-compiler/src/stdlib/collections/ordered_runtime.rs','../../../../crates/tsrust-compiler/src/stdlib/collections/ordered_key_runtime.rs','../../../../crates/tsrust-compiler/src/stdlib/date/runtime.rs','../../../../Cargo.lock'];
const sources=()=>Object.fromEntries(sourceFiles.map(file=>[file,hash(path.resolve(here,file))]));const before=sources(),dependencyHash=hash(dependency);
const scratch=fs.mkdtempSync(path.join(target,'collections-numeric-date-profile-'));
const run=(command,args)=>{const result=spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:180000,maxBuffer:16*1024*1024,env:{...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1'}});if(result.error||result.status!==0)throw Error(`${command}: ${result.error??result.stderr}`);return result.stdout;};
const records=(binary,args=[])=>run(binary,args).trim().split('\n').map(line=>JSON.parse(line));
try{
 const builds=[],binaries={};
 for(const allocation of [false,true]){
  const binary=path.join(scratch,`profile${allocation?'-alloc':''}`),args=['run',compiler,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on','--extern',`chrono=${dependency}`,'-L',`dependency=${path.dirname(dependency)}`];
  if(allocation)args.push('--cfg','feature="allocation-probe"');args.push(path.join(here,'profile.rs'),'-o',binary);
  const start=performance.now();run('rustup',args);builds.push({allocation,buildMs:performance.now()-start,binaryBytes:fs.statSync(binary).size,binarySha256:hash(binary)});binaries[allocation]=binary;
 }
 const correctness=records(binaries.false,['--check']);assert.deepEqual(correctness,[{kind:'correctness',passed:true}]);
 const allocations=[],measurements=[],quietLoads=[];
 for(let processIndex=0;processIndex<3;processIndex++){
  const counts=records(binaries.true);validate(counts,true);allocations.push({processIndex,records:counts});
  quietLoads.push(waitForQuietMachine({log:line=>process.stderr.write(`${line}\n`)}));
  const timing=records(binaries.false);validate(timing,false);measurements.push({processIndex,records:timing});
 }
 assert.deepEqual(sources(),before);assert.equal(hash(dependency),dependencyHash);
 console.log(JSON.stringify({scope:'B14 direct Rust backend admission; not generated API timings',allocationScope:'Prepared source containers and callback closure are retained outside counting. One complete query/snapshot, including full digest observation and output drop, is counted. Live bytes must return to the prepared baseline; peak is additional live bytes above that baseline. Source construction and final source destruction are excluded.',timingScope:'1024 complete queries per sample, including full digest and checksum assertion; input containers are prepared outside timing. No claim of isolated backend call latency.',compiler,node:process.version,sourceCommit:run('git',['rev-parse','HEAD']).trim(),protocol:{processes:3,warmups:8,samples:31,operationsPerSample:1024,separateAllocationBuild:true},sources:before,dependency:{path:dependency,sha256:dependencyHash,package:artifact.package_id,checksum,features:artifact.features,profile:artifact.profile},builds,correctness,quietLoads,allocations,measurements}));
}finally{fs.rmSync(scratch,{recursive:true,force:true});}
