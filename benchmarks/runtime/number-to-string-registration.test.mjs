import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertNumberToStringSource} from './number-to-string-source-contract.mjs';
import {numberToStringIds,expectedNumberToStringObservation,expectedNumberToStringValues} from './number-to-string-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='number-to-string');
test('number toString pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),numberToStringIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-number-methods-milestone');assert.equal(reportCategory(row.id),'numbers');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertNumberToStringSource(source,row,kind);
   assert.throws(()=>assertNumberToStringSource(replaceCodeSnippet(source, '.toString(2)','.toFixed(2)'),row,kind));
   assert.throws(()=>assertNumberToStringSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertNumberToStringSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertNumberToStringSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertNumberToStringSource\(source/);assert.ok(source.includes("'number-to-string-oracle.mjs'"));}
});
test('number toString independent results match hand-written cells and oracle routing',()=>{
 // Written by hand: seed 0 gives 0, 13.125 and 26.25.
 const cells=[
  ['number_to_string_radix',3,0,[['0','0','0','0','0'],['13.125','1101.001','d.2','d.4i','-15.1'],['26.25','11010.01','1a.4','q.9','-32.2']]],
 ];
 for(const [id,n,seed,values] of cells){
  assert.deepEqual(expectedNumberToStringValues(id,n,seed),values);
  const want=observeSliceTree(values);
  assert.deepEqual(expectedNumberToStringObservation(id,n,seed),want);assert.deepEqual(expectedExactResult(id,n,seed,'x'),want);assert.equal(expectedChecksum(id,n,seed,'x'),want.checksum);
 }
 // The independent digits agree with the platform formatter on every registered case.
 const all=expectedNumberToStringValues('number_to_string_radix',2048,7);
 for(let i=0;i<2048;i++){const value=(i*13+7)%97+(i%8)/8;assert.deepEqual(all[i],[value.toString(),value.toString(2),value.toString(16),value.toString(36),(-value).toString(8)]);}
 for(const row of rows){const want=expectedNumberToStringObservation(row.id,2048,7);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
