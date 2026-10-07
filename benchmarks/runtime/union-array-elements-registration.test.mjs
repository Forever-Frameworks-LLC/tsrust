import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertUnionArraySource} from './union-array-elements-source-contract.mjs';
import {unionArrayIds,expectedUnionArrayObservation,expectedUnionArrayValues} from './union-array-elements-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='union-array-elements');
test('union array pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),unionArrayIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-value-types-milestone');assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertUnionArraySource(source,row,kind);
   assert.throws(()=>assertUnionArraySource(replaceCodeSnippet(source, 'values.includes(null)','true'),row,kind));
   assert.throws(()=>assertUnionArraySource(replaceCodeSnippet(source, '(int | string | null)[]','string[]'),row,kind));
   assert.throws(()=>assertUnionArraySource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertUnionArraySource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertUnionArraySource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertUnionArraySource\(source/);assert.ok(source.includes("'union-array-elements-oracle.mjs'"));}
});
test('union array independent results match hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0, 13, 26 and modes 0, 1, 2.
 assert.deepEqual(expectedUnionArrayValues('union_array_values',3,0,'ab'),[
  ['0','tail','0|||tail','true','-1','tail','2'],
  ['27','ab','13|ab||14','true','3','14','1'],
  ['53','abab','26|abab||27','true','3','27','1']]);
 // The reasoned cells agree with the platform on every registered case.
 const input='dynamic é🙂 text',all=expectedUnionArrayValues('union_array_values',2048,7,input);
 for(let i=0;i<2048;i++){const base=(i*13+7)%97,mode=(i+7)%3,values=[base,input.repeat(mode),null];
  if(mode===0)values.push('tail');else values.push(base+1);
  let sum=0,text='';for(const value of values){if(typeof value==='number')sum+=value;else if(typeof value==='string')text+=value;}
  assert.deepEqual(all[i],[String(sum),text,values.join('|'),String(values.includes(null)),String(values.indexOf(base+1)),String(values.at(-1)),String(values.filter(value=>typeof value==='string').length)],String(i));}
 for(const row of rows){const want=observeSliceTree(expectedUnionArrayValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedUnionArrayObservation(row.id,2048,7,row.inputText),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
