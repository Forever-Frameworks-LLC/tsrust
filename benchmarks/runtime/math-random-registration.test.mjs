import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertMathRandomSource} from './math-random-source-contract.mjs';
import {mathRandomIds,expectedMathRandomObservation,expectedMathRandomValues} from './math-random-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='math-random');
const original=JSON.parse(read('../../archive/math-random-start/benchmarks-runtime/manifest.json'));
test('Math.random pair appends to the manifest and binds actual guarded sources',()=>{
 assert.deepEqual(manifest.workloads.slice(0,original.workloads.length),original.workloads);
 assert.deepEqual(rows.map(row=>row.id),mathRandomIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-math-functions-milestone');assert.equal(reportCategory(row.id),'numbers');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertMathRandomSource(source,row,kind);
   assert.throws(()=>assertMathRandomSource(source.replace('const second = Math.random();','const second = 0.5;'),row,kind));
   assert.throws(()=>assertMathRandomSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertMathRandomSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertMathRandomSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertMathRandomSource\(source/);assert.ok(source.includes("'math-random-oracle.mjs'"));}
});
test('Math.random range properties match a hand-written cell, the platform and oracle routing',()=>{
 assert.deepEqual(expectedMathRandomValues('math_random_range',2,0),[[true,true,true,true],[true,true,true,true]]);
 // The platform generator satisfies the same properties on every registered case.
 for(let i=0;i<2048;i++){const sides=(i*13+7)%97+1,first=Math.random(),second=Math.random(),roll=Math.floor(first*sides);
  assert.deepEqual([first>=0&&first<1,second>=0&&second<1,first!==second,roll>=0&&roll<sides],[true,true,true,true],String(i));}
 for(const row of rows){const want=observeSliceTree(expectedMathRandomValues(row.id,2048,7));assert.deepEqual(expectedMathRandomObservation(row.id,2048,7),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
