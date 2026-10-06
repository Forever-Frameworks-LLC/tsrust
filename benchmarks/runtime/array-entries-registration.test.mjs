import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertArrayEntriesSource} from './array-entries-source-contract.mjs';
import {arrayEntriesIds,expectedArrayEntriesObservation,expectedArrayEntriesValues} from './array-entries-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='array-entries');
test('array entries pairs append and bind actual native and explicit copied counterpart sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),arrayEntriesIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-array-methods-milestone');assert.equal(reportCategory(row.id),'arrays');
  for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+path);assertArrayEntriesSource(source,row,kind);
   assert.throws(()=>assertArrayEntriesSource(source.replace('source.entries()','source.values()'),row,kind));
   assert.throws(()=>assertArrayEntriesSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertArrayEntriesSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertArrayEntriesSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertArrayEntriesSource\(source/);assert.ok(source.includes("'array-entries-oracle.mjs'"));}
});
test('array entries direct oracle preserves original owners after pair mutation and routes complete observations',()=>{
 assert.deepEqual(expectedArrayEntriesValues('array_entries_string',2,0,'é😀'),[['5','é😀','é😀','1','é😀','2','2','true'],['18','é😀','é😀é😀','1','é😀','2','2','true']]);
 assert.deepEqual(expectedArrayEntriesValues('array_entries_records',1,7,'x'),[['0','1','xx','7,8','x','16,8','2']]);
 assert.deepEqual(expectedArrayEntriesValues('array_entries_string',0,0,''),[]);assert.throws(()=>expectedArrayEntriesValues('array_entries_string',2049,0,''));
 for(const row of rows){const want=observeSliceTree(expectedArrayEntriesValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedArrayEntriesObservation(row.id,2048,7,row.inputText),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
