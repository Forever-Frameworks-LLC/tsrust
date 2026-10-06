import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertForPatternInitializersSource} from './for-pattern-initializers-source-contract.mjs';
import {forPatternInitializersIds,expectedForPatternInitializersValues} from './for-pattern-initializers-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='for-pattern-initializers');
const original=JSON.parse(read('../../archive/for-pattern-initializers-benchmark-start/benchmarks/runtime/manifest.json'));
test('classic-for pattern registration binds complete scenarios and preserves prior manifest rows',()=>{
 assert.deepEqual(manifest.workloads.slice(0,original.workloads.length),original.workloads);
 assert.deepEqual(rows.map(row=>row.id),forPatternInitializersIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertForPatternInitializersSource(source,row,kind);
   for(const changed of [source.replace('tail.push("x")','tail.push("y")'),source.replace('{continue;}','{}'),source.replace('return output;','return [];')]){
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertForPatternInitializersSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertForPatternInitializersSource(source,{...row,operationsPerIteration:3},kind));
   assert.throws(()=>assertForPatternInitializersSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertForPatternInitializersSource\(source/);assert.ok(source.includes("'for-pattern-initializers-oracle.mjs'"));}
});
test('independent classic-for pattern oracle matches hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0, 13, 26 and loop starts 0, 1, 2.
 const expected=[['2','3','ab1ab2','2'],['4','3','ab1ab2','2'],['6','3','ab1ab2','2']];
 assert.deepEqual(expectedForPatternInitializersValues('for_pattern_initializers',3,0,'ab'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('for_pattern_initializers',3,0,'ab'),observation);
 assert.equal(expectedChecksum('for_pattern_initializers',3,0,'ab'),observation.checksum);
 assert.deepEqual(expectedForPatternInitializersValues('for_pattern_initializers',0,0,''),[]);
 // The closed forms agree with the platform's loops on every registered case.
 const input='dynamic é🙂 text',all=expectedForPatternInitializersValues('for_pattern_initializers',2048,7,input);
 for(let i=0;i<2048;i++){const base=(i*13+7)%97,words=[input,String(base)];
  let sum=0;for(let {start:k,end}={start:base%3,end:base%3+3};k<end;k++){if(k===base%3+1)continue;sum+=k;}
  let count=0;for(let [j,stop=2]=[0];j<stop;j++)count+=j+1;
  let text='';for(const [first,...tail]=words;tail.length<3;tail.push('x'))text+=first+String(tail.length);
  assert.deepEqual(all[i],[String(sum),String(count),text,String(words.length)],String(i));}
});
