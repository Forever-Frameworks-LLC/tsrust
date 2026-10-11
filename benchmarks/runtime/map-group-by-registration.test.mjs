import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertMapGroupBySource} from './map-group-by-source-contract.mjs';
import {expectedMapGroupByValues,expectedMapGroupByObservation} from './map-group-by-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='map_group_by');
test('Map grouping retains matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured, true);assert.equal(reportCategory(row.id),'arrays');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){const source=read('./'+name);assertMapGroupBySource(source,row,kind);assert.throws(()=>assertMapGroupBySource(source+' ',row,kind));assert.throws(()=>assertMapGroupBySource(source,{...row,operationsPerIteration:2},kind));}
 assert.deepEqual(expectedMapGroupByValues(row.id,1,3,'a🙂a'),[['2','1,0','3,5','4,6','10','3,4,5,6,7','2','a|a','🙂']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedMapGroupByObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertMapGroupBySource(source'));assert.ok(source.includes("'map-group-by-oracle.mjs'"));}
});
