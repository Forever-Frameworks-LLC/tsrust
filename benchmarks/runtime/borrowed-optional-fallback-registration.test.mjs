import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertBorrowedOptionalFallbackSource} from './borrowed-optional-fallback-source-contract.mjs';
import {borrowedOptionalFallbackIds,expectedBorrowedOptionalFallbackObservation,expectedBorrowedOptionalFallbackValues} from './borrowed-optional-fallback-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='borrowed-optional-fallback');
const original=JSON.parse(read('../../archive/borrowed-optional-fallback-start/benchmarks/runtime/manifest.json'));
test('Borrowed optional fallback pair appends to the manifest and binds actual guarded sources',()=>{
 assert.deepEqual(manifest.workloads.slice(0,original.workloads.length),original.workloads);
 assert.deepEqual(rows.map(row=>row.id),borrowedOptionalFallbackIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-value-types-milestone');assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertBorrowedOptionalFallbackSource(source,row,kind);
   assert.throws(()=>assertBorrowedOptionalFallbackSource(source.replace('label.slice()','source'),row,kind));
   assert.throws(()=>assertBorrowedOptionalFallbackSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertBorrowedOptionalFallbackSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertBorrowedOptionalFallbackSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertBorrowedOptionalFallbackSource\(source/);assert.ok(source.includes("'borrowed-optional-fallback-oracle.mjs'"));}
});
test('borrowed fallback oracle independently retains all three selections',()=>{
 const id='borrowed_optional_fallback';const expected=[['AB','Ab','0'],['NONE','none','0'],['','','0']];
 assert.deepEqual(expectedBorrowedOptionalFallbackValues(id,1,0,'Ab'),expected);
 assert.deepEqual(expectedExactResult(id,1,0,'Ab'),expectedBorrowedOptionalFallbackObservation(id,1,0,'Ab'));
 assert.deepEqual(expectedChecksum(id,1,0,'Ab'),expectedBorrowedOptionalFallbackObservation(id,1,0,'Ab').checksum);
});
