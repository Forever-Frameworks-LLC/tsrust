import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertDestructuringRestSource} from './destructuring-rest-source-contract.mjs';
import {destructuringRestIds,expectedDestructuringRestValues} from './destructuring-rest-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='destructuring-rest');
const original=JSON.parse(read('../../archive/destructuring-rest-benchmark-start/benchmarks/runtime/manifest.json'));
test('rest/spread registration binds complete scenarios and preserves prior manifest rows',()=>{
 assert.deepEqual(manifest.workloads.slice(0,original.workloads.length),original.workloads);
 assert.deepEqual(rows.map(row=>row.id),destructuringRestIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertDestructuringRestSource(source,row,kind);
   for(const changed of [source.replace('rest.push("tail")','rest.push("other")'),source.replace('return output;','return [];')]){
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertDestructuringRestSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertDestructuringRestSource(source,{...row,operationsPerIteration:4},kind));
   assert.throws(()=>assertDestructuringRestSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertDestructuringRestSource\(source/);assert.ok(source.includes("'destructuring-rest-oracle.mjs'"));}
});
test('independent rest/spread oracle retains all copied values and constructor key order',()=>{
 const expected=[['ab','0|abab|tail','ab','ab','0','active,count','ab','1','before,active,name,count,after','3','3'],['ab','13|abab|tail','ab','ab','13','active,count','ab','14','before,active,name,count,after','3','3']];
 assert.deepEqual(expectedDestructuringRestValues('destructuring_rest',2,0,'ab'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('destructuring_rest',2,0,'ab'),observation);
 assert.equal(expectedChecksum('destructuring_rest',2,0,'ab'),observation.checksum);
 assert.deepEqual(expectedDestructuringRestValues('destructuring_rest',0,0,''),[]);
});
