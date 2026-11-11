// Independent fixed corpus and byte-by-byte observation; no runtime helper imports.
import assert from 'node:assert/strict';import fs from 'node:fs';import {pathToFileURL} from 'node:url';import path from 'node:path';import {createHash} from 'node:crypto';
export function requireFinishedBuild(rows){
 assert.equal(rows.filter(row=>row.reason==='build-finished').length,1,'Require one terminal CargoJSON build result');
 assert.equal(rows.at(-1)?.reason,'build-finished','CargoJSON artifact listing is incomplete without terminal build-finished');
 assert.equal(rows.at(-1)?.success,true,'Cargo build did not finish successfully');
}
export function sourceHashes(directory,files){return Object.fromEntries(files.map(file=>[file,createHash('sha256').update(fs.readFileSync(path.resolve(directory,file))).digest('hex')]));}
export function assertSourceHashes(before,after){assert.deepEqual(after,before,'Admission inputs changed during execution; discard these results');}
export function requireOptimizedArtifacts(dependencies,profiling){
 if(!profiling)return;
 for(const name of ['tokio','bytes'])assert.equal(dependencies[name]?.profile?.opt_level,'3',`${name}: profiling requires exact CargoJSON opt_level=3; refresh release artifacts`);
}
export function validateContracts(actual){
 assert.deepEqual(actual,{zero:0,partial:[1,2,3,99,99],filled:3,cancelledDestination:[90,4,5,6,7,90],recovered:4,blockingCompletions:1,frameTail:[10,11],borrowedIdentity:true,drainWaited:true,deadlineWaited:true,partialError:[14,15,99,99],errorFilled:2});
}
export function expectedChecksum(reads){let sum=0n;for(let index=0n;index<BigInt(reads)*64n;index++)sum=(sum*131n+index%251n)%1000000007n;return Number(sum);}
export function validateProfile(row,instrumented){
 assert.ok(['staged','tokio'].includes(row.candidate));assert.ok([64,1024,8192].includes(row.reads));assert.equal(row.checksum,expectedChecksum(row.reads));assert.equal(row.reusedDestination,true);
 if(instrumented){for(const field of ['allocations','reallocations','requested','peak'])assert.ok(Number.isSafeInteger(row[field])&&row[field]>=0);assert.ok(Number.isSafeInteger(row.retained),'retained must be an actual signed live-byte delta');assert.deepEqual(row.durations,[]);}
 else{assert.equal(row.durations.length,31);for(const duration of row.durations)assert.ok(Number.isFinite(duration)&&duration>=0);}
}
if(process.argv[1]&&import.meta.url===pathToFileURL(process.argv[1]).href&&process.argv[2])validateContracts(JSON.parse(fs.readFileSync(process.argv[2],'utf8')));
