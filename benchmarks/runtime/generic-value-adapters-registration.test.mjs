import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertGenericAdapterSource} from './generic-value-adapters-source-contract.mjs';
import {expectedGenericAdapterValues,expectedGenericAdapterObservation} from './generic-value-adapters-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(row=>row.id==='generic_value_adapters');
test('Existing generic environment adapters retain matched sources and independent result evidence',()=>{
 assert.equal(row.id,'generic_value_adapters');
 assert.equal(row.implementationAccepted,false);assert.equal(row.measured,false);
 assert.equal(reportCategory(row.id),'callbacks');
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertGenericAdapterSource(source,row,kind);
  assert.throws(()=>assertGenericAdapterSource(source+' ',row,kind));
  assert.throws(()=>assertGenericAdapterSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['function main() {}',''],['calls += 1;',''],['= action;','= make(base);'],['const mapper = make(base + 10);','const mapper = make(base);'],['mapped[1].calls','mapped[0].calls']]){
   assert.ok(source.includes(from));const changed=source.replace(from,to);
   assert.throws(()=>assertGenericAdapterSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedGenericAdapterValues(row.id,2,0,'adapters'),[[1,2,7,3,5,4,1,2,12,13,1,14,3,22,5,32,8],[2,3,7,4,5,5,2,3,13,14,1,15,4,23,5,33,8]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedGenericAdapterObservation(row.id,n,seed,'adapters');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'adapters'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'adapters'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertGenericAdapterSource('));assert.ok(source.includes("'generic-value-adapters-oracle.mjs'"));
 }
});
