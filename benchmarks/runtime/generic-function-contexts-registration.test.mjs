import {containsCodeSnippet, replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertGenericContextSource} from './generic-function-contexts-source-contract.mjs';
import {expectedGenericContextValues,expectedGenericContextObservation} from './generic-function-contexts-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(row=>row.id==='generic_function_contexts');
test('Contextual generic values retain matched sources and independent result evidence',()=>{
 assert.equal(row.id,'generic_function_contexts');
 assert.equal(row.implementationAccepted,true);assert.equal(row.measured,true);assert.equal(row.measurementStatus,'measured-named-declaration-family');
 assert.equal(reportCategory(row.id),'callbacks');
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertGenericContextSource(source,row,kind);
  assert.throws(()=>assertGenericContextSource(source+' ',row,kind));
  assert.throws(()=>assertGenericContextSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['numberResult = numbers(base);','numberResult = numbers(base + 1);'],['calls += 1; return value;','calls += 2; return value;'],['headResult = first(values);','headResult = first([0]);']]){
   assert.ok(containsCodeSnippet(source, from));const changed=replaceCodeSnippet(source, from,to);
   assert.throws(()=>assertGenericContextSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedGenericContextValues(row.id,2,0,'context'),[[1,5,2,4,1,4,7],[2,5,3,5,1,4,7]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedGenericContextObservation(row.id,n,seed,'context');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'context'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'context'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertGenericContextSource('));assert.ok(source.includes("'generic-function-contexts-oracle.mjs'"));
 }
});
