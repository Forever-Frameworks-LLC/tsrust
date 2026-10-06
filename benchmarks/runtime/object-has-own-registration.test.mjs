import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertObjectHasOwnSource} from './object-has-own-source-contract.mjs';
import {objectHasOwnIds,expectedObjectHasOwnValues} from './object-has-own-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='object-has-own');
test('Object membership registration binds nine retained queries and untouched history',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),objectHasOwnIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertObjectHasOwnSource(source,row,kind);
   for(const changed of [source+' ',source.replace('Object.hasOwn(row,"label")','true'),source.replace('return output;','return [];')])assert.throws(()=>assertObjectHasOwnSource(changed,row,kind));
   assert.throws(()=>assertObjectHasOwnSource(source,{...row,operationsPerIteration:1},kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertObjectHasOwnSource\(source/);assert.ok(source.includes("'object-has-own-oracle.mjs'"));}
});
test('independent membership oracle counts UTF-16 positions and canonical array indices',()=>{
 const expected=[['true','false','false','true','true','false','true','true','true'],['true','false','false','true','true','false','true','true','true'],['true','false','false','false','true','false','true','true','true'],['true','false','false','false','true','false','false','true','true']];
 assert.deepEqual(expectedObjectHasOwnValues('object_has_own',4,0,'é😀'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('object_has_own',4,0,'é😀'),observation);
 assert.equal(expectedChecksum('object_has_own',4,0,'é😀'),observation.checksum);
 assert.deepEqual(expectedObjectHasOwnValues('object_has_own',0,0,''),[]);
});
