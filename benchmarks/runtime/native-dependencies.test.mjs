import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import childProcess from 'node:child_process';
import {syncBuiltinESMExports} from 'node:module';
import {fileURLToPath} from 'node:url';
import {prepareNativeDependencies, prepareReferenceDependencies, dependencyArtifacts, needsJsonDependencies} from './native-dependencies.mjs';

test('ordinary native workloads need no dependency preparation',()=>{
 assert.deepEqual(prepareNativeDependencies({emitted:'fn main() {}'}),{args:[],evidence:null});
});

test('dependency artifacts require both exact Cargo library identities',()=>{
 const target='/target';
 const artifact=(name,file)=>JSON.stringify({reason:'compiler-artifact',target:{name,kind:['lib']},filenames:[file]});
 const serde=artifact('serde','/target/release/deps/libserde-a.rlib');
 const json=artifact('serde_json','/target/release/deps/libserde_json-b.rlib');
 assert.deepEqual(dependencyArtifacts(`${serde}\n${json}`,target),{serde:'/target/release/deps/libserde-a.rlib',serde_json:'/target/release/deps/libserde_json-b.rlib'});
 assert.throws(()=>dependencyArtifacts(serde,target),/Missing.*serde_json/);
 assert.throws(()=>dependencyArtifacts(`${serde}\n${artifact('serde_json','/elsewhere/libserde_json-b.rlib')}`,target),/outside/);
 assert.throws(()=>dependencyArtifacts(`${serde}\n${json}\n${artifact('serde','/target/release/deps/libserde-c.rlib')}`,target),/Ambiguous/);
});

test('exported dependencies link instrumented JSON, files and hostname programs offline',{skip:!process.env.TSRUST},()=>{
 const root=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-bench-dependencies-'));
 try {
  const source=path.join(root,'input.tsr');
  fs.writeFileSync(source,`import {parse,kind} from 'std:json'; function main(){match(parse("null")){Ok(value)=>{console.log(kind(value));},Err(error)=>{console.log(error.code);}}}`);
  const run=(command,args,options={})=>{
   const result=spawnSync(command,args,{encoding:'utf8',timeout:120000,...options});
   assert.equal(result.status,0,result.stderr||String(result.error));return result.stdout;
  };
  const emitted=run(process.env.TSRUST,['emit',source]);
  const options={compiler:process.env.TSRUST,source,emitted,build:root,rustVersion:'1.96.1',env:process.env};
  const dependencies=prepareNativeDependencies(options);
  assert.equal(dependencies.evidence.libraries.length,2);
  assert.ok(dependencies.evidence.lockSha256);
  const repeated=prepareNativeDependencies(options);
  assert.deepEqual(repeated.evidence.libraries,dependencies.evidence.libraries);
  const occupied=path.join(root,'package-native-dependencies');fs.mkdirSync(occupied);
  fs.writeFileSync(path.join(occupied,'owner'),'keep');
  assert.throws(()=>prepareNativeDependencies(options),/EEXIST/);
  assert.equal(fs.readFileSync(path.join(occupied,'owner'),'utf8'),'keep');
  fs.rmSync(occupied,{recursive:true});
  const instrumented=path.join(root,'probe.rs'),binary=path.join(root,'probe');
  fs.writeFileSync(instrumented,emitted.replace('fn main()','fn source_main()')+'\nfn main(){source_main();println!("instrumented");}\n');
  run('rustup',['run','1.96.1','rustc','--edition=2024','-A','warnings','-C','overflow-checks=on',...dependencies.args,instrumented,'-o',binary]);
  assert.equal(run(binary,[]),'null\ninstrumented\n');
  assert.throws(()=>prepareNativeDependencies({...options,emitted:emitted+'\n// changed'}),/differs/);
  fs.writeFileSync(source,`import {readFile} from 'std:fs';function main(){match(readFile('missing-dependency-test-file')){Ok(bytes)=>{throw 'unexpected file';},Err(error)=>{console.log(error.code);}}}`);
  const fileEmitted=run(process.env.TSRUST,['emit',source]);assert.ok(!fileEmitted.includes('unsafe'));
  const fileDependencies=prepareNativeDependencies({...options,emitted:fileEmitted});
  assert.deepEqual(fileDependencies.evidence.libraries.map(library=>library.name),['nix']);
  fs.writeFileSync(instrumented,fileEmitted);
  run('rustup',['run','1.96.1','rustc','--edition=2024','-F','unsafe_code','-A','warnings',...fileDependencies.args,instrumented,'-o',binary]);
  assert.equal(run(binary,[]),'not-found\n');
  fs.writeFileSync(source,`import {createTempDir,removeDirAll} from 'std:fs';function main(){const path=unwrap(createTempDir());unwrap(removeDirAll(path));console.log('temporary');}`);
  const temporaryEmitted=run(process.env.TSRUST,['emit',source]);
  const temporaryDependencies=prepareNativeDependencies({...options,emitted:temporaryEmitted});
  assert.deepEqual(temporaryDependencies.evidence.libraries.map(library=>library.name),['nix','tempfile']);
  fs.writeFileSync(instrumented,temporaryEmitted);
  run('rustup',['run','1.96.1','rustc','--edition=2024','-F','unsafe_code','-A','warnings',...temporaryDependencies.args,instrumented,'-o',binary]);
  assert.equal(run(binary,[]),'temporary\n');
  fs.writeFileSync(source,`import {hostname} from 'std:os';function main(){match(hostname()){Ok(value)=>{console.log(value.length>=0);},Err(error)=>{console.log(error.code==='Io'||error.code==='NotUnicode');}}}`);
  const hostnameEmitted=run(process.env.TSRUST,['emit',source]);
  const hostnameDependencies=prepareNativeDependencies({...options,emitted:hostnameEmitted});
  assert.deepEqual(hostnameDependencies.evidence.libraries.map(library=>library.name),['hostname']);
  fs.writeFileSync(instrumented,hostnameEmitted);
  run('rustup',['run','1.96.1','rustc','--edition=2024','-F','unsafe_code','-A','warnings',...hostnameDependencies.args,instrumented,'-o',binary]);
  assert.equal(run(binary,[]),'true\n');
  assert.ok(!fs.readdirSync(root).some(name=>name.startsWith('package-')));
 } finally {fs.rmSync(root,{recursive:true,force:true});}
});

