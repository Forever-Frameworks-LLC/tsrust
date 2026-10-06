import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertDestructuringSource} from './destructuring-source-contract.mjs';
import {destructuringIds,expectedDestructuringObservation,expectedDestructuringValues} from './destructuring-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='destructuring');
const original=JSON.parse(read('../../archive/destructuring-start/benchmarks-runtime/manifest.json'));
test('destructuring pair appends to the manifest and binds actual guarded sources',()=>{
 assert.deepEqual(manifest.workloads.slice(0,original.workloads.length),original.workloads);
 assert.deepEqual(rows.map(row=>row.id),destructuringIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-value-types-milestone');assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertDestructuringSource(source,row,kind);
   assert.throws(()=>assertDestructuringSource(source.replace('const { x, y, label } = point;','const x = point.x; const y = point.y; const label = point.label;'),row,kind));
   assert.throws(()=>assertDestructuringSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertDestructuringSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertDestructuringSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertDestructuringSource\(source/);assert.ok(source.includes("'destructuring-oracle.mjs'"));}
});
test('destructuring independent results match hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0 and 13.
 assert.deepEqual(expectedDestructuringValues('destructuring_patterns',2,0,'ab'),[['1','ab!ab','ab','1'],['28','ab!ab','ab','27']]);
 // The arithmetic agrees with the platform's destructuring on every registered case.
 const input='dynamic é🙂 text',all=expectedDestructuringValues('destructuring_patterns',2048,7,input);
 for(let i=0;i<2048;i++){const base=(i*13+7)%97;
  const [count,word]=[base+1,input+'!'];const {x,y,label}={x:base,y:i%7,label:input.slice()};
  let names='',total=0;for(const [name,amount] of [['a',base],['b',count]]){names+=name;total+=amount;}
  assert.deepEqual(all[i],[String(count+x+y),word+label,names,String(total)],String(i));}
 for(const row of rows){const want=observeSliceTree(expectedDestructuringValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedDestructuringObservation(row.id,2048,7,row.inputText),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
