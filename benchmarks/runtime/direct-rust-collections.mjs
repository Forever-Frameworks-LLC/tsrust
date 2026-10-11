import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import crypto from 'node:crypto';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {expectedCollectionObservation, assertCollectionSource} from './collections-contract.mjs';

const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../..');
const verify=process.argv.includes('--verify');
if(process.argv.slice(2).some(arg=>arg!=='--verify'))throw Error('Only --verify is supported');
const manifest=JSON.parse(fs.readFileSync(path.join(here,'manifest.json')));
const pins=JSON.parse(fs.readFileSync(path.join(here,'versions.json')));
const hash=bytes=>crypto.createHash('sha256').update(bytes).digest('hex');
const inputs={};
const read=name=>{const bytes=fs.readFileSync(path.join(root,name));inputs[name]=hash(bytes);return bytes.toString();};
const commands=[];
function run(command){
 commands.push(command);
 const result=spawnSync(command[0],command.slice(1).map(String),{cwd:root,encoding:'utf8',timeout:120000});
 if(result.error||result.status!==0)throw Error(result.error??result.stderr);
 return result.stdout.trim();
}
const source=read('benchmarks/runtime/direct-rust-collections.rs');
let observer=read('benchmarks/runtime/slice-observation.rs');
// The direct Rust result uses Vec only. Remove generated-runtime-only adapters.
for(const name of ['Array','NumericArray']){
 const pattern=new RegExp(`impl<T:[^\\n]+> SliceTree for ${name}<T> \\{\\n[^\\n]+\\n\\}`,'g');
 if([...observer.matchAll(pattern)].length!==1)throw Error(`Observer adapter drift: ${name}`);
 observer=observer.replace(pattern,'');
}
const timing=read('benchmarks/runtime/slice-driver.rs').replaceAll('__FLOATING__','false');
const allocation=read('benchmarks/runtime/allocation-driver.rs');
for(const file of ['direct-rust-collections.mjs','collections-contract.mjs','source-contract-tokens.mjs','slice-observation.mjs','manifest.json','versions.json'])read(`benchmarks/runtime/${file}`);
const build=path.join(root,'target/runtime-comparisons/direct-rust-collections');fs.mkdirSync(build,{recursive:true});
const cases=[[0,0],[1,7],[2,19],[17,3],[2048,7]];
const rows=[];
for(const [id,entry,oracle,guard] of [
 ...['deque_clone','heap','ordered_set','ordered_map'].map(entry=>['native_'+entry,entry,expectedCollectionObservation,assertCollectionSource]),
]){
 const workload=manifest.workloads.find(row=>row.id===id);
 if(workload.iterations!==2048||manifest.seed!==7||manifest.processes!==3||manifest.warmups!==8||manifest.samples!==31)throw Error('Protocol drift');
 for(const [file,kind] of [[workload.source,'native'],[workload.counterpart,'typescript']])guard(read(`benchmarks/runtime/${file}`),workload,kind);
 const body=`${source}\n${observer}\nfn function0(n:i64, seed:i64)->Vec<Vec<i64>> { ${entry}(n,seed) }\n`;
 const row={workload:id,sourceSha256:workload.sourceSha256,counterpartSha256:workload.counterpartSha256,correctness:[],execution:[],profiles:[],artifacts:[]};
 for(const level of verify?['0','3']:['3']){
  const input=path.join(build,`${id}-O${level}.rs`),binary=input.slice(0,-3);
  fs.writeFileSync(input,body+timing);
  run(['rustup','run',pins.rust,'rustc','--edition=2024','-C',`opt-level=${level}`,'-C','overflow-checks=on','-A','warnings',input,'-o',binary]);
  row.artifacts.push({level,sourceSha256:hash(body+timing),binarySha256:hash(fs.readFileSync(binary))});
  for(const [n,seed] of cases){
   const actual=JSON.parse(run([binary,n,seed,0,1]));const expected=oracle(id,n,seed,'collections');
   if(actual.checksum!==expected.checksum||actual.exactBits!==expected.exactBits)throw Error(`Oracle mismatch ${id}/${n}/${seed}/O${level}`);
   row.correctness.push({n,seed,level,checksum:actual.checksum,exactBits:actual.exactBits});
  }
  if(!verify)for(let processIndex=0;processIndex<manifest.processes;processIndex++){
   const actual=JSON.parse(run([binary,2048,7,manifest.warmups,manifest.samples]));const expected=oracle(id,2048,7,'collections');
   if(actual.checksum!==expected.checksum||actual.exactBits!==expected.exactBits||actual.durations.length!==31||actual.durations.some(x=>!Number.isFinite(x)||x<=0))throw Error('Invalid timing result');
   row.execution.push({processIndex,...actual});
  }
 }
 const input=path.join(build,`${id}-profile.rs`),binary=input.slice(0,-3);fs.writeFileSync(input,body+allocation);
 run(['rustup','run',pins.rust,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',input,'-o',binary]);
 row.profileSourceSha256=hash(body+allocation);row.profileBinarySha256=hash(fs.readFileSync(binary));
 for(let sample=0;sample<3;sample++){
  const actual=JSON.parse(run([binary,2048]));
  if(actual.checksum!==oracle(id,2048,7,'collections').checksum||actual.liveBytesAfterDrop!==0||actual.liveBytesAfterOutputDrop!==0)throw Error('Invalid allocation result');
  row.profiles.push(actual);
 }
 if(row.profiles.some(sample=>JSON.stringify(sample)!==JSON.stringify(row.profiles[0])))throw Error('Nondeterministic allocation profile');
 rows.push(row);console.log(`Verified direct Rust ${id}${verify?'':' and measured 93 samples'}`);
}
for(const [name,digest] of Object.entries(inputs))if(hash(fs.readFileSync(path.join(root,name)))!==digest)throw Error(`Input changed during comparison: ${name}`);
const result={kind:'direct-rust-collection-comparison',timestamp:new Date().toISOString(),verifyOnly:verify,measurementScope:verify?'correctness-only':'execution-only',sourceCommit:run(['git','rev-parse','HEAD']),machine:{platform:os.platform(),release:os.release(),architecture:os.arch(),cpu:os.cpus()[0]?.model},rustVersion:run(['rustup','run',pins.rust,'rustc','--version']),protocol:{processes:verify?0:3,warmups:verify?0:8,samples:verify?0:31,iterations:2048,seed:7},scope:'Handwritten Rust over standard VecDeque, BinaryHeap, BTreeSet and BTreeMap with Vec results and independent existing output oracles. Same operation scenarios; different wrappers and result representations. Unused input string omitted; native input adaptation is outside timing. Allocation differences include result representation, not only container storage.',inputs,commands,rows};
const destination=path.join(here,'diagnostics',`${result.timestamp.replace(/[:.]/g,'-')}-direct-rust-collections${verify?'-verify':''}.json`);
fs.writeFileSync(destination,JSON.stringify(result,null,2)+'\n',{flag:'wx'});console.log(`Saved ${destination}`);
