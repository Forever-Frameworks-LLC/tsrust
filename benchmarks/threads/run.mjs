import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import { createHash } from 'node:crypto';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../..');
const verify=process.argv.includes('--verify');
const dataBridge=process.argv.includes('--data-bridge');
const throughput=process.argv.includes('--throughput')||process.argv.includes('--bridge')||dataBridge;
const bridge=process.argv.includes('--bridge')||dataBridge;
const workload=dataBridge?'rust-data-throughput':bridge?'rust-bridge-throughput':throughput?'channel-throughput':process.argv.includes('--pool')?'pool-join':process.argv.includes('--scoped')?'scoped-join':'spawn-join';
const compiler=process.env.TSRUST;
if(!compiler) throw new Error('Set TSRUST to the current verified compiler.');
if(process.version!=='v24.11.1') throw new Error('Node v24.11.1 is required.');
const hash=value=>createHash('sha256').update(value).digest('hex');
const run=(command,args,options={})=>{
 const result=spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:180000,...options});
 if(result.error||result.status!==0) throw new Error(`${command}: ${result.error??result.stderr}`);
 return result.stdout.trim();
};
const build=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-thread-workload-'));
try{
 const native=fs.readFileSync(path.join(here,`${workload}.tsr`),'utf8');
 const node=fs.readFileSync(path.join(here,`${workload}.ts`),'utf8');
 const probe=source=>verify?source.replace(/for\s*\(let warmup\s*=\s*0;\s*warmup\s*<\s*8;/g,'for(let warmup=0;warmup<0;').replace(/sample\s*<\s*31/g,'sample<1'):source;
 const rust=bridge?fs.readFileSync(path.join(here,dataBridge?'rust-data-compute.rs':'rust-bridge-compute.rs'),'utf8'):null;
 if(rust)fs.writeFileSync(path.join(build,dataBridge?'rust-data-compute.rs':'rust-bridge-compute.rs'),rust);
 const input=path.join(build,'native.tsr');fs.writeFileSync(input,probe(native));
 const ts=path.join(build,'node.ts');fs.writeFileSync(ts,probe(node));
 run(compiler,['format',input,'--write']);const once=fs.readFileSync(input);run(compiler,['format',input,'--write']);if(!once.equals(fs.readFileSync(input)))throw new Error('Formatting is not idempotent');
 const binary=path.join(build,'native');
 run(compiler,['build',input,'--release','-o',binary]);
 const tsc=path.join(root,'benchmarks/runtime/node_modules/typescript/bin/tsc');
 run(process.execPath,[tsc,'--target','ES2022','--module','commonjs','--strict','--skipLibCheck','--typeRoots',path.join(root,'benchmarks/runtime/node_modules/@types'),'--outDir',build,ts]);
 const expectedChecksum=throughput?Array.from({length:4},(_,index)=>{let value=BigInt(index+1);for(let step=0;step<250000;step++)value=value*48271n%2147483647n;return Number(value);}).reduce((a,b)=>a+b,0):140;
 const execution=[];
 const observations=[];
 for(let processIndex=0;processIndex<(verify?1:3);processIndex++)for(const engine of processIndex%2?['node-tsc','native']:['native','node-tsc']){
  const text=engine==='native'?run(binary,[]):run(process.execPath,[path.join(build,'node.js')]);
  const values=text.split('\n').map(Number);const samples=verify?1:31;
  if(throughput){
   if(values.length!==samples*9)throw new Error(`Incorrect ${engine} sample count`);
   for(let variant=0;variant<3;variant++){const workers=[0,2,4][variant];const block=values.slice(variant*samples*3,(variant+1)*samples*3);if(block.some((value,index)=>!Number.isFinite(value)||(index%3===0?value!==workers:index%3===1?value!==expectedChecksum:value<0)))throw new Error(`Incorrect ${engine} throughput result`);observations.push({engine,processIndex,workers,checksums:block.filter((_,index)=>index%3===1)});if(!verify)execution.push({engine,processIndex,workers,durations:block.filter((_,index)=>index%3===2)});}
   continue;
  }
  if(values.length!==samples*2||values.some((value,index)=>!Number.isFinite(value)||(index%2===0?value!==140:value<0)))throw new Error(`Incorrect ${engine} result`);
  observations.push({engine,processIndex,checksums:values.filter((_,index)=>index%2===0)});
  if(!verify)execution.push({engine,processIndex,durations:values.filter((_,index)=>index%2===1)});
 }
 const result={kind:`owned-thread-${workload}`,verifiedOnly:verify,timestamp:new Date().toISOString(),sourceCommit:run('git',['rev-parse','HEAD']),sourceTreeDirty:run('git',['status','--porcelain']).length>0,sourceSha256:hash(native),counterpartSha256:hash(node),rustSha256:rust?hash(rust):undefined,compilerSha256:hash(fs.readFileSync(compiler)),harnessSha256:hash(fs.readFileSync(fileURLToPath(import.meta.url))),versions:{node:process.version,rust:run('rustup',['run','1.96.1','rustc','--version']),typescript:run(process.execPath,[tsc,'--version'])},machine:{cpu:os.cpus()[0].model,logicalCpus:os.cpus().length,availableParallelism:os.availableParallelism(),platform:os.platform(),release:os.release()},protocol:{processes:verify?1:3,warmups:verify?0:8,samples:verify?1:31,workersPerBatch:throughput?4:8,workerCounts:throughput?[0,2,4]:undefined,durationUnit:'milliseconds'},expectedChecksum,observations,execution};
 const output=path.join(here,`${workload}-${result.timestamp.replace(/[:.]/g,'-')}${verify?'-verify':''}.json`);
 fs.writeFileSync(output,JSON.stringify(result,null,2)+'\n',{flag:'wx'});console.log(output);
}finally{fs.rmSync(build,{recursive:true,force:true});}
