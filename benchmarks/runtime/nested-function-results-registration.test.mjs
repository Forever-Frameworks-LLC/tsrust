import {containsCodeSnippet, replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNestedResultSource} from './nested-function-results-source-contract.mjs';
import {expectedNestedResultValues,expectedNestedResultObservation} from './nested-function-results-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(row=>row.id==='nested_function_results');
test('Inferred named results retain matched sources and independent result evidence',()=>{
 assert.equal(row.id,'nested_function_results');
 assert.equal(row.implementationAccepted,true);assert.equal(row.measured,false);
 assert.equal(reportCategory(row.id),'callbacks');
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertNestedResultSource(source,row,kind);
  assert.throws(()=>assertNestedResultSource(source+' ',row,kind));
  assert.throws(()=>assertNestedResultSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['identity(value + 3)','identity(value + 4)'],['return value + value;','return value;'],['values[1]','values[0]']]){
   assert.ok(containsCodeSnippet(source, from));const changed=replaceCodeSnippet(source, from,to);
   assert.throws(()=>assertNestedResultSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedNestedResultValues(row.id,2,0,'infer'),[[2,4,3,1,2,5],[4,5,4,2,3,5]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedNestedResultObservation(row.id,n,seed,'infer');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'infer'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'infer'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertNestedResultSource('));assert.ok(source.includes("'nested-function-results-oracle.mjs'"));
 }
});
