import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNestedRestPatternsSource} from './nested-rest-patterns-source-contract.mjs';
import {nestedRestPatternsIds,expectedNestedRestPatternsValues} from './nested-rest-patterns-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='nested-rest-patterns');
test('nested rest registration binds complete scenarios and preserves prior manifest rows',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),nestedRestPatternsIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertNestedRestPatternsSource(source,row,kind);
   for(const changed of [replaceCodeSnippet(source, 'tail.push("end")','tail.push("other")'),replaceCodeSnippet(source, 'return output;','return [];')]){
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertNestedRestPatternsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertNestedRestPatternsSource(source,{...row,operationsPerIteration:4},kind));
   assert.throws(()=>assertNestedRestPatternsSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertNestedRestPatternsSource\(source/);assert.ok(source.includes("'nested-rest-patterns-oracle.mjs'"));}
});
test('independent nested rest oracle retains all copied values and constructor key order',()=>{
 const expected=[['ab','0','abab|end','2','0','ab','1','abab:tail','3'],['ab','13','abab|end','2','13','ab','14','abab:tail','3']];
 assert.deepEqual(expectedNestedRestPatternsValues('nested_rest_patterns',2,0,'ab'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('nested_rest_patterns',2,0,'ab'),observation);
 assert.equal(expectedChecksum('nested_rest_patterns',2,0,'ab'),observation.checksum);
 assert.deepEqual(expectedNestedRestPatternsValues('nested_rest_patterns',0,0,''),[]);
});
