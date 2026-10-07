import {containsCodeSnippet, replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertCapturedGenericSource} from './generic-function-captures-source-contract.mjs';
import {expectedCapturedGenericValues,expectedCapturedGenericObservation} from './generic-function-captures-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(row=>row.id==='generic_function_captures');
test('Captured generic values retain matched sources and independent result evidence',()=>{
 assert.equal(row.id,'generic_function_captures');
 assert.equal(row.implementationAccepted,false);assert.equal(row.measured,false);
 assert.equal(reportCategory(row.id),'callbacks');
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertCapturedGenericSource(source,row,kind);
  assert.throws(()=>assertCapturedGenericSource(source+' ',row,kind));
  assert.throws(()=>assertCapturedGenericSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['function main() {}',''],['function select<T>(value: T): T { local += 1; return value; }','function select<T>(value: T): T { return value; }'],['const second = counter(base + 10);','const second = counter(base);'],['const sharedCount = first([base, base + 1]);','const sharedCount = second([base, base + 1]);'],['const finalCount = second(base);','const finalCount = first(base);']]){
   assert.ok(containsCodeSnippet(source, from));const changed=replaceCodeSnippet(source, from,to);
   assert.throws(()=>assertCapturedGenericSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedCapturedGenericValues(row.id,2,0,'captures'),[[2,12,3,1,5,4,3,13,8],[3,13,4,2,5,5,3,14,8]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedCapturedGenericObservation(row.id,n,seed,'captures');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'captures'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'captures'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertCapturedGenericSource('));assert.ok(source.includes("'generic-function-captures-oracle.mjs'"));
 }
});
