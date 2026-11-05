// Requires root-approved resolved release artifacts; never invokes Cargo/fetch.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {waitForQuietMachine} from '../../quiet-machine.mjs';
import {selfCheck,validateCorrectness,validateProfile} from './oracle.mjs';
selfCheck();
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../../../..');
const option=name=>process.argv.find(value=>value.startsWith(`--${name}=`))?.slice(name.length+3);
const candidate=option('candidate')??'sync';assert(['base','sync'].includes(candidate));
const artifacts=option('artifacts'),target=process.env.CARGO_TARGET_DIR;
assert(artifacts&&path.isAbsolute(artifacts),'Pass --artifacts=<approved absolute Cargo JSON path>.');
assert(target&&path.isAbsolute(target),'Pass coordinated absolute CARGO_TARGET_DIR.');
assert.equal(process.version,'v24.11.1','Use pinned Node24.11.1.');
const lock=fs.readFileSync(path.join(here,'Cargo.lock'),'utf8');
const baseClosure=JSON.parse(fs.readFileSync(path.join(here,'closure.json'),'utf8'));
const closure=baseClosure.map(row=>({...row,features:candidate==='base'&&row.name==='icu_provider'?row.features.filter(feature=>feature!=='sync'):row.features}));
assert.equal(closure.length,54);
const lockClosure=JSON.parse(fs.readFileSync(path.join(here,'lock-closure.json'),'utf8'));assert.equal(lockClosure.length,57);
assert.equal(lock.split('[[package]]').filter(section=>/^source = /m.test(section)).length,lockClosure.length);
const hash=file=>createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const sourceFiles=['Cargo.toml','Cargo.lock','closure.json','lock-closure.json','production-graphs.json','README.md','reference.rs','profile.rs','footprint.rs','profile.mjs','oracle.mjs','../../probes/counting-allocator.rs','../../quiet-machine.mjs','../../../../crates/tsrust-compiler/src/stdlib/locale/runtime.rs'];
const sourceHashes=()=>Object.fromEntries(sourceFiles.map(file=>[file,hash(path.resolve(here,file))]));
const sources=sourceHashes();
const metadata=fs.readFileSync(artifacts,'utf8').trim().split('\n').filter(line=>line.startsWith('{')).map(line=>JSON.parse(line));
assert(metadata.some(value=>value.reason==='build-finished'&&value.success),'Require successful completed Cargo metadata.');
const rootArtifacts=metadata.filter(row=>row.reason==='compiler-artifact'&&row.target.name==='locale-profile'&&row.target.kind.includes('bin'));assert.equal(rootArtifacts.length,1);assert.equal(rootArtifacts[0].features.includes('thread-safe-locale'),candidate==='sync');
const productionGraphs=JSON.parse(fs.readFileSync(path.join(here,'production-graphs.json'),'utf8'));
assert.equal(productionGraphs.profileMatchesNative,true);
for(const graph of productionGraphs.graphs){assert.equal(hash(path.join(root,graph.manifest)),graph.manifestSha256);assert.equal(hash(path.join(root,path.dirname(graph.manifest),'Cargo.lock')),graph.lockSha256);}
for(const row of lockClosure){const sections=lock.split('[[package]]').filter(section=>section.match(/^\s*name = "([^"]+)"\s*$/m)?.[1]===row.name&&section.match(/^version = "([^"]+)"$/m)?.[1]===row.version);assert.equal(sections.length,1);assert(sections[0].includes(`checksum = "${row.checksum}"`));}
const admittedIds=new Set(closure.map(row=>`${row.source}#${row.name}@${row.version}`));
for(const row of metadata.filter(row=>row.reason==='compiler-artifact'&&row.package_id.startsWith('registry+')))assert(admittedIds.has(row.package_id),`Unexpected built package ${row.package_id}`);
const dependencies={};
for(const admitted of closure){
 const sections=lock.split('[[package]]').filter(section=>section.match(/^\s*name = "([^"]+)"\s*$/m)?.[1]===admitted.name&&section.match(/^version = "([^"]+)"$/m)?.[1]===admitted.version);
 assert.equal(sections.length,1);assert(sections[0].includes(`checksum = "${admitted.checksum}"`));
 const packageId=`${admitted.source}#${admitted.name}@${admitted.version}`;
 const matches=metadata.filter(value=>value.reason==='compiler-artifact'&&value.package_id===packageId);
 assert(matches.some(row=>!row.target.kind.includes('custom-build')),`Missing active package ${packageId}`);
 for(const row of matches){
  assert.deepEqual([...row.features].sort(),[...admitted.features].sort(),`${admitted.name} artifact feature mismatch`);
  for(const file of row.filenames.filter(file=>file.endsWith('.rlib')||file.endsWith('.dylib')||file.endsWith('.so')||file.endsWith('.dll'))){
   assert(path.isAbsolute(file)&&fs.statSync(file).isFile());
   dependencies[`${row.target.name}:${path.basename(file)}`]={package:packageId,lockChecksum:admitted.checksum,path:file,features:row.features,profile:row.profile,sha256:hash(file)};
  }
 }
}
const externs={};for(const name of ['icu_collator','icu_decimal','icu_locale','icu_segmenter','fixed_decimal','bigdecimal']){
 const matches=Object.values(dependencies).filter(row=>row.path.endsWith('.rlib')&&row.package.includes(`#${name}@`));assert.equal(matches.length,1,`Require one ${name} artifact`);externs[name]=matches[0];assert.equal(String(matches[0].profile.opt_level),'3');
}
assert.deepEqual(externs.icu_segmenter.features,['compiled_data']);
const compiler=fs.readFileSync(path.join(root,'crates/tsrust-compiler/src/lib.rs'),'utf8').match(/pub const RUST_BACKEND_VERSION: &str = "([^"]+)"/)[1];
const scratch=fs.mkdtempSync(path.join(target,'locale-profile-'));
const run=(command,args)=>{const result=spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024,env:{...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1'}});if(result.error||result.status!==0)throw new Error(`${command}: ${result.error??result.stderr}`);return result.stdout;};
const records=(binary,args=[])=>run(binary,args).trim().split('\n').map(line=>JSON.parse(line));
try{
 const builds=[],binaries={};
 for(const instrumented of [false,true]){
  const binary=path.join(scratch,`locale${instrumented?'-alloc':''}${process.platform==='win32'?'.exe':''}`);
  const args=['run',compiler,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on',...Object.entries(externs).flatMap(([name,row])=>['--extern',`${name}=${row.path}`])];
  for(const directory of new Set(Object.values(dependencies).map(value=>path.dirname(value.path))))args.push('-L',`dependency=${directory}`);
  if(instrumented)args.push('--cfg','feature="allocation-probe"');
  if(candidate==='sync')args.push('--cfg','feature="thread-safe-locale"');
  args.push(path.join(here,'profile.rs'),'-o',binary);const started=performance.now();run('rustup',args);
  builds.push({instrumented,buildMs:performance.now()-started,binaryBytes:fs.statSync(binary).size});binaries[instrumented]=binary;
 }
 const footprint=[];
 for(const mode of ['baseline','locale','collator','decimal','grapheme','all']){
  const binary=path.join(scratch,`locale-footprint-${mode}${process.platform==='win32'?'.exe':''}`);
  const args=['run',compiler,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on',...Object.entries(externs).flatMap(([name,row])=>['--extern',`${name}=${row.path}`])];
  for(const directory of new Set(Object.values(dependencies).map(row=>path.dirname(row.path))))args.push('-L',`dependency=${directory}`);
  if(mode!=='baseline')args.push('--cfg',`footprint_${mode}`);
  if(candidate==='sync')args.push('--cfg','feature="thread-safe-locale"');
  args.push(path.join(here,'footprint.rs'),'-o',binary);const started=performance.now();run('rustup',args);
  const buildMs=performance.now()-started;const observed=run(binary,[]).trim();assert(/^\d+$/.test(observed));
  const sizeOutput=run('size',[binary]);
  footprint.push({mode,buildMs,binaryBytes:fs.statSync(binary).size,sectionSizes:sizeOutput,checksum:Number(observed)});
 }
 const expectedFootprintChecksums={baseline:8,locale:13,collator:8,decimal:17,grapheme:90,all:104};
 const baseline=footprint[0].binaryBytes;for(const row of footprint){assert.equal(row.checksum,expectedFootprintChecksums[row.mode]);row.deltaBytes=row.binaryBytes-baseline;}
 const correctness=records(binaries.false,['--check']);const correctnessClassification=validateCorrectness(correctness);
 const allocations=[],measurements=[],quietLoads=[];
 for(let processIndex=0;processIndex<3;processIndex++){
  const counts=records(binaries.true);validateProfile(counts,true);allocations.push({processIndex,records:counts});
  quietLoads.push(waitForQuietMachine({log:line=>process.stderr.write(`${line}\n`)}));
  const timing=records(binaries.false);validateProfile(timing,false);measurements.push({processIndex,records:timing});
 }
 assert.deepEqual(sourceHashes(),sources,'Profile source changed during build or execution');
 for(const dependency of Object.values(dependencies))assert.equal(hash(dependency.path),dependency.sha256,'Dependency artifact changed during execution');
 console.log(JSON.stringify({scope:'direct Rust locale admission prototype; not generated facade performance',candidate,publicOwnerSendSyncWitness:candidate==='sync',productionGraphMatch:candidate==='sync'?'exact selected native build identities and features':'historical Rc control: selected provider/sync omitted',compiler,platform:process.platform,arch:process.arch,node:process.version,cpuCount:os.cpus().length,protocol:{processes:3,warmups:8,samples:31,separateAllocationBuild:true,operation:'one complete query; input size records corpus repetitions or scaling bytes'},nodeIcu:process.versions.icu,nodeUnicode:process.versions.unicode,nodeCldr:process.versions.cldr,correctnessClassification,productionGraphs,closure,lockClosure,dependencies,sources,builds,footprint,correctness,quietLoads,allocations,measurements}));
}finally{fs.rmSync(scratch,{recursive:true,force:true});}
