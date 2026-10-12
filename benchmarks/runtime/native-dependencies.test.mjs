import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {prepareNativeDependencies, dependencyArtifacts, needsJsonDependencies} from './native-dependencies.mjs';

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

test('exported JSON dependencies link a separately instrumented native program offline',{skip:!process.env.TSRUST},()=>{
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
  const instrumented=path.join(root,'probe.rs'),binary=path.join(root,'probe');
  fs.writeFileSync(instrumented,emitted.replace('fn main()','fn source_main()')+'\nfn main(){source_main();println!("instrumented");}\n');
  run('rustup',['run','1.96.1','rustc','--edition=2024','-A','warnings','-C','overflow-checks=on',...dependencies.args,instrumented,'-o',binary]);
  assert.equal(run(binary,[]),'null\ninstrumented\n');
  assert.throws(()=>prepareNativeDependencies({...options,emitted:emitted+'\n// changed'}),/differs/);
  assert.ok(!fs.readdirSync(root).some(name=>name.startsWith('package-')));
 } finally {fs.rmSync(root,{recursive:true,force:true});}
});

test('JSON dependency discovery tolerates adjacent generated modules',()=>{
 for(const source of ['mod tsrust_native_json {', '}mod tsrust_native_json {', '}  mod tsrust_native_json{'])assert.equal(needsJsonDependencies(source),true);
 for(const source of ['fn main() {}', '// mod tsrust_native_json {', 'mod other {'])assert.equal(needsJsonDependencies(source),false);
});
