import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {monotonicTimeID,expectedMonotonicTimeObservation,assertMonotonicTimeSource} from './monotonic-time-contract.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
const read=name=>fs.readFileSync(new URL(name,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(w=>w.id===monotonicTimeID);
test('clock workload retains every contract observation and independent oracle',()=>{
 assert.equal(reportCategory(row.id),'time');assert.equal(row.measurementStatus,'pending-s09-family');
 assert.equal(expectedMonotonicTimeObservation(3,0).exactBits,'t:[b1,b1,b1]');
 for(const n of [0,1,3,137,20000])for(const seed of [0,7,1000000]){
  const expected=expectedMonotonicTimeObservation(n,seed);assert.deepEqual(expectedExactResult(row.id,n,seed),expected);assert.equal(expectedChecksum(row.id,n,seed),expected.checksum);
 }
 for(const [n,seed] of [[-1,0],[20001,0],[1,-1]])assert.throws(()=>expectedMonotonicTimeObservation(n,seed));
});
test('clock workload guards reject weakened observations and skipped reads',()=>{
 for(const [kind,file] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+file);assertMonotonicTimeSource(source,row,kind);
  for(const [from,to] of [['i = i + 1;','i = i + 2;'],['values.push(Number.isFinite(current) && current >= previous && current >= 0);','values.push(true);'],[kind==='native'?'const current = monotonicNow();':'const current = clock.now();','const current = previous;']]){
   const changed=replaceCodeSnippet(source,from,to);const key=kind==='native'?'sourceSha256':'counterpartSha256';
   assert.throws(()=>assertMonotonicTimeSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert(source.includes('assertMonotonicTimeSource(source'));assert(source.includes("'monotonic-time-contract.mjs'"));}
});
