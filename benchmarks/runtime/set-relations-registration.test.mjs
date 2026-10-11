import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertSetRelationsSource} from './set-relations-source-contract.mjs';
import {expectedSetRelationsValues,expectedSetRelationsObservation} from './set-relations-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='set_relations');
test('maps retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured, true);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertSetRelationsSource(source,row,kind);
  assert.throws(()=>assertSetRelationsSource(source+' ',row,kind));
  assert.throws(()=>assertSetRelationsSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedSetRelationsValues(row.id,2,3,'tail'),[['true','true','true','false','false','true'],['true','true','true','false','false','true']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedSetRelationsObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertSetRelationsSource(source'));assert.ok(source.includes("'set-relations-oracle.mjs'"));}
});
