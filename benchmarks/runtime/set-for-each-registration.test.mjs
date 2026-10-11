import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertSetForEachSource} from './set-for-each-source-contract.mjs';
import {expectedSetForEachValues,expectedSetForEachObservation} from './set-for-each-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='set_for_each');
test('maps retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured, true);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertSetForEachSource(source,row,kind);
  assert.throws(()=>assertSetForEachSource(source+' ',row,kind));
  assert.throws(()=>assertSetForEachSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedSetForEachValues(row.id,2,3,'tail'),[['18','2','tail|','1'],['22','2','tail|','1']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedSetForEachObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertSetForEachSource(source'));assert.ok(source.includes("'set-for-each-oracle.mjs'"));}
});
