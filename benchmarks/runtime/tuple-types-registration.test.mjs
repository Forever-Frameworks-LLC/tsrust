import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertTupleTypesSource} from './tuple-types-source-contract.mjs';
import {tupleTypesIds,expectedTupleTypesObservation,expectedTupleTypesValues} from './tuple-types-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='tuple-types');
test('tuple types pair appends to manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),tupleTypesIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-array-methods-milestone');assert.equal(reportCategory(row.id),'arrays');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertTupleTypesSource(source,row,kind);
   assert.throws(()=>assertTupleTypesSource(source.replace('pair[0]=base+1;','pair[0]=base;'),row,kind));
   assert.throws(()=>assertTupleTypesSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertTupleTypesSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertTupleTypesSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertTupleTypesSource\(source/);assert.ok(source.includes("'tuple-types-oracle.mjs'"));}
});
test('tuple types independent results preserve originals, copies and mapped positions',()=>{
 assert.deepEqual(expectedTupleTypesValues('tuple_types_values',2,0,'é😀'),[['1','é😀','0','é😀','1','é😀','2','true'],['14','é😀','13','é😀é😀','14','é😀é😀','2','true']]);
 assert.deepEqual(expectedTupleTypesValues('tuple_types_values',0,0,''),[]);
 assert.throws(()=>expectedTupleTypesValues('tuple_types_values',2049,0,''));
 for(const row of rows){const want=observeSliceTree(expectedTupleTypesValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedTupleTypesObservation(row.id,2048,7,row.inputText),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
