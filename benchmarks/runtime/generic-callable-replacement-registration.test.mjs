import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertGenericReplacementSource} from './generic-callable-replacement-source-contract.mjs';
import {expectedGenericReplacementValues,expectedGenericReplacementObservation} from './generic-callable-replacement-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=path=>fs.readFileSync(new URL(path,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const row=manifest.workloads.find(row=>row.id==='generic_callable_replacement');
test('Generic replacement retains source scenarios, oracle and execution registrations',()=>{
 assert.equal(manifest.workloads.at(-1).id,row.id);
 assert.equal(reportCategory(row.id),'callbacks');assert.equal(row.measured,false);
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 const fallible='fn tsr_result_function1(value0: i64, value1: i64, value2: &String) -> Result<Array<NumericArray<i64>>,TsrFunctionError1<Record0>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(fallible,row,read('./'+row.source)).fallible,true);
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertGenericReplacementSource(source,row,kind);
  assert.throws(()=>assertGenericReplacementSource(source+' ',row,kind));
  assert.throws(()=>assertGenericReplacementSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const snippet of ['const saved = action;','action = second;','action = failing;','const completed = attempt(flag, base);','action = secondDefault;','action = ownedTail;']){
   const changed=replaceCodeSnippet(source,snippet,'');
   assert.throws(()=>assertGenericReplacementSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedGenericReplacementValues(row.id,3,0,'replacement'),[[1,1,1,3,4,4,4,11,2],[2,9,11,7,10,8,0,9,11],[3,3,3,3,4,4,4,3,6]]);
 for(const n of [0,1,37,2048])for(const seed of [0,7,1000000]){
  const expected=expectedGenericReplacementObservation(row.id,n,seed,'replacement');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'replacement'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'replacement'),expected.checksum);
 }
 assert.throws(()=>expectedGenericReplacementValues(row.id,2049,0,'replacement'));
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertGenericReplacementSource(source'));assert.ok(source.includes("'generic-callable-replacement-oracle.mjs'"));
 }
});
