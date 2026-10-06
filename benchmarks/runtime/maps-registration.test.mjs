import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertMapsSource} from './maps-source-contract.mjs';
import {expectedMapsValues,expectedMapsObservation} from './maps-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='map_values');
test('maps retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertMapsSource(source,row,kind);
  assert.throws(()=>assertMapsSource(source+' ',row,kind));
  assert.throws(()=>assertMapsSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedMapsValues(row.id,2,3,'tail'),[['3','1','4','true','true','false','0','2','tail'],['3','1','4','true','true','false','0','2','tail']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedMapsObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertMapsSource(source'));assert.ok(source.includes("'maps-oracle.mjs'"));}
});
