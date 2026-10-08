import {containsCodeSnippet, replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNestedRestSource} from './nested-function-rest-source-contract.mjs';
import {expectedNestedRestValues,expectedNestedRestObservation} from './nested-function-rest-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(row=>row.id==='nested_function_rest');
test('Captured rest values retain matched sources and independent result evidence',()=>{
 assert.equal(row.id,'nested_function_rest');
 assert.equal(row.implementationAccepted,true);assert.equal(row.measured,true);assert.equal(row.measurementStatus,'measured-named-declaration-family');
 assert.equal(reportCategory(row.id),'callbacks');
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertNestedRestSource(source,row,kind);
  assert.throws(()=>assertNestedRestSource(source+' ',row,kind));
  assert.throws(()=>assertNestedRestSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['supplied = action(2, 3);','supplied = action(2, 4);'],['const pack = [4, 5];','const pack = [4, 6];'],['undefinedFirst = prefix(undefined, 1, 2);','undefinedFirst = prefix(0, 1, 2);']]){
   assert.ok(containsCodeSnippet(source, from));const changed=replaceCodeSnippet(source, from,to);
   assert.throws(()=>assertNestedRestSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedNestedRestValues(row.id,2,0,'rest'),[[1,6,10,1,3,4,4],[2,7,11,2,4,4,4]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedNestedRestObservation(row.id,n,seed,'rest');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'rest'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'rest'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertNestedRestSource('));assert.ok(source.includes("'nested-function-rest-oracle.mjs'"));
 }
});
