import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertSetEntriesSource} from './set-entries-source-contract.mjs';
import {expectedSetEntriesValues,expectedSetEntriesObservation} from './set-entries-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='set_entries');
test('maps retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured, true);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertSetEntriesSource(source,row,kind);
  assert.throws(()=>assertSetEntriesSource(source+' ',row,kind));
  assert.throws(()=>assertSetEntriesSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedSetEntriesValues(row.id,2,3,'tail'),[['4','4','3','3','tail','tail','1','0'],['5','5','4','4','tail','tail','1','0']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedSetEntriesObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertSetEntriesSource(source'));assert.ok(source.includes("'set-entries-oracle.mjs'"));}
});
