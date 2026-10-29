import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
const compiler=process.env.TSRUST;
if(!compiler) throw new Error('Set TSRUST to the verified compiler.');
if(process.argv.slice(2).some(arg=>!['--verify','--tls','--state','--service','--context','--producer','--static','--readers'].includes(arg))) throw new Error('Unknown option.');
const verify=process.argv.includes('--verify');
if(['--tls','--state','--service','--context','--producer','--static','--readers'].filter(flag=>process.argv.includes(flag)).length>1) throw new Error('Choose one workload.');
const workload=process.argv.includes('--readers')?'request-readers':process.argv.includes('--static')?'static-file':process.argv.includes('--producer')?'producer-stream':process.argv.includes('--context')?'context-middleware':process.argv.includes('--service')?'service-boundary':process.argv.includes('--state')?'retained-state':process.argv.includes('--tls')?'https':'loopback';
const requiredNode=workload==='static-file'?'v24.19.0':'v24.11.1';
if(process.version!==requiredNode) throw new Error(`Node ${requiredNode} is required.`);
const project=path.resolve(root,'../..');
const tsc=path.join(project,'benchmarks/runtime/node_modules/typescript/bin/tsc');
const scratch=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-http-'));
const run=(command,args)=>execFileSync(command,args,{encoding:'utf8',timeout:300000,cwd:project,env:{...process.env,...(workload==='static-file'?{TSRUST_STATIC_BENCH_ROOT:path.join(scratch,'assets')}:{})}});
try {
 if(workload==='static-file'){fs.mkdirSync(path.join(scratch,'assets'));fs.writeFileSync(path.join(scratch,'assets/value.txt'),'42');}
 const source=path.join(root,`${workload}.tsr`);
 const counterpart=path.join(root,`${workload}.ts`);
 if(workload==='static-file'){const manifest=JSON.parse(fs.readFileSync(path.join(root,'static-manifest.json'),'utf8'));for(const [extension,file] of [['tsr',source],['ts',counterpart]]){if(crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex')!==manifest.sources[extension])throw new Error('Static workload source identity changed');}if(manifest.expected!==128*42||manifest.fixture!=='42')throw new Error('Static oracle changed');}
 if(workload==='retained-state'){const {assertStateSource}=await import('./state-contract.mjs');const manifest=JSON.parse(fs.readFileSync(path.join(root,'state-manifest.json'),'utf8'));assertStateSource(fs.readFileSync(source,'utf8'),manifest,'native');assertStateSource(fs.readFileSync(counterpart,'utf8'),manifest,'typescript');}
 if(workload==='context-middleware'){const manifest=JSON.parse(fs.readFileSync(path.join(root,'context-manifest.json'),'utf8'));for(const [extension,file] of [['tsr',source],['ts',counterpart]]){if(crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex')!==manifest.sources[extension])throw new Error('Context workload source identity changed');}if(manifest.expected!==128*42)throw new Error('Context oracle changed');}
 if(workload==='producer-stream'){const manifest=JSON.parse(fs.readFileSync(path.join(root,'producer-manifest.json'),'utf8'));for(const [extension,file] of [['tsr',source],['ts',counterpart]]){if(crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex')!==manifest.sources[extension])throw new Error('Producer workload source identity changed');}if(manifest.expected!==128*6*7)throw new Error('Producer oracle changed');}
 if(workload==='request-readers'){const manifest=JSON.parse(fs.readFileSync(path.join(root,'readers-manifest.json'),'utf8'));for(const [extension,file] of [['tsr',source],['ts',counterpart]]){if(crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex')!==manifest.sources[extension])throw new Error('Request reader source identity changed');}if(manifest.expected!==128*(40+2))throw new Error('Request reader oracle changed');}
 if(workload==='service-boundary'){const manifest=JSON.parse(fs.readFileSync(path.join(root,'service-manifest.json'),'utf8'));for(const [extension,file] of [['tsr',source],['ts',counterpart]]){if(crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex')!==manifest.sources[extension])throw new Error('Service workload source identity changed');}if(manifest.expected!==128*(32+7+2+1))throw new Error('Service oracle changed');}
 const probe=text=>verify?text.replace(/warmup < 8/g,'warmup < 0').replace(/sample < 31/g,'sample < 1'):text;
 const input=path.join(scratch,'native.tsr');
 fs.writeFileSync(input,probe(fs.readFileSync(source,'utf8')));
 fs.writeFileSync(path.join(scratch,'node.ts'),probe(fs.readFileSync(counterpart,'utf8')));
 run(compiler,['format',input,'--write']);const once=fs.readFileSync(input);run(compiler,['format',input,'--write']);if(!once.equals(fs.readFileSync(input))) throw new Error('Formatting is not idempotent.');
 const native=path.join(scratch,'native');
 run(compiler,['build',input,...(verify?[]:['--release']),'-o',native]);
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
 if(workload==='https')record.fixtureSha256=Object.fromEntries(['ca.pem','server.pem','server.key'].map(name=>[name,hash(path.join(project,'crates/tsrust-compiler/tests/fixtures/tls',name))]));
 if(workload==='service-boundary'){record.schemaVersion=1;record.measurementScope='execution-only';record.semanticEpoch='service-boundary-v1';record.manifest={workloads:[{id:workload,sourceSha256:hash(source),counterpartSha256:hash(counterpart),iterations:128,operationsPerIteration:1}]};record.sources=[{id:workload,tsrSha256:hash(source),tsSha256:hash(counterpart)}];for(const item of execution)item.workload=workload;record.oracleInputs={sha256Input:'a'.repeat(64),form:'a=one+two&a=%C3%A9',decodedUtf8Lengths:[7,2],digestLength:32,successIncrement:1,batch:128};}
 if(workload==='context-middleware'){record.semanticEpoch=JSON.parse(fs.readFileSync(path.join(root,'context-manifest.json'),'utf8')).semanticEpoch;record.oracleInputs={body:'42',headers:{'x-before':'entry','x-after':'exit'},batch:128};}
 if(workload==='producer-stream'){record.semanticEpoch='producer-stream-v1';record.measurementScope='in-process bounded producer and body collection';record.oracleInputs={chunks:6,chunk:'abcdefg',capacity:1,batch:128};record.protocol.bodiesPerBatch=128;delete record.protocol.requestsPerBatch;}
 if(workload==='static-file'){record.semanticEpoch='static-file-v1';record.measurementScope='loopback HTTP file metadata/open/stream and body collection; warm filesystem cache; excludes startup/build';record.oracleInputs={file:'value.txt',utf8:'42',bytes:[52,50],value:42,batch:128};record.schemaVersion=1;record.manifest={workloads:[{id:workload,sourceSha256:hash(source),counterpartSha256:hash(counterpart),iterations:128,operationsPerIteration:1}]};record.sources=[{id:workload,tsrSha256:hash(source),tsSha256:hash(counterpart)}];for(const item of execution)item.workload=workload;}
 if(workload==='request-readers'){record.semanticEpoch='request-readers-v1';record.measurementScope='loopback JSON/schema and contextual streaming requests; excludes startup/build';record.oracleInputs={json:'{"value":40}',stream:'é',utf8Bytes:2,batch:128};record.protocol.requestsPerBatch=256;record.schemaVersion=1;record.manifest={workloads:[{id:workload,sourceSha256:hash(source),counterpartSha256:hash(counterpart),iterations:128,operationsPerIteration:2}]};record.sources=[{id:workload,tsrSha256:hash(source),tsSha256:hash(counterpart)}];for(const item of execution)item.workload=workload;}
 const output=path.join(root,`${workload}-${record.timestamp.replaceAll(':','-')}${verify?'-verify':''}.json`);
 fs.writeFileSync(output,JSON.stringify(record,null,2)+'\n',{flag:'wx'});console.log(output);
} finally {fs.rmSync(scratch,{recursive:true,force:true});}
