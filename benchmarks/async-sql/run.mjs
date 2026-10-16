import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url)),project=path.resolve(root,'../..');
const compiler=process.env.TSRUST,driver=process.env.TSRUST_SQL_DRIVER??'sqlite';
const url=process.env.TSRUST_SQL_URL??(driver==='sqlite'?'sqlite::memory:':null);
if(!compiler||!url||!['sqlite','postgres','mysql'].includes(driver))throw new Error('Set TSRUST, TSRUST_SQL_DRIVER and TSRUST_SQL_URL (a disposable benchmark database).');
if(process.version!=='v24.11.1')throw new Error('Node v24.11.1 required.');
if(process.argv.slice(2).some(arg=>!['--verify','--cursor','--jobs'].includes(arg)))throw new Error('Unknown option.');
const cursor=process.argv.includes('--cursor'),jobs=process.argv.includes('--jobs');
if(cursor&&jobs)throw new Error('Choose one workload.');
if((cursor||jobs)&&(driver!=='sqlite'||url!=='sqlite::memory:'))throw new Error('Cursor/jobs benchmarks currently support only in-memory SQLite; driver correctness uses the explicit three-driver runner.');
const workload=cursor?'cursor':jobs?'jobs':'prepared';
const expected=jobs?128:128*127;
const verify=process.argv.includes('--verify'),scratch=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-async-sql-'));
const run=(program,args)=>execFileSync(program,args,{cwd:project,encoding:'utf8',timeout:600000,env:{...process.env,TSRUST_SQL_DRIVER:driver,TSRUST_SQL_URL:url,NODE_PATH:path.join(root,'node_modules')}});
const hash=file=>crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
try{
 const source=path.join(root,`${workload}.tsr`),counterpart=path.join(root,`${workload}.ts`);
 if(cursor||jobs){const manifest=JSON.parse(fs.readFileSync(path.join(root,`${workload}-manifest.json`),'utf8'));for(const [extension,file] of [['tsr',source],['ts',counterpart]])if(hash(file)!==manifest.sources[extension])throw new Error('Workload source identity changed');if(manifest.expected!==expected)throw new Error('Workload oracle changed');}
 if(jobs){const helper=path.join(project,'examples/durable-jobs/jobs.tsr');const manifest=JSON.parse(fs.readFileSync(path.join(root,'jobs-manifest.json'),'utf8'));if(hash(helper)!==manifest.helperSha256)throw new Error('Job helper identity changed');fs.copyFileSync(helper,path.join(scratch,'jobs.tsr'));}
 let nativeSource=fs.readFileSync(source,'utf8').replace("'sqlite::memory:'",JSON.stringify(url));
 if(jobs)nativeSource=nativeSource.replace('../../examples/durable-jobs/jobs.tsr','./jobs.tsr');
 if(driver==='postgres')nativeSource=nativeSource.replace('VALUES (?, ?)','VALUES ($1, $2)');
 if(driver==='mysql')nativeSource=nativeSource.replace('AS BIGINT','AS SIGNED');
 const probe=text=>verify?text.replace(/warmup\s*<\s*8/g,'warmup < 0').replace(/sample\s*<\s*31/g,'sample < 1'):text;
 const input=path.join(scratch,'native.tsr'),native=path.join(scratch,'native');
 fs.writeFileSync(input,probe(nativeSource));fs.writeFileSync(path.join(scratch,'node.ts'),probe(fs.readFileSync(counterpart,'utf8')));
 run(compiler,['format',input,'--write']);const once=fs.readFileSync(input);run(compiler,['format',input,'--write']);if(!once.equals(fs.readFileSync(input)))throw new Error('Formatting not idempotent');
 run(compiler,['build',input,...((cursor||jobs)&&verify?[]:['--release']),'-o',native]);
 const tsc=path.join(project,'benchmarks/runtime/node_modules/typescript/bin/tsc');run(process.execPath,[tsc,path.join(scratch,'node.ts'),'--target','ES2022','--module','commonjs','--outDir',scratch,'--strict']);
 const observations=[],execution=[];
 for(let processIndex=0;processIndex<(verify?1:3);processIndex++)for(const engine of processIndex%2?['node-tsc','native']:['native','node-tsc']){
  const result=(engine==='native'?run(native,[]):run(process.execPath,[path.join(scratch,'node.js')])).trim().split('\n').map(Number);
  if(result.length!==(verify?1:31)*2||result.some((value,index)=>!Number.isFinite(value)||(index%2===0?value!==expected:value<0)))throw new Error(`Bad ${driver}/${engine} output`);
  observations.push({engine,processIndex,checksums:result.filter((_,index)=>index%2===0)});
  if(!verify)execution.push({engine,processIndex,durations:result.filter((_,index)=>index%2===1)});
 }
 const record={kind:`async-sql-${workload}`,driver,verifiedOnly:verify,timestamp:new Date().toISOString(),sourceCommit:run('git',['rev-parse','HEAD']).trim(),sourceTreeDirty:run('git',['status','--porcelain']).trim().length>0,sourceSha256:hash(source),counterpartSha256:hash(counterpart),compilerSha256:hash(compiler),harnessSha256:hash(fileURLToPath(import.meta.url)),nativeLockSha256:hash(path.join(project,'crates/tsrust-compiler/native-dependencies/Cargo.lock')),nodeLockSha256:hash(path.join(root,'package-lock.json')),versions:{node:process.version,sqlx:'0.9.0'},machine:{cpu:os.cpus()[0].model,logicalCpus:os.cpus().length,platform:os.platform(),release:os.release()},protocol:{processes:verify?1:3,warmups:verify?0:8,samples:verify?1:31,insertsPerBatch:128,durationUnit:'milliseconds'},expected,observations,execution};
 if(jobs){record.semanticEpoch='sql-jobs-v1';record.helperSha256=hash(path.join(project,'examples/durable-jobs/jobs.tsr'));record.protocol.jobsPerBatch=128;delete record.protocol.insertsPerBatch;record.oracleInputs={jobs:128,effectsPerJob:1};}
 if(cursor){record.semanticEpoch='sql-cursor-v1';record.protocol.rowsPerBatch=128;delete record.protocol.insertsPerBatch;record.oracleInputs={rows:128,valueMultiplier:2,order:'id ascending'};}
 const output=path.join(root,`${workload}-${driver}-${record.timestamp.replaceAll(':','-')}${verify?'-verify':''}.json`);fs.writeFileSync(output,JSON.stringify(record,null,2)+'\n',{flag:'wx'});console.log(output);
}finally{fs.rmSync(scratch,{recursive:true,force:true});}
