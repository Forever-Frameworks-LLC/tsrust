import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertSetAlgebraSource} from './set-algebra-source-contract.mjs';
import {expectedSetAlgebraValues,expectedSetAlgebraObservation} from './set-algebra-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='set_algebra');
test('maps retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured, true);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertSetAlgebraSource(source,row,kind);
  assert.throws(()=>assertSetAlgebraSource(source+' ',row,kind));
  assert.throws(()=>assertSetAlgebraSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedSetAlgebraValues(row.id,2,3,'tail'),[['6,4,5,7','4,5','6','6,7','5,4','tail|last|'],['7,5,6,8','5,6','7','7,8','6,5','tail|last|']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedSetAlgebraObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertSetAlgebraSource(source'));assert.ok(source.includes("'set-algebra-oracle.mjs'"));}
});
