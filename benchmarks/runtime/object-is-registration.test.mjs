import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertObjectIsSource} from './object-is-source-contract.mjs';
import {objectIsIds,expectedObjectIsObservation,expectedObjectIsValues} from './object-is-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='object-is');
test('Object.is pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),objectIsIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-object-functions-milestone');assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertObjectIsSource(source,row,kind);
   assert.throws(()=>assertObjectIsSource(replaceCodeSnippet(source, 'Object.is(left, right)','left === right'),row,kind));
   assert.throws(()=>assertObjectIsSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertObjectIsSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertObjectIsSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertObjectIsSource\(source/);assert.ok(source.includes("'object-is-oracle.mjs'"));}
});
test('Object.is independent results match hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives modes 0, 1, 2 and bases 0, 13, 26.
 const cells=[
  ['object_is_values',3,0,'ab',[[true,true,true,false,true,true],[false,false,true,true,false,false],[false,false,true,false,false,false]]],
 ];
 for(const [id,n,seed,input,values] of cells){
  assert.deepEqual(expectedObjectIsValues(id,n,seed,input),values);
  const want=observeSliceTree(values);
  assert.deepEqual(expectedObjectIsObservation(id,n,seed,input),want);assert.deepEqual(expectedExactResult(id,n,seed,input),want);assert.equal(expectedChecksum(id,n,seed,input),want.checksum);
 }
 // The reasoned answers agree with the platform on every registered case.
 const input='dynamic é🙂 text',all=expectedObjectIsValues('object_is_values',2048,7,input);
 for(let i=0;i<2048;i++){const mode=(i+7)%3,base=(i*13+7)%97,left=base/4,right=(base+mode)/4,text=input.repeat(mode);
  assert.deepEqual(all[i],[Object.is(left,right),Object.is(left-right,right-left),Object.is((left-right)/(left-right),(right-left)/(right-left)),Object.is(text,input),Object.is(mode,0),Object.is(left,base)],String(i));}
 for(const row of rows){const want=expectedObjectIsObservation(row.id,2048,7,row.inputText);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
