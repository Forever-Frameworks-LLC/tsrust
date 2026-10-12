import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
const here=path.dirname(fileURLToPath(import.meta.url));
const root=path.resolve(here,'../../..');
const manifest=JSON.parse(fs.readFileSync(path.join(here,'manifest.json'),'utf8'));
const args=process.argv.slice(2);
const checkOnly=args.length===1 && args[0]==='--check-only';
assert(checkOnly || (args.length===2 && args[0]==='--output'),'Use --check-only or --output <new-result.json>');
const output=checkOnly?null:path.resolve(args[1]);
if(output) assert(!fs.existsSync(output),'Refuse to overwrite an existing record');
const compiler=path.resolve(process.env.TSRUST || path.join(root,'target/editor/debug/tsrust'));
const tsc=path.join(root,'benchmarks/runtime/node_modules/typescript/bin/tsc');
const sha=file=>crypto.createHash('sha256').update(fs.readFileSync(file)).digest('hex');
const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-stdlib-selection-'));
const samples=checkOnly?1:manifest.samples;
const commands=[];
let activeRecord;
let ownsOutput=false;
function save(){if(ownsOutput) fs.writeFileSync(output,JSON.stringify(activeRecord,null,2)+'\n');}
function run(command,arguments_,env=process.env){
 const start=process.hrtime.bigint();
 const result=spawnSync(command,arguments_,{cwd:root,env,encoding:'utf8',timeout:120000,maxBuffer:32*1024*1024});
 const ns=Number(process.hrtime.bigint()-start);
 assert(!result.error && result.status===0,`${command} ${arguments_.join(' ')}\n${result.error || result.stderr}`);
 return {...result,ns};
}
function observe(command,arguments_,env){const result=run(command,arguments_,env);commands.push({command,arguments:arguments_});return result;}
function build(source,cache,binary,expectedState){
 const env={...process.env,TSRUST_CACHE_DIR:cache,TSRUST_BUILD_TIMINGS:'1'};
 const result=observe(compiler,['build',source,'-o',binary],env);
 assert(result.stderr.includes(`cache=${expectedState} (`),result.stderr);
 return result;
}
try {
 const sourceFiles=[path.join(here,'manifest.json'),fileURLToPath(import.meta.url),path.join(root,'benchmarks/runtime/tsconfig.json'),path.join(root,'benchmarks/runtime/package-lock.json'),path.join(root,'Cargo.lock')];
 for(const workload of manifest.workloads) for(const file of fs.readdirSync(path.join(here,workload.directory)).sort()) sourceFiles.push(path.join(here,workload.directory,file));
 const record={schemaVersion:1,status:'running',kind:checkOnly?'stdlib-selection-correctness':'stdlib-selection-timing',createdAt:new Date().toISOString(),
  gitHead:run('git',['rev-parse','HEAD']).stdout.trim(),platform:process.platform,architecture:process.arch,cpu:os.cpus()[0]?.model,node:process.version,
  compiler:{path:compiler,sha256:sha(compiler),version:run(compiler,['--version']).stdout.trim()},
  backend:run('rustup',['run','1.96.1','rustc','-vV']).stdout,typescript:run(process.execPath,[tsc,'--version']).stdout.trim(),
  sources:Object.fromEntries(sourceFiles.map(file=>[path.relative(root,file),sha(file)])),
  protocol:{samples:checkOnly?0:samples,correctnessOnly:checkOnly,manifestSamples:manifest.samples,buildDiagnostics:'TSRUST_BUILD_TIMINGS=1; included in build wall time',cold:'fresh isolated cache each sample',warm:'one excluded priming build per workload',scope:'process wall time, not runtime throughput; native and TypeScript validation guarantees differ'},workloads:[],commands};
 activeRecord=record;
 if(output){fs.writeFileSync(output,JSON.stringify(record,null,2)+'\n',{flag:'wx'});ownsOutput=true;}
 for(const workload of manifest.workloads){
  const directory=path.join(here,workload.directory),source=path.join(directory,'main.tsr');
  const binary=path.join(temporary,`${workload.id}-program`),compiled=path.join(temporary,`${workload.id}-ts`),config=path.join(temporary,`${workload.id}.json`);
  fs.writeFileSync(config,JSON.stringify({extends:path.join(root,'benchmarks/runtime/tsconfig.json'),files:[path.join(directory,'main.ts')],compilerOptions:{outDir:compiled}}));
  observe(process.execPath,[tsc,'--project',config]);assert.equal(run(process.execPath,[path.join(compiled,'main.js')]).stdout,workload.stdout);
  const rows={id:workload.id,samples:[]};
  record.workloads.push(rows);
  const warmCache=path.join(temporary,`${workload.id}-warm`);build(source,warmCache,binary,'miss');assert.equal(run(binary,[]).stdout,workload.stdout);
  for(let sample=0;sample<samples;sample++){
   const check=observe(compiler,['check',source]);const typescript=observe(process.execPath,[tsc,'--project',config,'--noEmit']);
   const emit=observe(compiler,['emit',source]);assert.equal(emit.stdout.includes('mod tsrust_native_path'),workload.nativeFeatures.includes('path'));
   const coldCache=path.join(temporary,`${workload.id}-cold-${sample}`);
   const cold=build(source,coldCache,binary,'miss');assert.equal(run(binary,[]).stdout,workload.stdout);
   fs.rmSync(coldCache,{recursive:true,force:true});
   const warm=build(source,warmCache,binary,'hit');assert.equal(run(binary,[]).stdout,workload.stdout);
   rows.samples.push(checkOnly?{sample,verified:true}:{sample,frontendCheckNs:check.ns,typescriptCheckNs:typescript.ns,frontendEmitNs:emit.ns,coldBuildNs:cold.ns,warmBuildNs:warm.ns,coldDiagnostics:cold.stderr,warmDiagnostics:warm.stderr,emittedSha256:crypto.createHash('sha256').update(emit.stdout).digest('hex')});
   save();
  }
 }
 const transition=path.join(temporary,'transition'),binary=path.join(temporary,'transition-program');
 for(const workload of manifest.workloads){build(path.join(here,workload.directory,'main.tsr'),transition,binary,'miss');assert.equal(run(binary,[]).stdout,workload.stdout);}
 record.featureTransitionVerified=true;
 record.status='complete';
 if(output){save();console.log(output);}
 else console.log(JSON.stringify(record,null,2));
} catch(error) {
 if(activeRecord && ownsOutput){activeRecord.status='failed';activeRecord.failure=String(error);save();}
 throw error;
} finally {fs.rmSync(temporary,{recursive:true,force:true});}
