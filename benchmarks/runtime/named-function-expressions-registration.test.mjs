import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNamedFunctionExpressionsSource} from './named-function-expressions-source-contract.mjs';
import {expectedNamedFunctionExpressionValues,expectedNamedFunctionExpressionObservation} from './named-function-expressions-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const row=manifest.workloads.find(x=>x.id==='named_function_expressions');
test('Named function expressions retain sources, independent observations and execution hooks',()=>{
 assert.equal(manifest.workloads.filter(x=>x.id==='named_function_expressions').length,1);
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 assert.equal(row.measured,false);assert.equal(row.implementationAccepted,true);assert.equal(reportCategory(row.id),'callbacks');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertNamedFunctionExpressionsSource(source,row,kind);
  assert.throws(()=>assertNamedFunctionExpressionsSource(source+' ',row,kind));
  assert.throws(()=>assertNamedFunctionExpressionsSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['count * fact(count - 1)','count * 2'],['function down(depth: int)','function (depth: int)'],['down(depth - 1);',''],['function shift(item: int)','function (item: int)'],['1 + add(count - 1)','1'],['function choose<T>','function <T>']]){
   assert.ok(source.includes(from),from);const changed=source.replace(from,to);
   assert.throws(()=>assertNamedFunctionExpressionsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind),from);
  }
 }
 assert.deepEqual(expectedNamedFunctionExpressionValues(row.id,3,0,'named'),[[1,9,3,4,3,1,1],[2,12,4,5,4,2,1],[6,15,5,6,5,3,1]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedNamedFunctionExpressionObservation(row.id,n,seed,'named');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'named'),expected);assert.equal(expectedChecksum(row.id,n,seed,'named'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertNamedFunctionExpressionsSource(source'));assert.ok(source.includes("'named-function-expressions-oracle.mjs'"));
 }
});
