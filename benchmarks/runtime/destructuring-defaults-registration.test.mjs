import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertDestructuringDefaultsSource} from './destructuring-defaults-source-contract.mjs';
import {destructuringDefaultsIds,expectedDestructuringDefaultsObservation,expectedDestructuringDefaultsValues} from './destructuring-defaults-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='destructuring-defaults');
test('Destructuring defaults pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),destructuringDefaultsIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-value-types-milestone');assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertDestructuringDefaultsSource(source,row,kind);
   assert.throws(()=>assertDestructuringDefaultsSource(replaceCodeSnippet(source, 'text.slice()','source'),row,kind));
   assert.throws(()=>assertDestructuringDefaultsSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertDestructuringDefaultsSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertDestructuringDefaultsSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertDestructuringDefaultsSource\(source/);assert.ok(source.includes("'destructuring-defaults-oracle.mjs'"));}
});
test('default oracle retains null and lazy call counts independently',()=>{
 const id='destructuring_defaults';const expected=[['ab','tail','none:0','0','1'],['ab','tail','abab:null','null','1'],['ab','tail','ababab:7','8','2']];
 assert.deepEqual(expectedDestructuringDefaultsValues(id,3,0,'ab'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult(id,3,0,'ab'),observation);
 assert.equal(expectedChecksum(id,3,0,'ab'),observation.checksum);
});
