import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNestedOwnedSource} from './nested-function-owned-escape-source-contract.mjs';
import {expectedNestedOwnedValues,expectedNestedOwnedObservation} from './nested-function-owned-escape-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='nested_function_owned_escape');
test('Retained nested owners have matched sources, independent oracle and execution hooks',()=>{
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Result<Array<NumericArray<i64>>, TsrFunctionError0> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,true);
 assert.equal(row.measured,false);assert.equal(row.implementationAccepted,false);assert.equal(reportCategory(row.id),'callbacks');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertNestedOwnedSource(source,row,kind);
  assert.throws(()=>assertNestedOwnedSource(source+' ',row,kind));
  assert.throws(()=>assertNestedOwnedSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['const again = actions[0]();','const again = actions[1]();'],['const take = owner(value + 20);','const take = owner(value + 10);'],['caught = 1;','caught = 2;']]){
   assert.ok(source.includes(from));const changed=source.replace(from,to);
   assert.throws(()=>assertNestedOwnedSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedNestedOwnedValues(row.id,2,0,'owned'),[[2,12,3,1,2,21,1,5],[3,13,4,2,3,22,1,5]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedNestedOwnedObservation(row.id,n,seed,'owned');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'owned'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'owned'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertNestedOwnedSource(source'));assert.ok(source.includes("'nested-function-owned-escape-oracle.mjs'"));
 }
});
