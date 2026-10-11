import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertMapIterationSource} from './map-iteration-source-contract.mjs';
import {expectedMapIterationValues,expectedMapIterationObservation} from './map-iteration-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='map_iteration');
test('maps retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured, true);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertMapIterationSource(source,row,kind);
  assert.throws(()=>assertMapIterationSource(source+' ',row,kind));
  assert.throws(()=>assertMapIterationSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedMapIterationValues(row.id,2,3,'tail'),[['tail:4|','tail:4|','2'],['tail:5|','tail:5|','2']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedMapIterationObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertMapIterationSource(source'));assert.ok(source.includes("'map-iteration-oracle.mjs'"));}
});
