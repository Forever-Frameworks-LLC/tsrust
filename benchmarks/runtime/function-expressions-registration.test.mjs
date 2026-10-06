import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertFunctionExpressionsSource} from './function-expressions-source-contract.mjs';
import {functionExpressionsIds,expectedFunctionExpressionsValues} from './function-expressions-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='function-expressions');
test('function expression registration binds complete scenarios and preserves prior manifest rows',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),functionExpressionsIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertFunctionExpressionsSource(source,row,kind);
   for(const changed of [source.replace('tags.map(function (tag){return tag.length;})','tags.map((tag)=>{return tag.length;})'),source.replace('sizes.forEach(function (size:int){total+=size;});',''),source.replace('return output;','return [];')]){
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertFunctionExpressionsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertFunctionExpressionsSource(source,{...row,operationsPerIteration:5},kind));
   assert.throws(()=>assertFunctionExpressionsSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertFunctionExpressionsSource\(source/);assert.ok(source.includes("'function-expressions-oracle.mjs'"));}
});
test('independent function expression oracle matches hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0 and 13; digits sort before letters.
 const expected=[['0','5','0','a0','b2'],['16','6','13','a0','b2']];
 assert.deepEqual(expectedFunctionExpressionsValues('function_expressions',2,0,'ab'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('function_expressions',2,0,'ab'),observation);
 assert.equal(expectedChecksum('function_expressions',2,0,'ab'),observation.checksum);
 assert.deepEqual(expectedFunctionExpressionsValues('function_expressions',0,0,''),[]);
 // The direct forms agree with the platform's function expressions on every registered case.
 for(const input of ['dynamic é🙂 text','abc','','~tilde']){
  const all=expectedFunctionExpressionsValues('function_expressions',2048,7,input);
  for(let i=0;i<2048;i++){
   const base=(i*13+7)%97;const scale=function(value){return value*3+base;};
   const sizes=[String(base),'z','tag'].map(function(tag){return tag.length;});
   const words=[input,String(base),'z'];words.sort(function(left,right){return left<right?-1:1;});
   let total=0;sizes.forEach(function(size){total+=size;});
   const join=function(first='a',...rest){return first+String(rest.length);};
   assert.deepEqual(all[i],[String(scale(i%5)),String(total),words[0],join(),join('b','c','d')],input+':'+i);}
 }
});
