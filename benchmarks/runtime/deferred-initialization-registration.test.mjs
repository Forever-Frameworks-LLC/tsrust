import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertDeferredInitializationSource} from './deferred-initialization-source-contract.mjs';
import {deferredInitializationIds,expectedDeferredInitializationValues} from './deferred-initialization-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='deferred-initialization');
const original=JSON.parse(read('../../archive/uninitialized-let-benchmark-start/benchmarks/runtime/manifest.json'));
test('deferred initialization registration binds complete scenarios and preserves prior manifest rows',()=>{
 assert.deepEqual(manifest.workloads.slice(0,original.workloads.length),original.workloads);
 assert.deepEqual(rows.map(row=>row.id),deferredInitializationIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertDeferredInitializationSource(source,row,kind);
   for(const changed of [source.replace('let label:string;','let label:string="";'),source.replace('let size:int,step:int=base%5;','let size:int=0,step:int=base%5;'),source.replace('let note:string|undefined;','let note:string|undefined=undefined;'),source.replace('return output;','return [];')]){
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertDeferredInitializationSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertDeferredInitializationSource(source,{...row,operationsPerIteration:5},kind));
   assert.throws(()=>assertDeferredInitializationSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertDeferredInitializationSource\(source/);assert.ok(source.includes("'deferred-initialization-oracle.mjs'"));}
});
test('independent deferred initialization oracle matches hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 60 gives bases 60 and 73; steps 0 and 3.
 const expected=[['mid','1','zero','60ab','true'],['high','7','many','73ab','true']];
 assert.deepEqual(expectedDeferredInitializationValues('deferred_initialization',2,60,'ab'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('deferred_initialization',2,60,'ab'),observation);
 assert.equal(expectedChecksum('deferred_initialization',2,60,'ab'),observation.checksum);
 assert.deepEqual(expectedDeferredInitializationValues('deferred_initialization',0,0,''),[]);
 // The direct forms agree with the platform's deferred declarations on every registered case.
 for(const input of ['dynamic é🙂 text','abc','']){
  const all=expectedDeferredInitializationValues('deferred_initialization',2048,7,input);
  for(let i=0;i<2048;i++){
   const base=(i*13+7)%97;
   let label;if(base>=64){label='high';}else if(base>=32){label='mid';}else{label='low';}
   let size,step=base%5;size=step*2+1;
   let kind;switch(step){case 0:kind='zero';break;case 1:case 2:kind='few';break;default:kind='many';}
   let text;{const prefix=String(base);text=prefix+input;}
   let note;const absent=note===undefined;
   assert.deepEqual(all[i],[label,String(size),kind,text,String(absent)],input+':'+i);}
 }
});
