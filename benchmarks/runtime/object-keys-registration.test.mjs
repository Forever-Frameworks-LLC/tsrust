import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertObjectKeysSource} from './object-keys-source-contract.mjs';
import {objectKeysIds,expectedObjectKeysObservation,expectedObjectKeysValues} from './object-keys-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='object-keys');
test('Object.keys pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),objectKeysIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-object-functions-milestone');assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertObjectKeysSource(source,row,kind);
   assert.throws(()=>assertObjectKeysSource(replaceCodeSnippet(source, 'Object.keys(source)','Object.values(source)'),row,kind));
   assert.throws(()=>assertObjectKeysSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertObjectKeysSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertObjectKeysSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertObjectKeysSource\(source/);assert.ok(source.includes("'object-keys-oracle.mjs'"));}
});
test('Object.keys independent formula retains every key and routes exact observations',()=>{
 assert.deepEqual(expectedObjectKeysValues('object_keys_order',2,0,'é😀'),[['gamma|alpha|beta','gamma|alpha|beta','changed|alpha|beta','0|1|2','0'],['beta|gamma|alpha','beta|gamma|alpha','changed|gamma|alpha','0|1|2','13']]);
 assert.deepEqual(expectedObjectKeysValues('object_keys_order',0,0,''),[]);assert.throws(()=>expectedObjectKeysValues('object_keys_order',2049,0,''));
 for(const row of rows){const want=observeSliceTree(expectedObjectKeysValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
