import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertMapForEachSource} from './map-for-each-source-contract.mjs';
import {expectedMapForEachValues,expectedMapForEachObservation} from './map-for-each-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='map_for_each');
test('maps retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured, true);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertMapForEachSource(source,row,kind);
  assert.throws(()=>assertMapForEachSource(source+' ',row,kind));
  assert.throws(()=>assertMapForEachSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedMapForEachValues(row.id,2,3,'tail'),[['18','2','tail:updated|','1'],['20','2','tail:updated|','1']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedMapForEachObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertMapForEachSource(source'));assert.ok(source.includes("'map-for-each-oracle.mjs'"));}
});
