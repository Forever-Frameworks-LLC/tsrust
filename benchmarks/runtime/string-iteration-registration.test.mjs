import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertStringIterationSource} from './string-iteration-source-contract.mjs';
import {expectedStringIterationValues,expectedStringIterationObservation} from './string-iteration-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='string_iteration');
test('maps retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured, true);assert.equal(reportCategory(row.id),'strings');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertStringIterationSource(source,row,kind);
  assert.throws(()=>assertStringIterationSource(source+' ',row,kind));
  assert.throws(()=>assertStringIterationSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedStringIterationValues(row.id,2,3,'é🙂'),[['3:é|🙂|','2','🙂','éaéb🙂a🙂b'],['4:é|🙂|','2','🙂','éaéb🙂a🙂b']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedStringIterationObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertStringIterationSource(source'));assert.ok(source.includes("'string-iteration-oracle.mjs'"));}
});
