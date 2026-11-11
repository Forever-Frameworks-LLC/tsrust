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
const artifacts=option('artifacts'),target=process.env.CARGO_TARGET_DIR;
assert(artifacts&&path.isAbsolute(artifacts),'Pass --artifacts=<approved absolute Cargo JSON path>.');
assert(target&&path.isAbsolute(target),'Pass coordinated absolute CARGO_TARGET_DIR.');
assert.equal(process.version,'v24.11.1','Use pinned Node24.11.1.');
const lock=fs.readFileSync(path.join(here,'Cargo.lock'),'utf8');
const regexLock=lock.split('[[package]]').find(section=>/^\s*name = "regex"\s*$/m.test(section));
assert(regexLock&&/^version = "1.13.1"$/m.test(regexLock)&&/^checksum = "f020237b6c8eed93db2e2cb53c00c60a8e1bc73da7d073199a1180401450218d"$/m.test(regexLock),'Resolve and verify exact selected package first.');
const hash=file=>createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const sourceFiles=['Cargo.toml','Cargo.lock','reference.rs','profile.rs','profile.mjs','oracle.mjs','../../probes/counting-allocator.rs','../../quiet-machine.mjs'];
const sourceHashes=()=>Object.fromEntries(sourceFiles.map(file=>[file,hash(path.resolve(here,file))]));
const sources=sourceHashes();
const metadata=fs.readFileSync(artifacts,'utf8').trim().split('\n').filter(line=>line.startsWith('{')).map(line=>JSON.parse(line));
assert(metadata.some(value=>value.reason==='build-finished'&&value.success),'Require successful completed Cargo metadata.');
const dependencies={};
for(const name of ['regex','regex_automata','regex_syntax','aho_corasick','memchr']){
 const matches=metadata.filter(value=>value.reason==='compiler-artifact'&&value.target.name===name&&value.filenames.some(file=>file.endsWith('.rlib')));
 assert.equal(matches.length,1,`Ambiguous ${name} artifact`);const row=matches[0],files=row.filenames.filter(file=>file.endsWith('.rlib'));assert.equal(files.length,1);
 assert.equal(String(row.profile.opt_level),'3',`${name} must use release optimization`);
 const packageName=name.replaceAll('_','-'),sections=lock.split('[[package]]').filter(section=>section.match(/^\s*name = "([^"]+)"\s*$/m)?.[1]===packageName);
 assert.equal(sections.length,1,`Ambiguous locked ${packageName}`);const version=sections[0].match(/^version = "([^"]+)"$/m)?.[1];
 const source=sections[0].match(/^source = "([^"]+)"$/m)?.[1],checksum=sections[0].match(/^checksum = "([a-f0-9]{64})"$/m)?.[1];
 assert.equal(source,"registry+https://github.com/rust-lang/crates.io-index",`${packageName} must use admitted registry source`);
 assert(checksum&&version&&row.package_id===`${source}#${packageName}@${version}`,`${packageName} artifact differs from approved lock`);
 assert(path.isAbsolute(files[0])&&fs.statSync(files[0]).isFile());
 dependencies[name]={package:row.package_id,lockChecksum:checksum,path:files[0],features:row.features,profile:row.profile,sha256:createHash('sha256').update(fs.readFileSync(files[0])).digest('hex')};
}
assert(dependencies.regex.package.endsWith('@1.13.1'));
for(const feature of ['std','perf','unicode'])assert(dependencies.regex.features.includes(feature));
const compiler=fs.readFileSync(path.join(root,'crates/tsrust-compiler/src/lib.rs'),'utf8').match(/pub const RUST_BACKEND_VERSION: &str = "([^"]+)"/)[1];
const scratch=fs.mkdtempSync(path.join(target,'regex-text-profile-'));
const run=(command,args)=>{const result=spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024,env:{...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1'}});if(result.error||result.status!==0)throw new Error(`${command}: ${result.error??result.stderr}`);return result.stdout;};
const records=(binary,args=[])=>run(binary,args).trim().split('\n').map(line=>JSON.parse(line));
try{
 const builds=[],binaries={};
 for(const instrumented of [false,true]){
  const binary=path.join(scratch,`regex-text${instrumented?'-alloc':''}${process.platform==='win32'?'.exe':''}`);
  const args=['run',compiler,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on','--extern',`regex=${dependencies.regex.path}`];
  for(const directory of new Set(Object.values(dependencies).map(value=>path.dirname(value.path))))args.push('-L',`dependency=${directory}`);
  if(instrumented)args.push('--cfg','feature="allocation-probe"');
  args.push(path.join(here,'profile.rs'),'-o',binary);const started=performance.now();run('rustup',args);
  builds.push({instrumented,buildMs:performance.now()-started,binaryBytes:fs.statSync(binary).size});binaries[instrumented]=binary;
 }
 const correctness=records(binaries.false,['--check']);validateCorrectness(correctness);
 const allocations=[],measurements=[],quietLoads=[];
 for(let processIndex=0;processIndex<3;processIndex++){
  const counts=records(binaries.true);validateProfile(counts,true);allocations.push({processIndex,records:counts});
  quietLoads.push(waitForQuietMachine({log:line=>process.stderr.write(`${line}\n`)}));
  const timing=records(binaries.false);validateProfile(timing,false);measurements.push({processIndex,records:timing});
 }
 assert.deepEqual(sourceHashes(),sources,'Profile source changed during build or execution');
 for(const dependency of Object.values(dependencies))assert.equal(hash(dependency.path),dependency.sha256,'Dependency artifact changed during execution');
 console.log(JSON.stringify({scope:'direct Rust regex admission prototype; not generated facade performance',compiler,platform:process.platform,arch:process.arch,node:process.version,cpuCount:os.cpus().length,protocol:{processes:3,warmups:8,samples:31,separateAllocationBuild:true,operation:'one complete query; input size records corpus repetitions or scaling bytes'},dependencies,sources,builds,correctness,quietLoads,allocations,measurements}));
}finally{fs.rmSync(scratch,{recursive:true,force:true});}