test('JSON dependency discovery tolerates adjacent generated modules',()=>{
 for(const source of ['mod tsrust_native_json {', '}mod tsrust_native_json {', '}  mod tsrust_native_json{'])assert.equal(needsJsonDependencies(source),true);
 for(const source of ['fn main() {}', '// mod tsrust_native_json {', 'mod other {'])assert.equal(needsJsonDependencies(source),false);
});

test('reference dependency selection leaves ordinary/native preparation untouched',async()=>{
 const {prepareReferenceDependencies}=await import('./native-dependencies.mjs');
 assert.deepEqual(prepareReferenceDependencies({workload:{id:'native_json_parse'}}),{args:[],evidence:null});
 const target='/target';const artifact=JSON.stringify({reason:'compiler-artifact',target:{name:'unicode_normalization',kind:['lib']},filenames:['/target/release/deps/libunicode_normalization-a.rlib']});
 assert.deepEqual(dependencyArtifacts(artifact,target,['unicode_normalization']),{unicode_normalization:'/target/release/deps/libunicode_normalization-a.rlib'});
 assert.throws(()=>dependencyArtifacts(artifact,target,['rand']),/Missing.*rand/);
});

// Capture actual prepared Cargo commands; no Cargo, compiler or library executes.
test('reference dependency routing selects declared compression and existing family features',()=>{
 const manifest=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url),'utf8'));
 const compression=manifest.workloads.filter(row=>row.family==='compression');assert.equal(compression.length,36);
 const backend=new URL('../../crates/tsrust-compiler/backend-dependencies/Cargo.toml',import.meta.url),parity=new URL('./parity-dependencies/Cargo.toml',import.meta.url);
 const expected=[...compression.map(row=>[row,backend,row.id.includes('_buffer_')?'compression':'compression-streams']),
  [{family:'explicit-locale'},backend,'locale'],[{family:'child-processes'},backend,'process'],[{family:'network-transport'},backend,'io'],
  [{family:'async-io'},parity,'async-io'],[{id:'task_all_ready'},parity,'task-join'],[{id:'task_all_yield'},parity,'task-join'],
  [{family:'regex-text'},parity,'regex'],[{family:'path-env-os',id:'os_hostname_read'},parity,'hostname'],
  ...[{family:'native-url'},{family:'byte-operations',id:'byte_ops_index_ascii'},{family:'byte-codecs',id:'bytes_base64_encode'},{id:'math_random_range'},{id:'string_normalize_forms'}].map(row=>[row,parity,null])];
 const original=childProcess.spawnSync,sentinel=new Error('captured reference preparation');let observed;
 childProcess.spawnSync=(command,args,options)=>{observed={command,args,options};throw sentinel;};syncBuiltinESMExports();
 try{for(const [workload,url,feature]of expected){
  observed=undefined;assert.throws(()=>prepareReferenceDependencies({workload,rustVersion:'1.96.1',targetDirectory:'/unused-reference-target'}),error=>error===sentinel);
  assert.equal(observed.command,'rustup');assert.deepEqual(observed.args.slice(0,4),['run','1.96.1','cargo','build']);
  assert.equal(observed.args[observed.args.indexOf('--manifest-path')+1],fileURLToPath(url));
  assert.equal(path.resolve(observed.options.cwd),path.dirname(fileURLToPath(url)));
  const index=observed.args.indexOf('--features');assert.deepEqual(index<0?[]:observed.args.slice(index),feature?['--features',feature]:[]);
  if(feature){const source=fs.readFileSync(url,'utf8');assert(source.includes('\n'+feature+' = '),'selected manifest does not declare '+feature);}
  for(const flag of ['--offline','--locked','--release','--message-format=json'])assert(observed.args.includes(flag));
 }
 assert.deepEqual(prepareReferenceDependencies({workload:{family:'ordinary'}}),{args:[],evidence:null});
 }finally{childProcess.spawnSync=original;syncBuiltinESMExports();}
});
