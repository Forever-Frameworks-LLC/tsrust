// No Cargo/fetch. Root supplies exact admitted graph artifacts and the heavy slot.
import fs from 'node:fs';import os from 'node:os';import path from 'node:path';import {spawnSync} from 'node:child_process';import {fileURLToPath} from 'node:url';import {createHash} from 'node:crypto';
import {waitForQuietMachine} from '../../quiet-machine.mjs';import {validateContracts,validateProfile,requireOptimizedArtifacts,requireFinishedBuild,sourceHashes,assertSourceHashes} from './oracle.mjs';
const here=path.dirname(fileURLToPath(import.meta.url)),root=path.resolve(here,'../../../..');
const option=name=>process.argv.find(arg=>arg.startsWith(`--${name}=`))?.slice(name.length+3);
const metadataPath=option('artifacts'),toolchain=option('toolchain');if(!metadataPath||!toolchain)throw Error('Pass --artifacts=<exact CargoJSON> --toolchain=<pinned Rust>.');
if(process.version!=='v24.11.1')throw Error('Use pinned Node24.11.1.');
const target=process.env.CARGO_TARGET_DIR;if(!target||!path.isAbsolute(target))throw Error('Pass coordinated absolute CARGO_TARGET_DIR.');
const rows=fs.readFileSync(metadataPath,'utf8').split('\n').filter(line=>line.startsWith('{')).map(line=>JSON.parse(line)),dependencies={};
requireFinishedBuild(rows);
for(const[name,version]of[['tokio','1.53.3'],['bytes','1.12.1']]){
 const matches=rows.filter(row=>row.reason==='compiler-artifact'&&row.target.name===name&&row.filenames.some(file=>file.endsWith('.rlib')));
 if(matches.length!==1||!matches[0].package_id.endsWith(`@${version}`))throw Error(`Missing/ambiguous exact ${name} artifact`);
 const row=matches[0],files=row.filenames.filter(file=>file.endsWith('.rlib'));if(files.length!==1||!path.isAbsolute(files[0])||!fs.statSync(files[0]).isFile())throw Error(`Missing ${name} rlib`);
 if(name==='tokio')for(const feature of ['rt-multi-thread','time','sync','fs','io-util'])if(!row.features.includes(feature))throw Error(`Tokio feature prerequisite: ${feature}`);
 dependencies[name]={path:files[0],version,profile:row.profile,features:row.features,sha256:createHash('sha256').update(fs.readFileSync(files[0])).digest('hex')};
}
requireOptimizedArtifacts(dependencies,process.argv.includes('--profile'));
const sourceFiles=['owned_read.rs','frame_cursor.rs','reference.rs','profile.rs','oracle.mjs','oracle.test.mjs','admission.mjs','borrowed_destination_rejected.rs','readonly_destination_rejected.rs','reader_reuse_rejected.rs','../../probes/counting-allocator.rs','../../quiet-machine.mjs','../../../../crates/tsrust-compiler/src/stdlib/futures/runtime.rs'];
const sources=sourceHashes(here,sourceFiles);
const temporary=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-async-io-'));
const env={...process.env,CARGO_INCREMENTAL:'0',CARGO_BUILD_JOBS:'1',TSRUST_IO_FUTURES_SOURCE:path.join(root,'crates/tsrust-compiler/src/stdlib/futures/runtime.rs')};
const run=(command,args,success=true)=>{const result=spawnSync(command,args,{cwd:root,env,encoding:'utf8',timeout:180000,maxBuffer:8*1024*1024});if(result.error||success&&result.status!==0)throw Error(`${command}: ${result.error??result.stderr}`);return result;};
const rustArgs=()=>['run',toolchain,'rustc','--edition=2024','-C','opt-level=3','-C','overflow-checks=on',...Object.entries(dependencies).flatMap(([name,value])=>['--extern',`${name}=${value.path}`]),...Array.from(new Set(Object.values(dependencies).map(value=>path.dirname(value.path)))).flatMap(directory=>['-L',`dependency=${directory}`])];
try{
 const binary=path.join(temporary,'contracts');run('rustup',[...rustArgs(),path.join(here,'reference.rs'),'-o',binary]);const contracts=JSON.parse(run(binary,[]).stdout);validateContracts(contracts);
 const negative=[];for(const[file,codes]of[['borrowed_destination_rejected.rs',['E0506','E0499','E0503']],['readonly_destination_rejected.rs',['E0308']],['reader_reuse_rejected.rs',['E0499']]]){const result=run('rustup',[...rustArgs(),path.join(here,file),'-o',path.join(temporary,'negative')],false);if(result.status===0||!codes.some(code=>result.stderr.includes(code)))throw Error(`Negative lifetime witness did not fail as required: ${file}\n${result.stderr}`);negative.push({file,matched:codes.filter(code=>result.stderr.includes(code))});}
 const profiles=[],quietLoads=[];
 if(process.argv.includes('--profile')){
  const binaries={};for(const instrumented of [false,true]){const output=path.join(temporary,`profile-${instrumented}`);run('rustup',[...rustArgs(),'--cfg','profile_probe',...(instrumented?['--cfg','allocation_probe']:[]),path.join(here,'profile.rs'),'-o',output]);binaries[instrumented]=output;}
  for(let processIndex=0;processIndex<3;processIndex++)for(const reads of [64,1024,8192])for(const candidate of processIndex%2?['tokio','staged']:['staged','tokio']){
   const payload=path.join(temporary,'payload');fs.writeFileSync(payload,Buffer.from(Array.from({length:reads*64},(_,index)=>index%251)));
   const allocation=JSON.parse(run(binaries[true],[candidate,payload,String(reads)]).stdout);validateProfile(allocation,true);
   quietLoads.push(waitForQuietMachine({log:line=>process.stderr.write(line+'\n')}));const timing=JSON.parse(run(binaries[false],[candidate,payload,String(reads)]).stdout);validateProfile(timing,false);profiles.push({processIndex,allocation,timing});
  }
 }
 assertSourceHashes(sources,sourceHashes(here,sourceFiles));
 console.log(JSON.stringify({scope:'B05 direct Rust admission; no public compiler facade acceptance',toolchain,dependencies,sources,contracts,negative,protocol:{processes:3,warmups:8,samples:31,separateAllocationBuild:true,setupExcludedFromTiming:true,teardownExcludedFromTiming:true,setupIncludedInAllocation:true,teardownIncludedInAllocation:true},profiles,quietLoads}));
}finally{fs.rmSync(temporary,{recursive:true,force:true});}
