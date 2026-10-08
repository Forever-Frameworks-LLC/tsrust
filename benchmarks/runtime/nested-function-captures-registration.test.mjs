import {containsCodeSnippet, replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNestedFunctionCapturesSource} from './nested-function-captures-source-contract.mjs';
import {expectedNestedFunctionCaptureValues,expectedNestedFunctionCaptureObservation} from './nested-function-captures-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='nested_function_captures');
test('Nested captures retain sources, independent observations and execution hooks',()=>{
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 assert.equal(row.measured,true);assert.equal(row.measurementStatus,'measured-named-declaration-family');assert.equal(reportCategory(row.id),'callbacks');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertNestedFunctionCapturesSource(source,row,kind);
  assert.throws(()=>assertNestedFunctionCapturesSource(source+' ',row,kind));
  assert.throws(()=>assertNestedFunctionCapturesSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['depth(3)','depth(2)'],['const run = tick;','const run = (): int => tick();'],['later = value + 3;\n','']]){
   assert.ok(containsCodeSnippet(source, from));const changed=replaceCodeSnippet(source, from,to);
   assert.throws(()=>assertNestedFunctionCapturesSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedNestedFunctionCaptureValues(row.id,2,0,'captures'),[[2,2,2,3,5,8,2],[6,4,3,4,6,10,4]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedNestedFunctionCaptureObservation(row.id,n,seed,'captures');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'captures'),expected);assert.equal(expectedChecksum(row.id,n,seed,'captures'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertNestedFunctionCapturesSource(source'));assert.ok(source.includes("'nested-function-captures-oracle.mjs'"));
 }
});
