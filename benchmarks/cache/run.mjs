import fs from 'node:fs';
import {assertMatchingSource} from './source-contract.mjs';
import path from 'node:path';
import os from 'node:os';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../..');
const verify=process.argv.includes('--verify');
const workload='cache-lru';
const expectedChecksum=3*256*255/2+16;
const compiler=process.env.TSRUST;
if(!compiler) throw new Error('Set TSRUST to the current verified compiler.');
if(process.version!=='v24.11.1') throw new Error('Node v24.11.1 is required.');
const hash=value=>createHash('sha256').update(value).digest('hex');
const run=(command,args,options={})=>{
 const result=spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:600000,...options});
 if(result.error||result.status!==0) throw new Error(`${command}: ${result.error??result.stderr}`);
 return result.stdout.trim();
};
const build=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-cache-workload-'));
try{
 const native=fs.readFileSync(path.join(here,`${workload}.tsr`),'utf8');
 const node=fs.readFileSync(path.join(here,`${workload}.ts`),'utf8');
 const manifest=JSON.parse(fs.readFileSync(path.join(here,'manifest.json'),'utf8'));
 assertMatchingSource(native,manifest,'native');assertMatchingSource(node,manifest,'typescript');
 const probe=source=>verify?source.replace(/for\s*\(let warmup\s*=\s*0;\s*warmup\s*<\s*8;/,'for(let warmup=0;warmup<0;').replace(/sample\s*<\s*31/,'sample<1'):source;
 const input=path.join(build,'native.tsr');fs.writeFileSync(input,probe(native));
 const ts=path.join(build,'node.ts');fs.writeFileSync(ts,probe(node));
 run(compiler,['format',input,'--write']);const once=fs.readFileSync(input);run(compiler,['format',input,'--write']);if(!once.equals(fs.readFileSync(input)))throw new Error('Formatting is not idempotent');
 const binary=path.join(build,'native');
 run(compiler,['build',input,'--release','-o',binary]);
 const tsc=path.join(root,'benchmarks/runtime/node_modules/typescript/bin/tsc');
 run(process.execPath,[tsc,'--target','ES2022','--module','commonjs','--strict','--esModuleInterop','--skipLibCheck','--typeRoots',path.join(root,'benchmarks/runtime/node_modules/@types'),'--outDir',build,ts]);
 const execution=[];
 const observations=[];
 for(let processIndex=0;processIndex<(verify?1:3);processIndex++)for(const engine of processIndex%2?['node-tsc','native']:['native','node-tsc']){
  const text=engine==='native'?run(binary,[]):run(process.execPath,[path.join(build,'node.js')]);
  const values=text.split('\n').map(Number);const samples=verify?1:31;
  if(values.length!==samples*2||values.some((value,index)=>!Number.isFinite(value)||(index%2===0?value!==expectedChecksum:value<0)))throw new Error(`Incorrect ${engine} result`);
  observations.push({engine,processIndex,checksums:values.filter((_,index)=>index%2===0)});
  if(!verify)execution.push({workload,engine,processIndex,durations:values.filter((_,index)=>index%2===1)});
 }
 const result={schemaVersion:1,measurementScope:'execution-only',semanticEpoch:'owned-coordination-v1',measuredEngines:['native','node-tsc'],manifest:{workloads:[{id:workload,sourceSha256:hash(native),counterpartSha256:hash(node),iterations:256,operationsPerIteration:1,notes:'Bounded scalar LRU insertion and lookup; entry capacity 16.'}]},sources:[{id:workload,tsrSha256:hash(native),tsSha256:hash(node)}],kind:`cache-${workload}`,verifiedOnly:verify,timestamp:new Date().toISOString(),sourceCommit:run('git',['rev-parse','HEAD']),sourceTreeDirty:run('git',['status','--porcelain']).length>0,sourceSha256:hash(native),counterpartSha256:hash(node),compilerSha256:hash(fs.readFileSync(compiler)),harnessSha256:hash(fs.readFileSync(fileURLToPath(import.meta.url))),nativeLockSha256:hash(fs.readFileSync(path.join(root,"crates/tsrust-compiler/native-dependencies/Cargo.lock"))),versions:{node:process.version,rust:run('rustup',['run','1.96.1','rustc','--version']),typescript:run(process.execPath,[tsc,'--version'])},machine:{cpu:os.cpus()[0].model,platform:os.platform(),release:os.release()},protocol:{processes:verify?1:3,warmups:verify?0:8,samples:verify?1:31,iterationsPerBatch:256,durationUnit:'milliseconds'},expectedChecksum,observations,execution};
 const output=path.join(here,`${workload}-${result.timestamp.replace(/[:.]/g,'-')}${verify?'-verify':''}.json`);
 fs.writeFileSync(output,JSON.stringify(result,null,2)+'\n',{flag:'wx'});console.log(output);
}finally{fs.rmSync(build,{recursive:true,force:true});}
