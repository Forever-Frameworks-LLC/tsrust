// Source-only admission preparation until an exclusive build/timing lease.
// Existing rlibs are inputs; this driver never invokes Cargo or fetches crates.
import fs from 'node:fs';
import path from 'node:path';
import {spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import {createHash} from 'node:crypto';
import {waitForQuietMachine} from '../../quiet-machine.mjs';
import {validateContracts,validateRecords} from './profile-oracle.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../../../..');
const target=process.env.CARGO_TARGET_DIR;
if(!target||!path.isAbsolute(target))throw new Error('Pass the coordinated absolute shared CARGO_TARGET_DIR.');
if(spawnSync(process.execPath,['--version'],{encoding:'utf8'}).stdout.trim()!=='v24.11.1')throw new Error('Use pinned Node24.11.1.');
const metadataPath=process.argv.find(value=>value.startsWith('--artifacts='))?.slice(12);
if(!metadataPath)throw new Error('Pass --artifacts=<exact approved CargoJSON path>.');
const metadata=fs.readFileSync(metadataPath,'utf8').trim().split('\n').filter(line=>line.startsWith('{')).map(line=>JSON.parse(line));
const dependencies={};
for(const [name,version] of [['tokio','1.53.3'],['tokio_util','0.7.20'],['rayon','1.11.0']]){
    const matches=metadata.filter(value=>value.reason==='compiler-artifact'&&value.target.name===name&&value.filenames.some(file=>file.endsWith('.rlib')));
    if(matches.length!==1||!matches[0].package_id.endsWith(`@${version}`))throw new Error(`Ambiguous or wrong exact ${name} artifact.`);
    const value=matches[0],files=value.filenames.filter(file=>file.endsWith('.rlib'));
    if(files.length!==1||!path.isAbsolute(files[0])||!fs.statSync(files[0]).isFile())throw new Error(`Missing exact ${name} rlib.`);
    dependencies[name]={version,path:files[0],features:value.features,profile:value.profile,sha256:createHash('sha256').update(fs.readFileSync(files[0])).digest('hex')};
}
const compilerVersion=fs.readFileSync(path.join(root,'crates/tsrust-compiler/src/lib.rs'),'utf8').match(/pub const RUST_BACKEND_VERSION: &str = "([^"]+)"/)[1];
const scratch=fs.mkdtempSync(path.join(target,'task-timer-channel-profile-'));
const run=(command,args,env={})=>{const result=spawnSync(command,args,{cwd:root,encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024,env:{...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1',...env}});if(result.error||result.status!==0)throw new Error(`${command} failed: ${result.error??result.stderr}`);return result.stdout;};
try{
    let runtime='mod tsrust_native_threads {\n'+['runtime_core.rs','runtime_sync.rs'].map(file=>fs.readFileSync(path.join(root,'crates/tsrust-compiler/src/stdlib/threads',file),'utf8')).join('\n')+'}\n';
    for(const [before,after] of [
        ['pool: Option<Arc<Pool>> }','pool: Option<Arc<Pool>>, probe_done: Mutex<Option<tokio::sync::oneshot::Sender<()>>> }'],
        ['released: AtomicBool::new(false), pool })','released: AtomicBool::new(false), pool, probe_done: Mutex::new(None) })'],
        ['completion.ready.notify_all();','completion.ready.notify_all(); if let Some(done)=completion.probe_done.lock().unwrap().take(){let _=done.send(());}'],
    ]){if(runtime.split(before).length!==2)throw new Error(`Prototype anchor changed: ${before}`);runtime=runtime.replace(before,after);}
    const end=runtime.lastIndexOf('}');runtime=runtime.slice(0,end)+fs.readFileSync(path.join(here,'pool_transport.rs'),'utf8')+runtime.slice(end);
    const support=path.join(scratch,'pool.rs');fs.writeFileSync(support,runtime);
    const env={TSRUST_B06_THREADS_SOURCE:support,TSRUST_B06_FUTURES_SOURCE:path.join(root,'crates/tsrust-compiler/src/stdlib/futures/runtime.rs')};
    const builds=[],binaries={};
    for(const candidate of ['strong','weak'])for(const instrumented of [false,true]){
        const binary=path.join(scratch,`${candidate}${instrumented?'-alloc':''}${process.platform==='win32'?'.exe':''}`);
        const args=['run',compilerVersion,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on','--cfg','profile_probe'];
        if(candidate==='weak')args.push('--cfg','profile_weak');if(instrumented)args.push('--cfg','allocation_probe');
        for(const [name,value]of Object.entries(dependencies))args.push('--extern',`${name}=${value.path}`);
        for(const directory of new Set(Object.values(dependencies).map(value=>path.dirname(value.path))))args.push('-L',`dependency=${directory}`);
        args.push(path.join(here,'profile.rs'),'-o',binary);const started=performance.now();run('rustup',args,env);
        builds.push({candidate,instrumented,buildMs:performance.now()-started,binaryBytes:fs.statSync(binary).size});binaries[`${candidate}:${instrumented}`]=binary;
    }
    const contracts=[];
    for(const candidate of ['strong','weak']){const value=JSON.parse(run(binaries[`${candidate}:false`],['--check']));validateContracts(value,candidate);contracts.push(value);}
    const allocations=[],measurements=[],quietLoads=[];
    for(let processIndex=0;processIndex<3;processIndex++)for(const candidate of processIndex%2?['weak','strong']:['strong','weak']){
        const args=candidate==='weak'?['--capability-only']:[];
        const counts=run(binaries[`${candidate}:true`],args).trim().split('\n').map(line=>JSON.parse(line));validateRecords(counts,candidate,true);allocations.push({processIndex,candidate,records:counts});
        quietLoads.push(waitForQuietMachine({log:line=>process.stderr.write(`${line}\n`)}));
        const timing=run(binaries[`${candidate}:false`],args).trim().split('\n').map(line=>JSON.parse(line));validateRecords(timing,candidate,false);measurements.push({processIndex,candidate,records:timing});
    }
    const sources=Object.fromEntries(['profile.rs','profile.mjs','profile-oracle.mjs','reference.rs','pool_transport.rs','../../probes/counting-allocator.rs','../../quiet-machine.mjs','../../../../crates/tsrust-compiler/src/stdlib/threads/runtime.rs','../../../../crates/tsrust-compiler/src/stdlib/threads/runtime_core.rs','../../../../crates/tsrust-compiler/src/stdlib/threads/runtime_sync.rs','../../../../crates/tsrust-compiler/src/stdlib/futures/runtime.rs'].map(file=>[file,createHash('sha256').update(fs.readFileSync(path.resolve(here,file))).digest('hex')]));
    console.log(JSON.stringify({scope:'admission prototype; not generated facade performance',protocol:{processes:3,warmups:8,samples:31,separateAllocationBuild:true},compilerVersion,dependencies,sources,builds,contracts,quietLoads,allocations,measurements}));
}finally{fs.rmSync(scratch,{recursive:true,force:true});}
