import fs from 'node:fs';
import path from 'node:path';
import os from 'node:os';
import http from 'node:http';
import {spawn,spawnSync} from 'node:child_process';
import {createInterface} from 'node:readline';
import {fileURLToPath} from 'node:url';
import {setTimeout as sleep} from 'node:timers/promises';
import * as io from './requests-contract.mjs';
import * as cpu from './cpu-contract.mjs';
import * as uploads from './client-uploads-contract.mjs';
import * as middleware from './middleware-contract.mjs';
import {processSnapshot} from './telemetry.mjs';
import {waitForQuietMachine,DEFAULT_LIMIT} from '../runtime/quiet-machine.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../..');
const flags=process.argv.slice(2);if(flags.some(flag=>!['--verify','--prepared','--cpu','--uploads','--middleware'].includes(flag)&&!flag.startsWith('--engines=')))throw Error('Unknown argument');
const cpuMode=flags.includes('--cpu'),uploadMode=flags.includes('--uploads'),middlewareMode=flags.includes('--middleware'),resourceMode=cpuMode||uploadMode||middlewareMode,engines=io.requestEngines(flags);if([cpuMode,uploadMode,middlewareMode].filter(Boolean).length>1)throw Error('Choose one HTTP workload family');
if(resourceMode&&!flags.includes('--verify')&&process.platform!=='darwin')throw Error('CPU/RSS protocol currently requires macOS ps time/RSS sampling');
const {protocol,workloads,sourceFiles,sourceHashes,assertManifest,cases,rejectionCases,validateResponse,hash}=cpuMode?cpu:uploadMode?uploads:middlewareMode?middleware:io;
const manifest=JSON.parse(fs.readFileSync(path.join(here,cpuMode?'cpu-manifest.json':uploadMode?'client-uploads-manifest.json':middlewareMode?'middleware-manifest.json':'requests-manifest.json'),'utf8'));assertManifest(manifest);
const verify=flags.includes('--verify'),prepared=flags.includes('--prepared');
if(process.version!=='v24.11.1')throw Error('Use Node v24.11.1 for the pinned comparison');
const compiler=process.env.TSRUST??path.join(root,'target/debug/tsrust');
const target=path.resolve(process.env.CARGO_TARGET_DIR??path.join(root,'target')),build=path.join(target,cpuMode?'http-cpu':uploadMode?'http-client-uploads':middlewareMode?'http-middleware':'http-requests'),pkg=path.join(build,'native-package');fs.mkdirSync(build,{recursive:true});
const env={...process.env,CARGO_TARGET_DIR:target,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1',NODE_NO_WARNINGS:'1'};
if(resourceMode)env.TOKIO_WORKER_THREADS=String(protocol.serverWorkers);
for(const key of Object.keys(env))if(key==='RUSTFLAGS'||key==='CARGO_ENCODED_RUSTFLAGS'||key==='RUSTC'||key.startsWith('RUSTC_')||key.startsWith('CARGO_PROFILE_'))delete env[key];
const run=(command,args)=>{const result=spawnSync(command,args,{cwd:root,env,encoding:'utf8',timeout:600000,maxBuffer:32*1024*1024});if(result.error||result.status!==0)throw Error(`${command} failed: ${result.error??result.stderr+result.stdout}`);return result.stdout;};
const commands={'native':[path.join(build,'tsrust-server')],'node-tsc':[process.execPath,path.join(build,cpuMode?'cpu.js':uploadMode?'client-uploads.js':middlewareMode?'middleware.js':'requests.js')],'rust-hand':[path.join(build,'rust-server')]};
const identities={sources:sourceHashes(),compiler:hash(fs.readFileSync(compiler)),contract:hash(fs.readFileSync(path.join(here,uploadMode?'client-uploads-contract.mjs':middlewareMode?'middleware-contract.mjs':'requests-contract.mjs'))),harness:hash(fs.readFileSync(fileURLToPath(import.meta.url))),...(cpuMode?cpu.oracleHashes():{})};
if(!prepared){
 console.log('Build Tsrust, Node-after-tsc and independent Rust servers (untimed)');
 fs.rmSync(pkg,{recursive:true,force:true});run(compiler,['emit',path.join(here,sourceFiles.native),'--package','-o',pkg]);
 const stdout=run('rustup',['run','1.99.1','cargo','build','--manifest-path',path.join(pkg,'Cargo.toml'),'--offline','--locked','--release','--message-format=json']);
 const artifacts=stdout.trim().split('\n').filter(Boolean).map(line=>JSON.parse(line));
 const binary=artifacts.find(row=>row.reason==='compiler-artifact'&&row.executable&&row.target.kind.includes('bin'))?.executable;if(!binary)throw Error('Missing Tsrust executable');fs.copyFileSync(binary,commands.native[0]);
 const names=cpuMode?['axum','tokio','serde','serde_json']:uploadMode?['axum','tokio','reqwest','http_body_util','futures_util','form_urlencoded','base64']:middlewareMode?['axum','tokio','tower_http']:['axum','tokio','sqlx','serde','serde_json'],libraries=new Map();
 for(const row of artifacts)if(row.reason==='compiler-artifact'&&names.includes(row.target.name))for(const file of row.filenames.filter(file=>file.endsWith('.rlib'))){if(libraries.has(row.target.name)&&libraries.get(row.target.name)!==file)throw Error('Ambiguous library');libraries.set(row.target.name,file);}
 const args=['run','1.99.1','rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on','-A','warnings',path.join(here,sourceFiles['rust-hand']),'-o',commands['rust-hand'][0]];
 for(const name of names){const file=libraries.get(name);if(!file)throw Error(`Missing ${name} library`);args.push('--extern',`${name}=${file}`);}
 for(const directory of new Set([...libraries.values()].map(file=>path.dirname(file))))args.push('-L',`dependency=${directory}`);
 run('rustup',args);
 run(process.execPath,[path.join(root,'benchmarks/runtime/node_modules/typescript/bin/tsc'),path.join(here,sourceFiles['node-tsc']),'--target','ES2024','--module','commonjs','--strict','--outDir',build]);
 fs.writeFileSync(path.join(build,'prepared.json'),JSON.stringify(identities));
}else {const saved=JSON.parse(fs.readFileSync(path.join(build,'prepared.json'),'utf8'));if(JSON.stringify(saved.sources)!==JSON.stringify(identities.sources)||saved.compiler!==identities.compiler)throw Error('Prepared server sources/compiler changed; rebuild');for(const [engine,command] of Object.entries(commands))if(!fs.existsSync(engine==='node-tsc'?command[1]:command[0]))throw Error('Prepared executable missing; rebuild');}
const preparedCases=new Map(workloads.map(row=>[row.id,cases(row)]));
const execution=[],observations=[],loads=[];let active;
async function start(command){
 const child=spawn(command[0],command.slice(1),{cwd:root,env,stdio:['ignore','pipe','pipe']});let stderr='';child.stderr.on('data',data=>stderr+=data);
 const completion=new Promise(resolve=>child.once('close',(code,signal)=>resolve({code,signal})));const lines=createInterface({input:child.stdout});
 const ready=await Promise.race([new Promise((resolve,reject)=>{lines.once('line',line=>{try{resolve(JSON.parse(line));}catch(error){reject(error);}});child.once('error',reject);completion.then(result=>reject(Error(`Server exited ${JSON.stringify(result)}: ${stderr}`)));}),sleep(15000,undefined,{ref:false}).then(()=>{throw Error('Server startup timeout')})]).catch(async error=>{child.kill('SIGKILL');await completion;throw error;});
 lines.close();if(!/^127\.0\.0\.1:\d+$/.test(ready.address)||(!resourceMode&&typeof ready.sqliteVersion!=='string')) {child.kill('SIGKILL');await completion;throw Error('Invalid server readiness');}
 return {child,completion,ready,stderr:()=>stderr};
}
async function stop(server){
 if(server.child.exitCode===null&&server.child.signalCode===null)server.child.kill('SIGINT');
 const result=await Promise.race([server.completion,sleep(7000,undefined,{ref:false}).then(()=>null)]);
 if(result===null){server.child.kill('SIGKILL');await server.completion;throw Error('Server failed graceful shutdown');}
 if(result.code!==0)throw Error(`Server exit ${JSON.stringify(result)}: ${server.stderr()}`);
}
function request(agent,address,test,latencies){const started=latencies?performance.now():0;return new Promise((resolve,reject)=>{
 const req=http.request(`http://${address}${test.path}`,{method:test.method,agent,headers:{...(test.body?{'content-type':'application/json','content-length':test.body.length}:{}),...test.headers}},response=>{const chunks=[];response.on('data',chunk=>chunks.push(chunk));response.once('error',reject);response.once('end',()=>{const elapsed=latencies?performance.now()-started:0;try{validateResponse(test,response.statusCode,response.headers,Buffer.concat(chunks));if(latencies)latencies.push(elapsed);resolve();}catch(error){reject(error);}});});
 req.setTimeout(5000,()=>req.destroy(Error('HTTP timeout')));req.once('error',reject);req.end(test.body);
});}
async function batch(agent,address,tests,concurrency=1,latencies){let next=0;await Promise.all(Array.from({length:concurrency},async()=>{while(next<tests.length){const test=tests[next++];await request(agent,address,test,latencies);}}));}
try{
 for(let processIndex=0;processIndex<(verify?1:protocol.processes);processIndex++){
  const offset=processIndex%engines.length;const rotated=[...engines.slice(offset),...engines.slice(0,offset)];
  for(const engine of rotated){
   if(!verify)loads.push(waitForQuietMachine());
   console.log(`Validate ${engine}, process ${processIndex+1}`);active=await start(commands[engine]);
   const agents=new Map(protocol.concurrency.map(concurrency=>[concurrency,new http.Agent({keepAlive:true,maxSockets:concurrency,maxFreeSockets:concurrency})]));
   try{
    for(const workload of workloads)await batch(agents.get(workload.concurrency),active.ready.address,preparedCases.get(workload.id),workload.concurrency);
    for(const test of rejectionCases)await request(agents.get(1),active.ready.address,test);
    observations.push({engine,processIndex,sqliteVersion:active.ready.sqliteVersion,verifiedRequests:workloads.length*protocol.requestsPerBatch+rejectionCases.length});
    if(!verify)for(const workload of workloads){
     const agent=agents.get(workload.concurrency);const tests=preparedCases.get(workload.id);for(let i=0;i<protocol.warmups;i++)await batch(agent,active.ready.address,tests,workload.concurrency);
     const durations=[],latencies=resourceMode?[]:undefined;for(let sample=0;sample<protocol.samples;sample++){
      const load=os.loadavg()[0];if(load>=DEFAULT_LIMIT)throw Error(`Machine became busy (${load}); no accepted result`);loads.push(load);
      const start=performance.now();await batch(agent,active.ready.address,tests,workload.concurrency,latencies);durations.push(performance.now()-start);
     }
     let resources;
     if(resourceMode){
      const before=processSnapshot(active.child.pid);let peak=before.rssBytes,rssSamples=1;
      const timer=setInterval(()=>{const value=processSnapshot(active.child.pid);peak=Math.max(peak,value.rssBytes);rssSamples++;},protocol.rssSampleMs);
      try{for(let i=0;i<protocol.resourceBatches;i++)await batch(agent,active.ready.address,tests,workload.concurrency);}
      finally{clearInterval(timer);}
      const after=processSnapshot(active.child.pid);peak=Math.max(peak,after.rssBytes);rssSamples++;
      if(after.cpuMs<before.cpuMs)throw Error('Process CPU time regressed');
      resources={cpuMs:after.cpuMs-before.cpuMs,requests:protocol.resourceBatches*protocol.requestsPerBatch,peakRssBytes:peak,rssSamples};
     }
     execution.push({workload:workload.id,engine,processIndex,durations,...(resourceMode?{latencies,resources}:{})});console.log(`Measured ${workload.id}/${engine}, process ${processIndex+1}`);
    }
   }finally{for(const agent of agents.values())agent.destroy();await stop(active);active=undefined;}
  }
 }
 assertManifest(manifest);
 const result={schemaVersion:1,measuredEngines:engines,timestamp:new Date().toISOString(),verifiedOnly:verify,measurementScope:(resourceMode?'One server worker; request latency; separate CPU/RSS pass. ':'')+(uploadMode?'validated keep-alive loopback HTTP at 1/16 connections; upload buffer generation/fill and backend validation INCLUDED; excludes build/startup and outer common-client fixture preparation':middlewareMode?'validated fixed-corpus keep-alive middleware HTTP; payload generation and backend middleware/encoding INCLUDED; batch elapsed includes common-client exact validation/decompression; request latency ends at complete encoded body before validation; separate server CPU/RSS; excludes build/startup/outer fixtures':'validated keep-alive loopback HTTP at 1/16 connections; common Node client; excludes build, startup, data seeding and payload preparation'),semanticEpoch:manifest.semanticEpoch,manifest:{workloads:workloads.map(row=>({...row,iterations:128,operationsPerIteration:1}))},protocol,versions:{node:process.version,rust:run('rustup',['run','1.99.1','rustc','--version']).trim(),typescript:run(process.execPath,[path.join(root,'benchmarks/runtime/node_modules/typescript/bin/tsc'),'--version']).trim()},machine:{cpu:os.cpus()[0].model,platform:os.platform(),architecture:os.arch()},quietMachine:{allowBusy:false,loadLimit:DEFAULT_LIMIT,maxOneMinuteLoad:loads.length?Math.max(...loads):null},identities,observations,execution};
 const file=path.join(root,'benchmarks/runtime/results',`${result.timestamp.replace(/[:.]/g,'-')}${verify?'-verify':''}-http-${cpuMode?'cpu':uploadMode?'client-uploads':middlewareMode?'middleware':'requests'}.json`);fs.writeFileSync(file,JSON.stringify(result,null,2)+'\n');console.log(`Saved ${file}`);
}finally{if(active)await stop(active);}
