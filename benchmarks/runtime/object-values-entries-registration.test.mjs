import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertObjectValuesSource} from './object-values-entries-source-contract.mjs';
import {objectValuesIds,expectedObjectValuesObservation,expectedObjectValuesValues} from './object-values-entries-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='object-values-entries');
test('object values pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),objectValuesIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-object-functions-milestone');assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertObjectValuesSource(source,row,kind);
   assert.throws(()=>assertObjectValuesSource(replaceCodeSnippet(source, 'const values = Object.values(row);','const values = [row.alpha];'),row,kind));
   assert.throws(()=>assertObjectValuesSource(replaceCodeSnippet(source, 'Object.entries(sides).length','2'),row,kind));
   assert.throws(()=>assertObjectValuesSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertObjectValuesSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertObjectValuesSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertObjectValuesSource\(source/);assert.ok(source.includes("'object-values-entries-oracle.mjs'"));}
});
test('object values independent results match hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0 and 13; the first row uses the gamma-first order.
 assert.deepEqual(expectedObjectValuesValues('object_values_entries',2,0,'ab'),[
  ['Tab','0','true|0|ab','0','gamma=true;alpha=0;beta=ab;','2'],
  ['abF','13','ab|false|13','12','beta=ab;gamma=false;alpha=13;','2']]);
 // The written orders agree with the platform on every registered case.
 const input='dynamic é🙂 text',all=expectedObjectValuesValues('object_values_entries',2048,7,input);
 for(let i=0;i<2048;i++){const base=(i*13+7)%97;
  const row=(i+7)%2===0?{gamma:true,alpha:base,beta:input.slice()}:{beta:input.slice(),gamma:false,alpha:base};
  const values=Object.values(row);let text='',sum=0;
  for(const value of values){if(typeof value==='string')text+=value;else if(typeof value==='number')sum+=value;else text+=value?'T':'F';}
  const sides={h:base,w:i%5},lengths=Object.values(sides);let names='';
  for(const pair of Object.entries(row))names+=pair[0]+'='+String(pair[1])+';';
  assert.deepEqual(all[i],[text,String(sum),values.join('|'),String(lengths[0]-lengths[1]),names,String(Object.entries(sides).length)],String(i));}
 for(const row of rows){const want=observeSliceTree(expectedObjectValuesValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedObjectValuesObservation(row.id,2048,7,row.inputText),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
