import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNestedDefaultSource} from './nested-function-defaults-source-contract.mjs';
import {expectedNestedDefaultValues,expectedNestedDefaultObservation} from './nested-function-defaults-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(row=>row.id==='nested_function_defaults');
test('Captured default values retain matched sources and independent result evidence',()=>{
 assert.equal(row.id,'nested_function_defaults');
 assert.equal(row.implementationAccepted,false);assert.equal(row.measured,false);
 assert.equal(reportCategory(row.id),'callbacks');
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertNestedDefaultSource(source,row,kind);
  assert.throws(()=>assertNestedDefaultSource(source+' ',row,kind));
  assert.throws(()=>assertNestedDefaultSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['second = action(3);','second = action(4);'],['hits += 1; return 2;','hits += 1; return 3;'],['third = action(undefined);','third = action(0);']]){
   assert.ok(source.includes(from));const changed=source.replace(from,to);
   assert.throws(()=>assertNestedDefaultSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedNestedDefaultValues(row.id,2,0,'defaults'),[[3,6,8,2,5,2,5,8],[4,7,9,2,6,3,5,8]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedNestedDefaultObservation(row.id,n,seed,'defaults');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'defaults'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'defaults'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertNestedDefaultSource('));assert.ok(source.includes("'nested-function-defaults-oracle.mjs'"));
 }
});
