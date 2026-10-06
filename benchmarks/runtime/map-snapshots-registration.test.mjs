import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertMapSnapshotsSource} from './map-snapshots-source-contract.mjs';
import {expectedMapSnapshotsValues,expectedMapSnapshotsObservation} from './map-snapshots-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='map_snapshots');
test('maps retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertMapSnapshotsSource(source,row,kind);
  assert.throws(()=>assertMapSnapshotsSource(source+' ',row,kind));
  assert.throws(()=>assertMapSnapshotsSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedMapSnapshotsValues(row.id,2,3,'tail'),[['4,3','tail|tail','4','tail','3','tail','0'],['5,4','tail|tail','5','tail','4','tail','0']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedMapSnapshotsObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertMapSnapshotsSource(source'));assert.ok(source.includes("'map-snapshots-oracle.mjs'"));}
});
