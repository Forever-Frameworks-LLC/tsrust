import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertRecursiveResultInferenceSource} from './recursive-result-inference-source-contract.mjs';
import {expectedRecursiveResultInferenceValues,expectedRecursiveResultInferenceObservation} from './recursive-result-inference-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const row=manifest.workloads.find(x=>x.id==='recursive_result_inference');
test('Recursive result inference retains sources, independent observations and execution hooks',()=>{
 assert.equal(manifest.workloads.filter(x=>x.id==='recursive_result_inference').length,1);
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 assert.equal(row.measured,false);assert.equal(row.implementationAccepted,true);assert.equal(reportCategory(row.id),'recursion');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertRecursiveResultInferenceSource(source,row,kind);
  assert.throws(()=>assertRecursiveResultInferenceSource(source+' ',row,kind));
  assert.throws(()=>assertRecursiveResultInferenceSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [["count * factorial(count - 1)", "count"], ["!parity(count - 1)", "false"], ["\"x\" + label(count - 1)", "\"x\""], ["1 + depth(count - 1)", "1"], ["fib(count - 1) + fib(count - 2)", "count"]]){
   assert.ok(source.includes(from),from);const changed=source.replace(from,to);
   assert.throws(()=>assertRecursiveResultInferenceSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind),from);
  }
 }
 const native=read('./'+row.source);
 assert.throws(()=>{const changed=native.replace('function parity(count: int) {','function parity(count: int): boolean {');assertRecursiveResultInferenceSource(changed,{...row,sourceSha256:createHash('sha256').update(changed).digest('hex')},'native');});
 assert.deepEqual(expectedRecursiveResultInferenceValues(row.id,3,0,'recursive'),[[1, 0, 1, 1, 2, 1], [2, 1, 2, 2, 3, 1], [6, 0, 3, 3, 4, 2]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedRecursiveResultInferenceObservation(row.id,n,seed,'recursive');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'recursive'),expected);assert.equal(expectedChecksum(row.id,n,seed,'recursive'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertRecursiveResultInferenceSource(source'));assert.ok(source.includes("'recursive-result-inference-oracle.mjs'"));
 }
});
