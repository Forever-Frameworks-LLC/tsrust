// Disposable direct-backend admission profile. This does not ingest benchmark rows.
import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {waitForQuietMachine} from '../quiet-machine.mjs';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../../..');
const sharedTarget=process.env.CARGO_TARGET_DIR;
if(!sharedTarget||!path.isAbsolute(sharedTarget))throw new Error('Use the coordinated absolute shared CARGO_TARGET_DIR.');
const build=path.join(sharedTarget,'byte-operations-prototype');fs.mkdirSync(build,{recursive:true});
const compilerVersion=fs.readFileSync(path.join(root,'crates/tsrust-compiler/src/lib.rs'),'utf8').match(/pub const RUST_BACKEND_VERSION: &str = "([^"]+)"/)[1];
const source=path.join(here,'byte-operations.rs'),binary=path.join(build,`profile${process.platform==='win32'?'.exe':''}`);
const run=(command,args)=>{const result=spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:180000,env:{...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1'},maxBuffer:4*1024*1024});if(result.error||result.status!==0)throw new Error(`${command} failed: ${result.error??result.stderr}`);return result.stdout;};
const nodeExecutable=process.argv.find(argument=>argument.startsWith('--node='))?.slice(7)??process.env.TSRUST_BENCH_NODE??process.execPath;
const nodeVersion=run(nodeExecutable,['--version']).trim();
if(nodeVersion!=='v24.11.1')throw new Error('Pass --node=<pinned Node24.11.1 binary>; prototype Node26 observations are supplementary.');
const started=performance.now();
run('rustup',['run',compilerVersion,'rustc','--edition=2024','-C','opt-level=3',source,'-o',binary]);
const buildMs=performance.now()-started;
// Existing lockfile candidate, compiled directly without a manifest/dependency edit.
const registry=path.join(process.env.CARGO_HOME??path.join(os.homedir(),'.cargo'),'registry/src');
const memchrSource=fs.readdirSync(registry).map(name=>path.join(registry,name,'memchr-2.8.3')).find(candidate=>fs.existsSync(path.join(candidate,'src/lib.rs')));
if(!memchrSource)throw new Error('Pinned memchr 2.8.3 candidate is not cached.');
const memchrLibrary=path.join(build,'libmemchr.rlib'),memchrBinary=path.join(build,`profile-memchr${process.platform==='win32'?'.exe':''}`);
const memchrStarted=performance.now();
run('rustup',['run',compilerVersion,'rustc','--edition=2021','--crate-name','memchr','--crate-type','rlib','-C','opt-level=3','--cfg','feature="std"','--cfg','feature="alloc"',path.join(memchrSource,'src/lib.rs'),'-o',memchrLibrary]);
run('rustup',['run',compilerVersion,'rustc','--edition=2024','-C','opt-level=3','--cfg','memchr_candidate','--extern',`memchr=${memchrLibrary}`,source,'-o',memchrBinary]);
const memchrBuildMs=performance.now()-memchrStarted;
run(nodeExecutable,[path.join(here,'byte-operations-node.mjs'),'--check-only']);
const measurements=[],loads=[];
for(let processIndex=0;processIndex<3;processIndex++){
 for(const engine of processIndex%2?['node','rust-memchr','rust']:['rust','rust-memchr','node']){
  loads.push(waitForQuietMachine({log:line=>process.stderr.write(`${line}\n`)}));
  const stdout=engine==='rust'?run(binary,[]):engine==='rust-memchr'?run(memchrBinary,[]):run(nodeExecutable,[path.join(here,'byte-operations-node.mjs')]);
  for(const line of stdout.trim().split('\n'))measurements.push({engine,processIndex,...JSON.parse(line)});
 }
}
const sources=Object.fromEntries(['byte-operations.rs','byte-operations-node.mjs','counting-allocator.rs','../byte-operations-oracle.mjs'].map(name=>[name,createHash('sha256').update(fs.readFileSync(path.resolve(here,name))).digest('hex')]));
const result={date:new Date().toISOString(),protocol:{processes:3,warmups:8,samples:31},compilerVersion,nodeVersion,nodeExecutable,sources,buildMs,binaryBytes:fs.statSync(binary).size,memchr:{version:'2.8.3',defaultFeatures:['std','alloc'],buildMs:memchrBuildMs,binaryBytes:fs.statSync(memchrBinary).size,rlibBytes:fs.statSync(memchrLibrary).size,manifestSha256:createHash('sha256').update(fs.readFileSync(path.join(memchrSource,'Cargo.toml'))).digest('hex')},quietLoads:loads,measurements};
const output=path.join(build,'results.json');fs.writeFileSync(output,JSON.stringify(result));console.log(output);
