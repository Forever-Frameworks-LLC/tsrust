import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertStringAtSource} from './string-at-source-contract.mjs';
import {stringAtIds,expectedStringAtObservation,expectedStringAtValues} from './string-at-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='string-at');
const original=JSON.parse(read('../../archive/string-at-start/benchmarks-runtime/manifest.json'));
test('string at pair appends to the manifest and binds actual guarded sources',()=>{
 assert.deepEqual(manifest.workloads.slice(0,original.workloads.length),original.workloads);
 assert.deepEqual(rows.map(row=>row.id),stringAtIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-string-methods-milestone');assert.equal(reportCategory(row.id),'strings');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertStringAtSource(source,row,kind);
   assert.throws(()=>assertStringAtSource(source.replace('.at(','.charAt('),row,kind));
   assert.throws(()=>assertStringAtSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertStringAtSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertStringAtSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertStringAtSource\(source/);assert.ok(source.includes("'string-at-oracle.mjs'"));}
});
test('string at independent results match hand-written cells and oracle routing',()=>{
 // Written by hand from the workload specification.
 const cells=[
  ['string_at_characters',3,0,'aé🙂',[['none','none'],['é','é'],['🙂','a']]],
  ['string_at_characters',2,1,'ab',[['a','a'],['b','b']]],
 ];
 for(const [id,n,seed,input,values] of cells){
  assert.deepEqual(expectedStringAtValues(id,n,seed,input),values);
  const want=observeSliceTree(values);
  assert.deepEqual(expectedStringAtObservation(id,n,seed,input),want);assert.deepEqual(expectedExactResult(id,n,seed,input),want);assert.equal(expectedChecksum(id,n,seed,input),want.checksum);
 }
 for(const row of rows){const want=expectedStringAtObservation(row.id,2048,7,row.inputText);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
