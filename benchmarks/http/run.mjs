import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const compiler=process.env.TSRUST;
if(!compiler) throw new Error('Set TSRUST to the verified compiler.');
if(process.argv.slice(2).some(arg=>!['--verify'].includes(arg))) throw new Error('Unknown option.');
const verify=process.argv.includes('--verify');
const workload='loopback';
if(process.version!=='v24.11.1') throw new Error('Node v24.11.1 is required.');
const project=path.resolve(root,'../..');
const tsc=path.join(project,'benchmarks/runtime/node_modules/typescript/bin/tsc');
const scratch=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-http-'));
const run=(command,args)=>execFileSync(command,args,{encoding:'utf8',timeout:300000});
try {
 const source=path.join(root,`${workload}.tsr`);
 const counterpart=path.join(root,`${workload}.ts`);
 const probe=text=>verify?text.replace(/warmup < 8/g,'warmup < 0').replace(/sample < 31/g,'sample < 1'):text;
 const input=path.join(scratch,'native.tsr');
 fs.writeFileSync(input,probe(fs.readFileSync(source,'utf8')));
 fs.writeFileSync(path.join(scratch,'node.ts'),probe(fs.readFileSync(counterpart,'utf8')));
 run(compiler,['format',input,'--write']);const once=fs.readFileSync(input);run(compiler,['format',input,'--write']);if(!once.equals(fs.readFileSync(input))) throw new Error('Formatting is not idempotent.');
 const native=path.join(scratch,'native');
 run(compiler,['build',input,'--release','-o',native]);
 run(process.execPath,[tsc,path.join(scratch,'node.ts'),'--target','ES2022','--module','commonjs','--outDir',scratch,'--strict']);
 const expected=128*42;
 const observations=[];const execution=[];
 for(let processIndex=0;processIndex<(verify?1:3);processIndex++) for(const engine of processIndex%2?['node-tsc','native']:['native','node-tsc']) {
  const result=(engine==='native'?run(native,[]):run(process.execPath,[path.join(scratch,'node.js')])).trim().split('\n').map(Number);
  if(result.length!==(verify?1:31)*2 || result.some((value,index)=>!Number.isFinite(value)||(index%2===0?value!==expected:value<0))) throw new Error(`Invalid ${engine} checksum, sample count or duration.`);
  observations.push({engine,processIndex,checksums:result.filter((_,index)=>index%2===0)});
  if(!verify)execution.push({engine,processIndex,durations:result.filter((_,index)=>index%2===1)});
 }
 const hash=file=>crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
 const record={kind:`http-${workload}`,verifiedOnly:verify,timestamp:new Date().toISOString(),sourceCommit:run('git',['rev-parse','HEAD']).trim(),sourceTreeDirty:run('git',['status','--porcelain']).trim().length>0,sourceSha256:hash(source),counterpartSha256:hash(path.join(root,`${workload}.ts`)),compilerSha256:hash(compiler),harnessSha256:hash(fileURLToPath(import.meta.url)),versions:{node:process.version,rust:run('rustup',['run','1.96.1','rustc','--version']).trim(),typescript:run(process.execPath,[tsc,'--version']).trim()},machine:{cpu:os.cpus()[0].model,logicalCpus:os.cpus().length,availableParallelism:os.availableParallelism(),platform:os.platform(),release:os.release()},protocol:{processes:verify?1:3,warmups:verify?0:8,samples:verify?1:31,requestsPerBatch:128,durationUnit:'milliseconds'},expected,observations,execution};
 const output=path.join(root,`${workload}-${record.timestamp.replaceAll(':','-')}${verify?'-verify':''}.json`);
 fs.writeFileSync(output,JSON.stringify(record,null,2)+'\n',{flag:'wx'});console.log(output);
} finally {fs.rmSync(scratch,{recursive:true,force:true});}
