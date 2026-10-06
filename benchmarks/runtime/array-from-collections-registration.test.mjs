import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertArrayFromCollectionsSource} from './array-from-collections-source-contract.mjs';
import {expectedArrayFromCollectionsValues,expectedArrayFromCollectionsObservation} from './array-from-collections-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='array_from_collections');
test('maps retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'arrays');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertArrayFromCollectionsSource(source,row,kind);
  assert.throws(()=>assertArrayFromCollectionsSource(source+' ',row,kind));
  assert.throws(()=>assertArrayFromCollectionsSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedArrayFromCollectionsValues(row.id,2,3,'tail'),[['5,3','50,31','1','tail:4','tail:3'],['6,4','60,41','1','tail:5','tail:4']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedArrayFromCollectionsObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertArrayFromCollectionsSource(source'));assert.ok(source.includes("'array-from-collections-oracle.mjs'"));}
});
