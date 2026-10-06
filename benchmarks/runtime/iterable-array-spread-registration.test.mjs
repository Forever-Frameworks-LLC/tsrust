import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertIterableArraySpreadSource} from './iterable-array-spread-source-contract.mjs';
import {expectedIterableArraySpreadValues,expectedIterableArraySpreadObservation} from './iterable-array-spread-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='iterable_array_spread');
test('iterable spread retains matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'arrays');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertIterableArraySpreadSource(source,row,kind);
  assert.throws(()=>assertIterableArraySpreadSource(source+' ',row,kind));
  assert.throws(()=>assertIterableArraySpreadSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedIterableArraySpreadValues(row.id,2,3,'tail'),[['0,5,3,6','t|a|i|l|é|🙂','1','tail:4','tail:3'],['0,6,4,7','t|a|i|l|é|🙂','1','tail:5','tail:4']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedIterableArraySpreadObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertIterableArraySpreadSource(source'));assert.ok(source.includes("'iterable-array-spread-oracle.mjs'"));}
});
