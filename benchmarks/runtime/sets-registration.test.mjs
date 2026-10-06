import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertSetsSource} from './sets-source-contract.mjs';
import {expectedSetsValues,expectedSetsObservation} from './sets-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='set_values');
test('sets retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertSetsSource(source,row,kind);
  assert.throws(()=>assertSetsSource(source+' ',row,kind));
  assert.throws(()=>assertSetsSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedSetsValues(row.id,2,3,'tail'),[['2','true','4,3','1','0','4:3:','tail'],['2','true','5,4','1','0','5:4:','tail']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedSetsObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertSetsSource(source'));assert.ok(source.includes("'sets-oracle.mjs'"));}
});
