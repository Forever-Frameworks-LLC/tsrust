import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertNumberParseIntSource} from './number-parse-int-source-contract.mjs';
import {numberParseIntIds,expectedNumberParseIntObservation,expectedNumberParseIntValues} from './number-parse-int-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='number-parse-int');
const original=JSON.parse(read('../../archive/number-parse-int-start/benchmarks-runtime/manifest.json'));
test('Number.parseInt pair appends to the manifest and binds actual guarded sources',()=>{
 assert.deepEqual(manifest.workloads.slice(0,original.workloads.length),original.workloads);
 assert.deepEqual(rows.map(row=>row.id),numberParseIntIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-number-methods-milestone');assert.equal(reportCategory(row.id),'numbers');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertNumberParseIntSource(source,row,kind);
   assert.throws(()=>assertNumberParseIntSource(source.replace('Number.parseInt(','Number.parseFloat('),row,kind));
   assert.throws(()=>assertNumberParseIntSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertNumberParseIntSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertNumberParseIntSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertNumberParseIntSource\(source/);assert.ok(source.includes("'number-parse-int-oracle.mjs'"));}
});
test('Number.parseInt independent results match hand-written cells and oracle routing',()=>{
 // Written by hand: digits are 0, 92 and 184 for seed 0; 8 and 100 for seed 1.
 const cells=[
  ['number_parse_int_radix',3,0,[[0,0,0,-0,NaN],[92,146,146,-326,NaN],[184,388,388,-1588,NaN]]],
  ['number_parse_int_radix',2,1,[[7,7,7,-7,NaN],[99,153,153,-333,NaN]]],
 ];
 for(const [id,n,seed,values] of cells){
  assert.deepEqual(expectedNumberParseIntValues(id,n,seed),values);
  const want=observeSliceTree(values,{numbers:true});
  assert.deepEqual(expectedNumberParseIntObservation(id,n,seed),want);assert.deepEqual(expectedExactResult(id,n,seed,'x'),want);assert.equal(expectedChecksum(id,n,seed,'x'),want.checksum);
 }
 for(const row of rows){const want=expectedNumberParseIntObservation(row.id,2048,7);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
