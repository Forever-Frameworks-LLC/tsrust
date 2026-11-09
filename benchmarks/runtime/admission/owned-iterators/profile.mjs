// Admission only; never invokes Cargo, fetch or a public compiler build.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {waitForQuietMachine} from '../../quiet-machine.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../../../..');
const candidates=['static','finite-enum','boxed'];
const kernels=candidate=>['reduce','some','take-array','partial-drop',...(candidate==='static'?['flat-array','flat-iterator']:[])];
const cases=[[0,0],[1,7],[2,19],[17,3],[17,4],[4096,7]],sizes=[0,1,64,4096];
const digest=values=>Number(values.reduce((sum,value)=>(sum*131n+BigInt(value))%1000000007n,7n));
function expected(kernel,size,seed){
 const input=Array.from({length:size},(_,index)=>index+seed),selected=input.filter(value=>value%2===0).map(value=>value*3+1);
 if(kernel==='reduce')return digest(selected);
 if(kernel==='take-array')return digest(selected.slice(0,8));
 if(kernel==='partial-drop')return selected[0]??-1;
 if(kernel.startsWith('flat-'))return digest(input.slice(0,8).flatMap(value=>[value,value+1]));
 if(kernel==='some'){const first=selected.findIndex(value=>value>=seed*3+7);return first<0?selected.length*2:(first+1)*2+1;}
 throw Error('Unknown iterator kernel');
}
function validate(rows,candidate,allocation=false,check=false){
 const contracts=rows.filter(row=>row.kind==='contracts'),representations=rows.filter(row=>row.kind==='representation');
 if(check){assert.equal(contracts.length,1);assert.equal(representations.length,1);for(const name of ['nonCloneOwners','partialDrop','retryAfterError','errorsNotSkipped','consumingSomeEvery','borrowedFilterFind','ownedFlatMapForms','finiteOwnerJoin'])assert.equal(contracts[0][name],true);assert.equal(contracts[0].takeZeroPulls,0);assert.equal(contracts[0].surplusDrops,0);for(const name of ['staticBytes','enumBytes','boxedHandleBytes'])assert(Number.isSafeInteger(representations[0][name])&&representations[0][name]>0);assert.equal(representations[0].boxedAllocationNotIncluded,true);}
 else{assert.equal(contracts.length,0);assert.equal(representations.length,0);}
 const records=rows.filter(row=>!['contracts','representation'].includes(row.kind));assert.equal(records.length,kernels(candidate).length*(check?cases.length:sizes.length*2));const seen=new Set();
 for(const row of records){assert.equal(row.candidate,candidate);assert(kernels(candidate).includes(row.kernel));const key=`${row.kernel}/${row.size}/${row.seed}`;assert(!seen.has(key));seen.add(key);
  if(check){assert(cases.some(([size,seed])=>size===row.size&&seed===row.seed));assert.equal(row.kind,'correctness');assert.equal(row.checksum,expected(row.kernel,row.size,row.seed));}
  else{assert(sizes.includes(row.size));assert([7,8].includes(row.seed));if(allocation){assert.equal(row.kind,'allocation');assert.equal(row.checksum,expected(row.kernel,row.size,row.seed));assert.equal(row.retained,0);for(const name of ['allocations','reallocations','requested','peak'])assert(Number.isSafeInteger(row[name])&&row[name]>=0);}
   else{assert.equal(row.kind,'timing');assert.equal(row.queries,16);assert.equal(row.durations_ns.length,31);assert.equal(row.checksums.length,31);for(const ns of row.durations_ns)assert(Number.isSafeInteger(ns)&&ns>0);for(const checksum of row.checksums)assert.equal(checksum,expected(row.kernel,row.size,row.seed)*16);}}
 }
}
function selfCheck(){assert.equal(expected('reduce',0,7),7);assert.equal(expected('reduce',2,7),942);assert.equal(expected('partial-drop',1,7),-1);assert.equal(expected('some',64,7),5);assert.equal(expected('flat-array',17,3),expected('flat-iterator',17,3));
 for(const candidate of candidates){const contract={kind:'contracts',nonCloneOwners:true,partialDrop:true,retryAfterError:true,errorsNotSkipped:true,consumingSomeEvery:true,borrowedFilterFind:true,ownedFlatMapForms:true,finiteOwnerJoin:true,takeZeroPulls:0,surplusDrops:0},repr={kind:'representation',staticBytes:32,enumBytes:40,boxedHandleBytes:16,boxedAllocationNotIncluded:true};
  const rows=[contract,repr,...kernels(candidate).flatMap(kernel=>cases.map(([size,seed])=>({kind:'correctness',candidate,kernel,size,seed,checksum:expected(kernel,size,seed)})))];validate(rows,candidate,false,true);assert.throws(()=>validate(rows.slice(1),candidate,false,true));assert.throws(()=>validate(rows.map((row,index)=>index===2?{...row,checksum:row.checksum+1}:row),candidate,false,true));assert.throws(()=>validate([{...contract,retryAfterError:false},...rows.slice(1)],candidate,false,true));
 }
}
selfCheck();if(process.argv.includes('--self-check')){console.log('Owned iterator admission oracle self-check passed');process.exit(0);}
assert.equal(process.version,'v24.11.1');const target=process.env.CARGO_TARGET_DIR;assert(target&&path.isAbsolute(target));assert.equal(path.resolve(target),'/Users/seancotter/Desktop/dev/learning/tsrust/target');
const compiler=fs.readFileSync(path.join(root,'crates/tsrust-compiler/src/lib.rs'),'utf8').match(/pub const RUST_BACKEND_VERSION: &str = "([^"]+)"/)[1];
const hash=file=>createHash('sha256').update(fs.readFileSync(file)).digest('hex');const files=['reference.rs','profile.rs','profile.mjs','../../probes/counting-allocator.rs','../../quiet-machine.mjs'];const sources=()=>Object.fromEntries(files.map(file=>[file,hash(path.resolve(here,file))]));const before=sources();
const scratch=fs.mkdtempSync(path.join(target,'owned-iterator-admission-'));
const run=(command,args)=>{const result=spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:180000,maxBuffer:16*1024*1024,env:{...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1'}});if(result.error||result.status!==0)throw Error(`${command}: ${result.error??result.stderr}`);return result.stdout;};
const records=(binary,args=[])=>run(binary,args).trim().split('\n').map(line=>JSON.parse(line));
try{
 const builds=[],binaries={};for(const candidate of candidates)for(const allocation of [false,true]){const binary=path.join(scratch,`${candidate}${allocation?'-alloc':''}`),args=['run',compiler,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on'];if(candidate==='finite-enum')args.push('--cfg','candidate_enum');if(candidate==='boxed')args.push('--cfg','candidate_boxed');if(allocation)args.push('--cfg','allocation_probe');else args.push('-F','unsafe_code');args.push(path.join(here,'profile.rs'),'-o',binary);const start=performance.now();run('rustup',args);builds.push({candidate,allocation,buildMs:performance.now()-start,binaryBytes:fs.statSync(binary).size,binarySha256:hash(binary)});binaries[`${candidate}/${allocation}`]=binary;}
 const correctness=[];for(const candidate of candidates){const rows=records(binaries[`${candidate}/false`],['--check']);validate(rows,candidate,false,true);correctness.push({candidate,records:rows});}
 const measurements=[],allocations=[],quietLoads=[];for(let processIndex=0;processIndex<3;processIndex++)for(const candidate of candidates){const counts=records(binaries[`${candidate}/true`]);validate(counts,candidate,true);allocations.push({processIndex,candidate,records:counts});const load=waitForQuietMachine({log:line=>process.stderr.write(`${line}\n`)});assert(Number.isFinite(load)&&load>=0);quietLoads.push({processIndex,candidate,load});const rows=records(binaries[`${candidate}/false`]);validate(rows,candidate);measurements.push({processIndex,candidate,records:rows});}
 assert.deepEqual(sources(),before);console.log(JSON.stringify({scope:'B15 direct Rust owned iterator admission; no public facade implementation or acceptance',sourceCommit:run('git',['rev-parse','HEAD']).trim(),compiler,node:process.version,protocol:{processes:3,warmups:8,samples:31,queriesPerSample:16,separateAllocationBuild:true,sizes,seeds:[7,8]},allocationScope:'Complete query: owned input construction, lazy composition, callback execution, terminal/early drop, retained output observation and output drop. Result Vecs and callback-returned arrays are counted. Finite enum joins carry concrete owned variants; boxed control uses one box for source and each map/filter adaptor. Warmups/builds are excluded; live bytes return to warmed baseline.',compilerScope:'Same prototype source, candidate-specific monomorphization; build time and linked binary bytes are controls, not generated compiler or isolated library footprint claims.',sources:before,builds,correctness,quietLoads,allocations,measurements}));
}finally{fs.rmSync(scratch,{recursive:true,force:true});}
