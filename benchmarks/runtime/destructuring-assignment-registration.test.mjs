import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertDestructuringAssignmentSource} from './destructuring-assignment-source-contract.mjs';
import {destructuringAssignmentIds,expectedDestructuringAssignmentValues} from './destructuring-assignment-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='destructuring-assignment');
const original=JSON.parse(read('../../archive/destructuring-assignment-benchmark-start/benchmarks/runtime/manifest.json'));
test('destructuring assignment registration binds complete scenarios and preserves prior manifest rows',()=>{
 assert.deepEqual(manifest.workloads.slice(0,original.workloads.length),original.workloads);
 assert.deepEqual(rows.map(row=>row.id),destructuringAssignmentIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertDestructuringAssignmentSource(source,row,kind);
   for(const changed of [source.replace('[left,right]=[right,left];',''),source.replace('const cell:Cell={x,y,label};','const cell:Cell={x:x,y:y,label:label};'),source.replace('return output;','return [];')]){
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertDestructuringAssignmentSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertDestructuringAssignmentSource(source,{...row,operationsPerIteration:5},kind));
   assert.throws(()=>assertDestructuringAssignmentSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertDestructuringAssignmentSource\(source/);assert.ok(source.includes("'destructuring-assignment-oracle.mjs'"));}
});
test('independent destructuring assignment oracle matches hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0 and 13; the loop runs 2 and 3 times.
 const expected=[['0','ab','1','2','ab?','2','ab?!'],['13','ab','2','15','ab?','28','ab?!']];
 assert.deepEqual(expectedDestructuringAssignmentValues('destructuring_assignment',2,0,'ab'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('destructuring_assignment',2,0,'ab'),observation);
 assert.equal(expectedChecksum('destructuring_assignment',2,0,'ab'),observation.checksum);
 assert.deepEqual(expectedDestructuringAssignmentValues('destructuring_assignment',0,0,''),[]);
 // The table and closed forms agree with the platform's pattern assignments on every registered case.
 const input='dynamic é🙂 text',all=expectedDestructuringAssignmentValues('destructuring_assignment',2048,7,input);
 for(let i=0;i<2048;i++){
  const base=(i*13+7)%97;let left=input,right=String(base);[left,right]=[right,left];
  let f0=0,f1=1;for(let turn=0;turn<base%4+2;[f0,f1]=[f1,f0+f1],turn++){}
  let count=0,word='';[count,word]=[base+2,input+'?'];
  const x=base,y=count,label=word+'!',cell={x,y,label};let cx=0,text='';({x:cx,label:text}=cell);
  assert.deepEqual(all[i],[left,right,String(f0),String(count),word,String(cx+cell.y),text],String(i));}
});
