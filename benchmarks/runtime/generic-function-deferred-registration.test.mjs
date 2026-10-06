import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertDeferredGenericSource} from './generic-function-deferred-source-contract.mjs';
import {expectedDeferredGenericValues,expectedDeferredGenericObservation} from './generic-function-deferred-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(row=>row.id==='generic_function_deferred');
test('Deferred generic values retain matched sources and independent result evidence',()=>{
 assert.equal(row.id,'generic_function_deferred');
 assert.equal(row.implementationAccepted,false);assert.equal(row.measured,false);
 assert.equal(reportCategory(row.id),'callbacks');
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertDeferredGenericSource(source,row,kind);
  assert.throws(()=>assertDeferredGenericSource(source+' ',row,kind));
  assert.throws(()=>assertDeferredGenericSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['function identity<T>(value: T): T { return value; }','function identity<T>(value: T): T { return structuredClone(value); }'],['const action = identity;','const action = offset;'],['const numberResult = action(base);','const numberResult = action(base + 1);'],['const headResult = pick(base + 2, base + 3);','const headResult = pick(base, base + 3);']]){
   assert.ok(source.includes(from));const changed=source.replace(from,to);
   assert.throws(()=>assertDeferredGenericSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedDeferredGenericValues(row.id,2,0,'deferred'),[[1,5,2,3,5,2,8],[2,5,3,4,6,2,8]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedDeferredGenericObservation(row.id,n,seed,'deferred');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'deferred'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'deferred'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertDeferredGenericSource('));assert.ok(source.includes("'generic-function-deferred-oracle.mjs'"));
 }
});
