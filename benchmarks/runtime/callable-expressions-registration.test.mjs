import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertCallableExpressionsSource} from './callable-expressions-source-contract.mjs';
import {callableExpressionsIds,expectedCallableExpressionsValues} from './callable-expressions-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='callable-expressions');
test('callable expression registration preserves history and complete scenarios',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),callableExpressionsIds);
 for(const row of rows){assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertCallableExpressionsSource(source,row,kind);
   for(const text of ['while (i < n)','const direct=make(base)(3);','const nested=chain(base)(2)(3);','trace.push(1);','trace.push(2);','return output;','input+String(base)']){
    const changed=source.replace(text,'');const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertCallableExpressionsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertCallableExpressionsSource(source,{...row,operationsPerIteration:9},kind));
   assert.throws(()=>assertCallableExpressionsSource(source+' ',row,kind));
  }
 }
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.match(source,/assertCallableExpressionsSource\(source/);assert.ok(source.includes("'callable-expressions-oracle.mjs'"));}
});
test('independent callable oracle retains results and agrees with platform',()=>{
 const expected=[['3','5','0','4','2','7','5','1,2','ab0'],['16','18','26','17','15','7','18','1,2','ab13']];
 assert.deepEqual(expectedCallableExpressionsValues('callable_expressions',2,0,'ab'),expected);
 const observation=observeSliceTree(expected);assert.deepEqual(expectedExactResult('callable_expressions',2,0,'ab'),observation);assert.equal(expectedChecksum('callable_expressions',2,0,'ab'),observation.checksum);
 assert.deepEqual(expectedCallableExpressionsValues('callable_expressions',0,0,''),[]);
 for(const input of ['dynamic é🙂 text','abc','']){const expected=expectedCallableExpressionsValues('callable_expressions',2048,7,input);
  for(let i=0;i<2048;i++){const base=(i*13+7)%97,trace=[];
   const make=base=>value=>base+value,chain=a=>b=>c=>a+b+c;
   const factory=(trace,base)=>{trace.push(1);return value=>value+base;};
   const mark=(trace,value)=>{trace.push(2);return value;};
   const args=[base,2],ordered=factory(trace,base)(mark(trace,5));
   assert.deepEqual(expected[i],[String(make(base)(3)),String(chain(base)(2)(3)),String((value=>value*2)(base)),String((function(value){return value+4;})(base)),String(((a,b)=>a+b)(...args)),String(((value=7)=>value)()),String(ordered),trace.join(','),input+base]);
  }
 }
});
