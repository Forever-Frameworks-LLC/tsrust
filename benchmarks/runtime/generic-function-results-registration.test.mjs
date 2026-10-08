import {containsCodeSnippet, replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertGenericResultSource} from './generic-function-results-source-contract.mjs';
import {expectedGenericResultValues,expectedGenericResultObservation} from './generic-function-results-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(row=>row.id==='generic_function_results');
test('Result-context generic values retain matched sources and independent result evidence',()=>{
 assert.equal(row.id,'generic_function_results');
 assert.equal(row.implementationAccepted,true);assert.equal(row.measured,true);assert.equal(row.measurementStatus,'measured-named-declaration-family');
 assert.equal(reportCategory(row.id),'callbacks');
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertGenericResultSource(source,row,kind);
  assert.throws(()=>assertGenericResultSource(source+' ',row,kind));
  assert.throws(()=>assertGenericResultSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['function main() {}',''],['function fresh<T>(): T[] { calls += 1; return []; }','function fresh<T>(): T[] { return []; }'],['ownedNumbers.push(base + 1);','ownedNumbers.push(base);'],['ownedStrings.push("owner");','ownedStrings.push("different");']]){
   assert.ok(containsCodeSnippet(source, from));const changed=replaceCodeSnippet(source, from,to);
   assert.throws(()=>assertGenericResultSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedGenericResultValues(row.id,2,0,'results'),[[1,5,2,5,2,7],[2,5,3,5,2,7]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedGenericResultObservation(row.id,n,seed,'results');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'results'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'results'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertGenericResultSource('));assert.ok(source.includes("'generic-function-results-oracle.mjs'"));
 }
});
