import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertFallibleRecordCallbacksSource} from './fallible-record-callbacks-source-contract.mjs';
import {expectedFallibleRecordCallbacksValues,expectedFallibleRecordCallbacksObservation} from './fallible-record-callbacks-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const row=manifest.workloads.find(x=>x.id==='fallible_record_callbacks');
test('Fallible record callbacks retains sources, independent observations and execution hooks',()=>{
 assert.equal(manifest.workloads.filter(x=>x.id==='fallible_record_callbacks').length,1);
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 assert.equal(row.measured,false);assert.equal(row.implementationAccepted,true);assert.equal(reportCategory(row.id),'callbacks');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertFallibleRecordCallbacksSource(source,row,kind);
  assert.throws(()=>assertFallibleRecordCallbacksSource(source+' ',row,kind));
  assert.throws(()=>assertFallibleRecordCallbacksSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [["throw \"x\".repeat(value);", "return value;"], ["throw value;", "return value;"], ["throw \"big\";", "return limit;"], ["checked(plain, 2);", "call(plain);"], ["typeof error === \"number\" ? error + 1000 : 0", "0"]]){
   assert.ok(source.includes(from),from);const changed=source.replace(from,to);
   assert.throws(()=>assertFallibleRecordCallbacksSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind),from);
  }
 }
 assert.deepEqual(expectedFallibleRecordCallbacksValues(row.id,3,0,'fallible'),[[1, 1, 1, 1001, 1], [2, 2, 2, 1002, 1], [3, 3, 3, 1003, 3]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedFallibleRecordCallbacksObservation(row.id,n,seed,'fallible');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'fallible'),expected);assert.equal(expectedChecksum(row.id,n,seed,'fallible'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertFallibleRecordCallbacksSource(source'));assert.ok(source.includes("'fallible-record-callbacks-oracle.mjs'"));
 }
});
