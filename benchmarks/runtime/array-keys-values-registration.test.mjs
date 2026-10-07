import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertArrayKeysValuesSource} from './array-keys-values-source-contract.mjs';
import {arrayKeysValuesIds,expectedArrayKeysValuesObservation,expectedArrayKeysValuesValues} from './array-keys-values-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='array-keys-values');
test('array keys/values pairs append to the manifest and bind actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),arrayKeysValuesIds);
 for(const row of rows){
  const method=row.id==='array_keys_int'?'keys':'values';
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-array-methods-milestone');assert.equal(reportCategory(row.id),'arrays');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertArrayKeysValuesSource(source,row,kind);
   assert.throws(()=>assertArrayKeysValuesSource(replaceCodeSnippet(source, 'values.'+method+'()','values'),row,kind));
   assert.throws(()=>assertArrayKeysValuesSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertArrayKeysValuesSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertArrayKeysValuesSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertArrayKeysValuesSource\(source/);assert.ok(source.includes("'array-keys-values-oracle.mjs'"));}
});
test('array keys/values independent results match hand-written cells and oracle routing',()=>{
 // Written by hand from the workload specification.
 const cells=[
  ['array_keys_int',3,0,[[],[0],[0,1,2,3,4]]],
  ['array_keys_int',2,1,[[0],[0,1,2,3,4]]],
  ['array_values_int',3,0,[[],[13],[26,27,28,29,30]]],
  ['array_values_int',2,1,[[1],[14,15,16,17,18]]],
 ];
 for(const [id,n,seed,values] of cells){
  assert.deepEqual(expectedArrayKeysValuesValues(id,n,seed),values);
  const want=observeSliceTree(values);
  assert.deepEqual(expectedArrayKeysValuesObservation(id,n,seed),want);assert.deepEqual(expectedExactResult(id,n,seed,'x'),want);assert.equal(expectedChecksum(id,n,seed,'x'),want.checksum);
 }
 for(const row of rows){const want=expectedArrayKeysValuesObservation(row.id,2048,7);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
