import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertArrayFromSource} from './array-from-source-contract.mjs';
import {arrayFromIds,expectedArrayFromObservation,expectedArrayFromValues} from './array-from-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='array-from');
test('Array.from pairs append to the manifest and bind actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),arrayFromIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-array-methods-milestone');assert.equal(reportCategory(row.id),'arrays');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertArrayFromSource(source,row,kind);
   assert.throws(()=>assertArrayFromSource(source.replace('Array.from(','copy('),row,kind));
   assert.throws(()=>assertArrayFromSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertArrayFromSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertArrayFromSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertArrayFromSource\(source/);assert.ok(source.includes("'array-from-oracle.mjs'"));}
});
test('Array.from independent results match hand-written cells and oracle routing',()=>{
 // Written by hand from the workload specification.
 const cells=[
  ['array_from_copy_int',3,0,'ab',[[],[13],[26,27,28,29,30]]],
  ['array_from_copy_int',2,1,'ab',[[1],[14,15,16,17,18]]],
  ['array_from_map_int',3,0,'ab',[[],[26],[52,55,58,61,64]]],
  ['array_from_string_characters',3,0,'aé🙂',[[],['a','é','🙂'],['a','é','🙂','a','é','🙂']]],
  ['array_from_map_string',3,0,'aé',[[],['a0','é1'],['a0','é1','a2','é3']]],
 ];
 for(const [id,n,seed,input,values] of cells){
  assert.deepEqual(expectedArrayFromValues(id,n,seed,input),values);
  const want=observeSliceTree(values);
  assert.deepEqual(expectedArrayFromObservation(id,n,seed,input),want);assert.deepEqual(expectedExactResult(id,n,seed,input),want);assert.equal(expectedChecksum(id,n,seed,input),want.checksum);
 }
 for(const row of rows){const want=expectedArrayFromObservation(row.id,2048,7,row.inputText);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
