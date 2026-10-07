import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertMathF16roundSource} from './math-f16round-source-contract.mjs';
import {mathF16roundIds,expectedMathF16roundObservation,expectedMathF16roundValues,nearestHalf} from './math-f16round-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='math-f16round');
test('Math.f16round pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),mathF16roundIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-math-functions-milestone');assert.equal(reportCategory(row.id),'numbers');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertMathF16roundSource(source,row,kind);
   assert.throws(()=>assertMathF16roundSource(replaceCodeSnippet(source, 'Math.f16round(65500.0 + scaled)','Math.fround(65500.0 + scaled)'),row,kind));
   assert.throws(()=>assertMathF16roundSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertMathF16roundSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertMathF16roundSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertMathF16roundSource\(source/);assert.ok(source.includes("'math-f16round-oracle.mjs'"));}
});
test('Math.f16round independent results match hand-written cells, the platform and oracle routing',()=>{
 // Written by hand from the half-precision grid: 11 significant bits.
 const cells=[[0.1,0.0999755859375],[2049,2048],[2051,2052],[-1/67108864,-0],[-3/67108864,-(2**-24)],[65519,65504],[65520,Infinity],[-1,-1],[1.337,1.3369140625],[2**-25,0],[1.5*2**-24,2**-23]];
 for(const [input,want] of cells){const got=nearestHalf(input);assert.ok(Object.is(got===0&&input<0?-0:got,want),`${input}`);}
 // The arithmetic agrees with the platform on every registered case.
 const all=expectedMathF16roundValues('math_f16round_values',2048,7);
 for(let i=0;i<2048;i++){const base=(i*13+7)%97;
  const want=[base/7+0.1,2049+base*2,-(base+1)/67108864,65500+base,i*0.001-1].map(value=>Math.f16round(value));
  for(let cell=0;cell<5;cell++)assert.ok(Object.is(all[i][cell],want[cell]),`${i}:${cell}`);}
 for(const row of rows){const want=expectedMathF16roundObservation(row.id,2048,7);assert.deepEqual(want,observeSliceTree(expectedMathF16roundValues(row.id,2048,7),{numbers:true}));assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
