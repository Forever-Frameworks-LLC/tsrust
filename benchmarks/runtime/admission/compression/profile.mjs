// Admission only; Cargo/rustc/native execution require the exclusive parent lease.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {waitForQuietMachine} from '../../quiet-machine.mjs';
import {dependencyArtifacts} from '../../native-dependencies.mjs';
import {cases,input,checksum,decode,encode,corpusName} from './node-control.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../../../..'),target=process.env.CARGO_TARGET_DIR;
const backends=['miniz','zlib-rs'],groups=['gzip.compress','gzip.decompress','zlib.compress','zlib.decompress','raw.compress','raw.decompress'];
const key=row=>[row.format,row.strategy,row.operation,row.pattern,row.level,row.chunk].join('/');
const groupCases=group=>cases().filter(row=>`${row.format}.${row.operation}`===group);
const digest=bytes=>createHash('sha256').update(bytes).digest('hex');
function domain(row){
 for(const [name,allowed]of Object.entries({format:['gzip','zlib','raw'],strategy:['writer','reader'],operation:['compress','decompress'],pattern:['compressible','random'],level:[0,1,-1,9],chunk:[64,8192,16384]}))assert(allowed.includes(row[name]),`Invalid ${name}`);
}
function original(row){return input(16384,row.pattern==='random');}
function fixtures(rows,backend){
 const expected=new Set(cases().filter(row=>row.operation==='compress').map(key)),joined=new Map();assert.equal(rows.length,expected.size);
 for(const row of rows){
  domain(row);assert.equal(row.operation,'compress');assert(expected.delete(key(row)),'Duplicate or unexpected fixture');assert.equal(row.backend,backend);assert.equal(row.kind,'fixture');
  const source=original(row);assert.equal(row.inputBytes,source.length);assert.equal(row.inputChecksum,checksum(source));
  assert.equal(typeof row.hex,'string');assert(/^(?:[0-9a-f]{2})+$/.test(row.hex));const bytes=Buffer.from(row.hex,'hex');
  assert.equal(row.outputBytes,bytes.length);assert.equal(row.checksum,checksum(bytes));assert.deepEqual(decode(row.format,bytes),source);
  joined.set(key(row),{outputBytes:bytes.length,checksum:row.checksum,sha256:digest(bytes)});
 }assert.equal(expected.size,0);return joined;
}
function validate(rows,backend,allocation,expectedCases,joined,corpus){
 assert.equal(rows.length,expectedCases.length);const remaining=new Set(expectedCases.map(key));
 for(const row of rows){
  domain(row);assert(remaining.delete(key(row)),'Duplicate or unexpected sample case');assert.equal(row.backend,backend);assert.equal(row.kind,allocation?'allocation':'timing');
  const source=row.operation==='compress'?original(row):corpus.get(corpusName(row));assert(source);assert.equal(row.inputBytes,source.length);assert.equal(row.inputChecksum,checksum(source));
  const expected=row.operation==='compress'?joined.get(key(row)):{outputBytes:16384,checksum:checksum(original(row))};assert(expected);
  assert.equal(row.outputBytes,expected.outputBytes);assert.equal(row.checksum,expected.checksum);
  if(allocation){assert.equal(row.retained,0);for(const name of ['allocations','reallocations','requested','peak'])assert(Number.isSafeInteger(row[name])&&row[name]>=0);}
  else{
   for(const name of ['durations_ns','checksums','outputLengths']){assert(Array.isArray(row[name]));assert.equal(row[name].length,31);}
   for(let sample=0;sample<31;sample++){assert(Number.isSafeInteger(row.durations_ns[sample])&&row.durations_ns[sample]>0);assert.equal(row.checksums[sample],expected.checksum);assert.equal(row.outputLengths[sample],expected.outputBytes);}
  }
 }assert.equal(remaining.size,0);
}
function contracts(rows){
 const required={contracts:['roundTrips','incrementalLimit','truncation','checksums','strictTrailingData','gzipMembers','deterministicGzip','finishFailure','backendDefaultLevel'],'writer-contracts':['explicitStreamEnd','incrementalLimit','truncation','strictTrailingData','sinkFailure'],'short-io-contracts':['shortWrites','interruptedWrite','finishTrailerFailure']};
 assert.equal(rows.length,3);const seen=new Set();for(const row of rows){assert(required[row.kind]);assert(!seen.has(row.kind));seen.add(row.kind);assert.deepEqual(Object.keys(row).sort(),['kind',...required[row.kind]].sort());for(const name of required[row.kind])assert.equal(row[name],name==='backendDefaultLevel'?6:true);}
}
function selfCheck(){
 assert.equal(cases().length,288);assert.equal(new Set(cases().map(key)).size,288);for(const group of groups)assert.equal(groupCases(group).length,48);
 assert.equal(checksum(Buffer.alloc(0)),7);assert.equal(checksum(Buffer.from([0,255])),120382);assert.equal(checksum(input(1,false)),1033);assert.notDeepEqual(input(64,false),input(64,true));
 const rows=cases().filter(row=>row.operation==='compress').map(row=>{const source=original(row),output=encode(row.format,source,row.level);return {kind:'fixture',backend:'miniz',...row,inputBytes:source.length,inputChecksum:checksum(source),outputBytes:output.length,checksum:checksum(output),hex:output.toString('hex')};});
 const joined=fixtures(rows,'miniz');assert.throws(()=>fixtures(rows.slice(1),'miniz'));assert.throws(()=>fixtures([rows[0],rows[0],...rows.slice(2)],'miniz'));
 for(const update of [{pattern:'other'},{level:'0'},{outputBytes:rows[0].outputBytes+1},{checksum:rows[0].checksum+1}])assert.throws(()=>fixtures([{...rows[0],...update},...rows.slice(1)],'miniz'));
 const corpus=new Map();for(const row of cases())if(row.operation==='decompress'&&!corpus.has(corpusName(row)))corpus.set(corpusName(row),encode(row.format,original(row),row.level));
 const samples=cases().map(row=>{const source=row.operation==='compress'?original(row):corpus.get(corpusName(row)),expected=row.operation==='compress'?joined.get(key(row)):{outputBytes:16384,checksum:checksum(original(row))};return {kind:'timing',backend:'miniz',...row,inputBytes:source.length,inputChecksum:checksum(source),...expected,durations_ns:Array(31).fill(1),checksums:Array(31).fill(expected.checksum),outputLengths:Array(31).fill(expected.outputBytes)};});
 validate(samples,'miniz',false,cases(),joined,corpus);assert.throws(()=>validate(samples.slice(1),'miniz',false,cases(),joined,corpus));assert.throws(()=>validate([samples[0],samples[0],...samples.slice(2)],'miniz',false,cases(),joined,corpus));
 for(const update of [{backend:'other'},{pattern:'other'},{checksum:samples[0].checksum+1},{outputBytes:samples[0].outputBytes+1},{inputChecksum:samples[0].inputChecksum+1},{durations_ns:Array(31).fill(0)},{durations_ns:Array(30).fill(1)},{checksums:[samples[0].checksum+1,...Array(30).fill(samples[0].checksum)]},{outputLengths:Array(31).fill(samples[0].outputBytes+1)}])assert.throws(()=>validate([{...samples[0],...update},...samples.slice(1)],'miniz',false,cases(),joined,corpus));
 const allocations=samples.map(row=>({...row,kind:'allocation',retained:0,allocations:1,reallocations:0,requested:1,peak:1}));validate(allocations,'miniz',true,cases(),joined,corpus);assert.throws(()=>validate([{...allocations[0],retained:1},...allocations.slice(1)],'miniz',true,cases(),joined,corpus));
}
selfCheck();if(process.argv.includes('--self-check')){console.log('Compression admission independent fixture/sample/matrix self-check passed');process.exit(0);}
assert.equal(process.version,'v24.11.1');assert.equal(target,'/Users/seancotter/Desktop/dev/learning/tsrust/target');assert.equal(process.env.CARGO_INCREMENTAL,'0');assert.equal(process.env.CARGO_BUILD_JOBS,'1');
const selectedGroups=process.env.TSRUST_COMPRESSION_GROUPS?.split(',')??groups;assert(selectedGroups.length>0);assert.equal(new Set(selectedGroups).size,selectedGroups.length);for(const group of selectedGroups)assert(groups.includes(group));
const hash=file=>digest(fs.readFileSync(file));
const files=['Cargo.toml','Cargo.lock','reference.rs','profile.rs','profile.mjs','node-control.mjs','admission.json','pool-contract.rs','../../native-dependencies.mjs','../../../../crates/tsrust-compiler/src/stdlib/futures/runtime.rs','../../../../crates/tsrust-compiler/src/stdlib/threads/runtime_core.rs','../../../../crates/tsrust-compiler/src/stdlib/threads/runtime_async.rs','../../probes/counting-allocator.rs','../../quiet-machine.mjs'];
const sources=()=>Object.fromEntries(files.map(file=>[file,hash(path.resolve(here,file))]));const before=sources();const scratch=fs.mkdtempSync(path.join(target,'compression-admission-')),corpusDirectory=path.join(scratch,'corpus');fs.mkdirSync(corpusDirectory);
process.stderr.write(`COMPRESSION_SCOPE_DIR ${scratch}\n`);
const corpus=new Map(),corpusMetadata=[];
for(const row of cases())if(row.operation==='decompress'&&!corpus.has(corpusName(row))){const source=original(row),bytes=encode(row.format,source,row.level),file=path.join(corpusDirectory,corpusName(row));assert.deepEqual(decode(row.format,bytes),source);fs.writeFileSync(file,bytes,{flag:'wx',mode:0o444});corpus.set(corpusName(row),bytes);corpusMetadata.push({name:corpusName(row),format:row.format,pattern:row.pattern,level:row.level,inputBytes:bytes.length,inputChecksum:checksum(bytes),sha256:hash(file)});}
const save=(name,value)=>fs.writeFileSync(path.join(scratch,name),JSON.stringify({sources:before,corpus:corpusMetadata,...value})+'\n',{flag:'wx'});
save('scope.json',{status:'preparing',selectedGroups,protocol:{processes:3,warmups:8,samples:31,caseCount:selectedGroups.length*48}});
let failedCommands=0,lastCommand=null;
const run=(command,args,options={})=>{const result=spawnSync(command,args,{cwd:here,encoding:'utf8',timeout:300000,maxBuffer:32*1024*1024,env:{...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1',TSRUST_COMPRESSION_CORPUS:corpusDirectory},...options});lastCommand={command,args,status:result.status,stdout:result.stdout??'',stderr:result.stderr??''};if(result.error||result.status!==0){const prefix=`failed-command-${failedCommands++}`;fs.writeFileSync(path.join(scratch,prefix+'.stdout'),result.stdout??'');fs.writeFileSync(path.join(scratch,prefix+'.stderr'),result.stderr??'');save(prefix+'.json',{command,args,status:result.status,error:String(result.error??'')});throw Error(`${command}: ${result.error??result.stderr}`);}return result;};
const parse=text=>text.trim().split('\n').filter(Boolean).map(line=>JSON.parse(line));
const python=String.raw`import json,resource,subprocess,sys
p=subprocess.run(sys.argv[1:],stdout=subprocess.PIPE,stderr=subprocess.PIPE,text=True)
r=resource.getrusage(resource.RUSAGE_CHILDREN)
sys.stdout.write(p.stdout)
sys.stderr.write(p.stderr)
sys.stderr.write('\nTSRUST_RESOURCE '+json.dumps({'userSeconds':r.ru_utime,'systemSeconds':r.ru_stime,'peakResidentBytes':r.ru_maxrss if sys.platform=='darwin' else r.ru_maxrss*1024})+'\n')
sys.exit(p.returncode)
`;
function measured(command,args=[]){const result=run('python3',['-c',python,command,...args]);const line=result.stderr.trim().split('\n').find(line=>line.startsWith('TSRUST_RESOURCE '));assert(line);const resources=JSON.parse(line.slice(16));for(const name of ['userSeconds','systemSeconds','peakResidentBytes'])assert(Number.isFinite(resources[name])&&resources[name]>=0);return {records:parse(result.stdout),resources};}
let success=false;
try{
 const builds=[],binaries={},dependencies={};
 for(const backend of backends)for(const allocation of [false,true]){
  const features=[backend,...(allocation?['allocation-probe']:[])].join(',');const start=performance.now();
  const result=run('rustup',['run','1.96.1','cargo','build','--manifest-path',path.join(here,'Cargo.toml'),'--release','--offline','--locked','--no-default-features','--features',features,'--message-format=json']);save(`${backend}${allocation?"-alloc":""}-cargo-artifacts.json`,{stdout:result.stdout,stderr:result.stderr});const rows=parse(result.stdout);assert.equal(rows.at(-1)?.reason,'build-finished');assert.equal(rows.at(-1)?.success,true);
  const executable=rows.findLast(row=>row.reason==='compiler-artifact'&&row.target.name==='compression-profile'&&row.executable);assert(executable);assert.equal(String(executable.profile.opt_level),'3');const flate=rows.filter(row=>row.reason==='compiler-artifact'&&row.target.name==='flate2');assert.equal(flate.length,1);assert.equal(flate[0].features.includes('zlib-rs'),backend==='zlib-rs');assert.equal(flate[0].features.includes('rust_backend'),backend==='miniz');assert(!flate[0].features.includes('any_c_zlib'));
  const binary=path.join(scratch,`${backend}${allocation?'-alloc':''}`);fs.copyFileSync(executable.executable,binary);binaries[`${backend}/${allocation}`]=binary;
  const closure=rows.filter(row=>row.reason==='compiler-artifact'&&row.target.kind.includes('lib')).map(row=>({name:row.target.name,package:row.package_id,features:row.features,files:row.filenames.filter(file=>file.endsWith('.rlib')).map(file=>({path:file,sha256:hash(file)}))}));dependencies[`${backend}/${allocation}`]=closure;
  builds.push({backend,allocation,buildMs:performance.now()-start,binaryBytes:fs.statSync(binary).size,binarySha256:hash(binary),features});
 }
 const correctness=[],cross=[],joinedFixtures=new Map();
 for(const backend of backends){const binary=binaries[`${backend}/false`],checked=parse(run(binary,['--check']).stdout);contracts(checked);correctness.push({backend,contracts:checked});save(`${backend}-contracts.json`,{backend,records:checked});
  const fixtureRows=parse(run(binary,['--fixtures']).stdout);const fixtureJoin=fixtures(fixtureRows,backend);joinedFixtures.set(backend,fixtureJoin);const observed=fixtureRows.map(({hex,...row})=>({...row,sha256:fixtureJoin.get(key(row)).sha256}));cross.push({backend,fixtures:observed});save(`${backend}-fixtures.json`,{backend,records:observed});
  const corruption=[];
  for(const format of ['gzip','zlib','raw'])for(const random of [false,true]){const expected=input(16384,random),compressed=encode(format,expected,-1),result=JSON.parse(run(binary,['--decode',format,compressed.toString('hex')]).stdout);assert.equal(result.ok,true);assert.deepEqual(Buffer.from(result.hex,'hex'),expected);const bounded=JSON.parse(run(binary,['--decode',format,compressed.toString('hex'),'16383']).stdout);assert.equal(bounded.ok,false);
   for(const removed of [1,2]){const broken=compressed.subarray(0,compressed.length-removed);assert.throws(()=>decode(format,broken));assert.equal(JSON.parse(run(binary,['--decode',format,broken.toString('hex')]).stdout).ok,false);corruption.push({format,random,kind:'truncated',removed,rustRejected:true,nodeRejected:true});}
   if(format!=='raw'){const broken=Buffer.from(compressed);broken[broken.length-1]^=1;assert.throws(()=>decode(format,broken));assert.equal(JSON.parse(run(binary,['--decode',format,broken.toString('hex')]).stdout).ok,false);corruption.push({format,random,kind:'checksum-or-size',rustRejected:true,nodeRejected:true});}
   const trailing=Buffer.concat([compressed,Buffer.from('unexpected tail')]);assert.equal(JSON.parse(run(binary,['--decode',format,trailing.toString('hex')]).stdout).ok,false);let nodeRejected=false;try{decode(format,trailing);}catch{nodeRejected=true;}corruption.push({format,random,kind:'trailing-data',rustRejected:true,nodeRejected});
  }
  const first=Buffer.from('one'),second=Buffer.from('two'),members=Buffer.concat([encode('gzip',first,-1),encode('gzip',second,-1)]);assert.deepEqual(decode('gzip',members),Buffer.concat([first,second]));const joined=JSON.parse(run(binary,['--decode','gzip',members.toString('hex')]).stdout);assert.equal(joined.ok,true);assert.deepEqual(Buffer.from(joined.hex,'hex'),Buffer.concat([first,second]));cross.at(-1).corruption=corruption;cross.at(-1).nodeGzipMembers=true;save(`${backend}-cross-corruption.json`,{backend,corruption,nodeGzipMembers:true});
 }
 const nodeFixtures=parse(run(process.execPath,[path.join(here,'node-control.mjs'),'--fixtures']).stdout),nodeJoined=fixtures(nodeFixtures,'node');joinedFixtures.set('node',nodeJoined);const nodeObserved=nodeFixtures.map(({hex,...row})=>({...row,sha256:nodeJoined.get(key(row)).sha256}));cross.push({backend:'node',fixtures:nodeObserved});save('node-fixtures.json',{backend:'node',records:nodeObserved});
 const pool=[],poolInputs=[];
 const poolArtifacts=process.env.TSRUST_COMPRESSION_POOL_ARTIFACTS;
 if(poolArtifacts){
  const artifactText=fs.readFileSync(poolArtifacts,'utf8'),artifactRows=parse(artifactText);poolInputs.push({path:poolArtifacts,sha256:hash(poolArtifacts)});assert.equal(artifactRows.at(-1)?.reason,'build-finished');assert.equal(artifactRows.at(-1)?.success,true);
  const libraries=dependencyArtifacts(artifactText,target,['tokio','rayon']);
  for(const name of ['tokio','rayon']){const row=artifactRows.findLast(row=>row.reason==='compiler-artifact'&&row.target.name===name);assert.equal(String(row.profile.opt_level),'3');if(name==='tokio')for(const feature of ['rt','sync','time'])assert(row.features.includes(feature));}
  const dependencyBefore=Object.fromEntries(Object.entries(libraries).map(([name,file])=>{poolInputs.push({path:file,sha256:hash(file)});return [name,hash(file)];}));
  for(const backend of backends){const flate=dependencies[`${backend}/false`].find(row=>row.name==='flate2');assert.equal(flate.files.length,1);const binary=path.join(scratch,`${backend}-pool-contract`);
   run('rustup',['run','1.96.1','rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-L',`dependency=${path.join(target,'release','deps')}`,'--extern',`flate2=${flate.files[0].path}`,...Object.entries(libraries).flatMap(([name,file])=>['--extern',`${name}=${file}`]),path.join(here,'pool-contract.rs'),'-o',binary]);
   const records=parse(run(binary,[]).stdout);assert.equal(records.length,1);assert.equal(records[0].kind,'pool-contracts');for(const [name,value]of Object.entries(records[0]))if(name!=='kind')assert.equal(value,true);pool.push({backend,records,binarySha256:hash(binary),dependencies:dependencyBefore});save(`${backend}-pool-contract.json`,{backend,records,dependencies:dependencyBefore});
  }
  for(const [name,file]of Object.entries(libraries))assert.equal(hash(file),dependencyBefore[name]);
 }
 const measurements=[],allocations=[],quietLoads=[],completedGroups=[];
 for(const group of selectedGroups){
  const groupMeasurements=[],groupAllocations=[],groupQuiet=[];
  for(let processIndex=0;processIndex<3;processIndex++)for(const backend of [...backends,'node']){
   if(backend!=='node'){
    const result=measured(binaries[`${backend}/true`],['--group',group]);validate(result.records,backend,true,groupCases(group),joinedFixtures.get(backend),corpus);
    const scope={group,processIndex,backend,...result};allocations.push(scope);groupAllocations.push(scope);save(`${group}-${backend}-${processIndex}-allocation.json`,scope);
   }
   const load=waitForQuietMachine({log:line=>process.stderr.write(`${line}\n`)});assert(Number.isFinite(load)&&load>=0);const quiet={group,processIndex,backend,load};quietLoads.push(quiet);groupQuiet.push(quiet);
   const result=backend==='node'?measured(process.execPath,[path.join(here,'node-control.mjs'),'--measure','--group',group]):measured(binaries[`${backend}/false`],['--group',group]);
   validate(result.records,backend,false,groupCases(group),joinedFixtures.get(backend),corpus);const scope={group,processIndex,backend,...result};measurements.push(scope);groupMeasurements.push(scope);save(`${group}-${backend}-${processIndex}-timing.json`,scope);
  }
  assert.deepEqual(sources(),before);for(const row of corpusMetadata)assert.equal(hash(path.join(corpusDirectory,row.name)),row.sha256);
  save(`${group}-complete.json`,{group,protocol:{processes:3,warmups:8,samples:31,caseCount:48},measurements:groupMeasurements,allocations:groupAllocations,quietLoads:groupQuiet});completedGroups.push(group);process.stderr.write(`COMPRESSION_GROUP_COMPLETE ${group}\n`);
 }
 assert.deepEqual(sources(),before);for(const row of corpusMetadata)assert.equal(hash(path.join(corpusDirectory,row.name)),row.sha256);
 for(const rows of Object.values(dependencies))for(const row of rows)for(const file of row.files)assert.equal(hash(file.path),file.sha256);for(const file of poolInputs)assert.equal(hash(file.path),file.sha256);
 console.log(JSON.stringify({scope:'B09 direct Rust backend/transform admission; no public or generated implementation acceptance',sourceCommit:run('git',['rev-parse','HEAD']).stdout.trim(),node:process.version,nodeZlib:process.versions.zlib,compiler:'1.96.1',protocol:{processes:3,warmups:8,samples:31,queriesPerSample:1,caseCount:selectedGroups.length*48,fullCaseCount:288,selectedGroups,completedGroups,separateAllocationBuild:true,quiet:true},allocationScope:'Rust codec state, transformed output observation and disposal; immutable input/common Node-encoded fixture construction and warmups excluded. Every decode engine reads identical compressed bytes, verified before and after timing. Output stays live during checksum observation and is dropped before retained measurement. Scratch buffers are stack allocated. No ordinary expansion cap. Node allocator counts unavailable. Node stream timings await codec close after readable end; Rust timings include codec/output drop.',resourceScope:'Per-group child-process CPU user/system seconds and peak RSS include fixture setup/reads, correctness assertions, warmups, all samples and JSON reporting; not isolated per operation. Build elapsed/size and watchdog peak are separate.',failureScope:'Failed command stdout/stderr, immutable corpus, validated process scopes and complete 3x31 groups remain in the announced COMPRESSION_SCOPE_DIR on failure. No source snapshots. Successful scratch is removed after publishing this complete result.',sources:before,corpus:corpusMetadata,builds,dependencies,poolInputs,correctness,cross,pool,poolStatus:poolArtifacts?'checked':'pending explicit existing Tokio/Rayon artifact manifest',quietLoads,allocations,measurements}));success=true;
}catch(error){if(lastCommand){fs.writeFileSync(path.join(scratch,'last-command.stdout'),lastCommand.stdout);fs.writeFileSync(path.join(scratch,'last-command.stderr'),lastCommand.stderr);save('last-command.json',{command:lastCommand.command,args:lastCommand.args,status:lastCommand.status});}save('failure.json',{status:'failed',error:String(error),stack:error.stack,scopeDirectory:scratch});process.stderr.write(`COMPRESSION_FAILURE_PRESERVED ${scratch}\n`);throw error;
}finally{if(success)fs.rmSync(scratch,{recursive:true,force:true});}
