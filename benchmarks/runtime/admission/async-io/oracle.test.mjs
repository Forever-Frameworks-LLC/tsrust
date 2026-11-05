import fs from 'node:fs';import os from 'node:os';import path from 'node:path';
import test from 'node:test';import assert from 'node:assert/strict';import {validateContracts,expectedChecksum,validateProfile,requireOptimizedArtifacts,requireFinishedBuild,sourceHashes,assertSourceHashes} from './oracle.mjs';
test('async I/O oracle rejects lost partial bytes and premature settlement',()=>{
 const expected={zero:0,partial:[1,2,3,99,99],filled:3,cancelledDestination:[90,4,5,6,7,90],recovered:4,blockingCompletions:1,frameTail:[10,11],borrowedIdentity:true,drainWaited:true,deadlineWaited:true,partialError:[14,15,99,99],errorFilled:2};validateContracts(expected);
 for(const altered of [{...expected,partial:[1,2,3,0,0]},{...expected,blockingCompletions:2},{...expected,drainWaited:false},{...expected,deadlineWaited:false},{...expected,errorFilled:0}])assert.throws(()=>validateContracts(altered));
 assert.equal(expectedChecksum(0),0);const value=Array.from({length:64},(_,index)=>index).reduce((sum,value)=>(sum*131+value)%1000000007,0);assert.equal(expectedChecksum(1),value);
 const row={candidate:'staged',reads:64,checksum:expectedChecksum(64),reusedDestination:true,durations:Array(31).fill(0)};validateProfile(row,false);assert.throws(()=>validateProfile({...row,checksum:row.checksum+1},false));assert.throws(()=>validateProfile({...row,reusedDestination:false},false));
});

test('profiling rejects either unoptimized artifact while correctness allows debug',()=>{
 const optimized={tokio:{profile:{opt_level:'3'}},bytes:{profile:{opt_level:'3'}}};requireOptimizedArtifacts(optimized,true);
 for(const name of ['tokio','bytes']){
  const changed={...optimized,[name]:{profile:{opt_level:'0'}}};assert.throws(()=>requireOptimizedArtifacts(changed,true),new RegExp(name));requireOptimizedArtifacts(changed,false);
  assert.throws(()=>requireOptimizedArtifacts({...optimized,[name]:{}},true),new RegExp(name));
 }
});

test('artifact selection requires a single successful terminal CargoJSON result',()=>{
 const artifact={reason:'compiler-artifact'};requireFinishedBuild([artifact,{reason:'build-finished',success:true}]);
 for(const rows of [[],[artifact],[artifact,{reason:'build-finished',success:false}],[{reason:'build-finished',success:true},artifact],[{reason:'build-finished',success:true},{reason:'build-finished',success:true}]])assert.throws(()=>requireFinishedBuild(rows));
});
test('source hashes are captured before work and reject changed inputs',()=>{
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-io-source-guard-'));
 try{const file=path.join(directory,'input.rs');fs.writeFileSync(file,'before');const before=sourceHashes(directory,['input.rs']);assertSourceHashes(before,sourceHashes(directory,['input.rs']));fs.writeFileSync(file,'after');assert.throws(()=>assertSourceHashes(before,sourceHashes(directory,['input.rs'])));}finally{fs.rmSync(directory,{recursive:true,force:true});}
});

test('allocation retention records signed live-byte delta after owner teardown',()=>{
 const row={candidate:'staged',reads:64,checksum:expectedChecksum(64),reusedDestination:true,durations:[],allocations:1,reallocations:0,requested:64,peak:64};
 for(const retained of [-64,0,64])validateProfile({...row,retained},true);
 for(const retained of [undefined,0.5,Number.NaN,Number.POSITIVE_INFINITY])assert.throws(()=>validateProfile({...row,retained},true));
});
