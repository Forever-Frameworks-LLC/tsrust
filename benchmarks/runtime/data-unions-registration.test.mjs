import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertDataUnionsSource} from './data-unions-source-contract.mjs';
import {dataUnionsIds,expectedDataUnionsObservation,expectedDataUnionsValues} from './data-unions-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='data-unions');
test('Data unions pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),dataUnionsIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-value-types-milestone');assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertDataUnionsSource(source,row,kind);
   assert.throws(()=>assertDataUnionsSource(replaceCodeSnippet(source, 'copyRow(source)','source'),row,kind));
   assert.throws(()=>assertDataUnionsSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertDataUnionsSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertDataUnionsSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertDataUnionsSource\(source/);assert.ok(source.includes("'data-unions-oracle.mjs'"));}
});
test('data union formula retains tags, original owners, copies and readonly observations',()=>{
 assert.deepEqual(expectedDataUnionsValues('data_union_values',4,0,'ab'),[['0','number','ab','5','0','ab'],['abab','string','13,14','18','ABAB','13,14'],['false','boolean','ab','31','false','ab'],['null','object','39,40','44','missing','39,40']]);
 assert.deepEqual(expectedDataUnionsValues('data_union_values',0,0,''),[]);assert.throws(()=>expectedDataUnionsValues('data_union_values',2049,0,''));
 for(const row of rows){const want=observeSliceTree(expectedDataUnionsValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
