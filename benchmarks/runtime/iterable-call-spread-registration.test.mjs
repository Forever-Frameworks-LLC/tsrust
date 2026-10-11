import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertIterableCallSpreadSource} from './iterable-call-spread-source-contract.mjs';
import {expectedIterableCallSpreadValues,expectedIterableCallSpreadObservation} from './iterable-call-spread-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='iterable_call_spread');
test('iterable spread retains matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured, true);assert.equal(reportCategory(row.id),'arrays');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertIterableCallSpreadSource(source,row,kind);
  assert.throws(()=>assertIterableCallSpreadSource(source+' ',row,kind));
  assert.throws(()=>assertIterableCallSpreadSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedIterableCallSpreadValues(row.id,2,3,'tail'),[['8','t|a|i|l','12','5','tail:3','2','<a:b>'],['10','t|a|i|l','14','6','tail:4','2','<a:b>']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedIterableCallSpreadObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertIterableCallSpreadSource(source'));assert.ok(source.includes("'iterable-call-spread-oracle.mjs'"));}
});
