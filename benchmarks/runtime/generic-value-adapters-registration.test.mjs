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
 const fallible='fn tsr_result_function2(value0: i64, value1: i64, value2: &String) -> Result<Array<NumericArray<i64>>,TsrFunctionError2<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(fallible,row,read('./'+row.source)).fallible,true);
 assert.throws(()=>assertABI(fallible.replace('TsrFunctionError2<i64>','UnknownError'),row,read('./'+row.source)));
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertGenericAdapterSource(source,row,kind);
  assert.throws(()=>assertGenericAdapterSource(source+' ',row,kind));
  assert.throws(()=>assertGenericAdapterSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['make(base + 190)','make(base + 191)'],['const absentValue = absent?.(base + skipped++);','const absentValue = absent?.(base + skipped);'],['(false ? scalarFactory : arrayFactory)()','(false ? scalarFactory : scalarFactory)()'],['callbackFactory(base + 180)', 'callbackFactory(base + 181)'], ['const callbackSecond = callback(increment, base + 33);', 'const callbackSecond = callback(increment, base + 34);'], ['return { value: action(value), calls: calls };', 'return { value: value, calls: calls };'], ['function main() {}',''],['const freshHolder: Holder = { run: make(base + 160) };','const freshHolder: Holder = { run: make(base + 161) };'],['const freshStored = freshActions[0]("fresh");','const freshStored = actions[0]("fresh");'],['calls += 2;','calls += 3;'],['(selections++ === 0 ? make : doubleFactory)(base + 100 + evaluations++)','(selections++ === 0 ? make : make)(base + 100 + evaluations++)'],['selections, evaluations,','0, 0,'],['nestedSelections++ === 1 ? make : doubleFactory','nestedSelections++ === 1 ? doubleFactory : make'],['nestedSelections, nestedEvaluations,','0, 0,'],['if (calls === start + 1) { return left; }','if (calls === start + 1) { return right; }'],['inferredFactory(base + 150)','inferredFactory(base + 151)'],['calls += 1;',''],['= action;','= make(base);'],['const mapper = make(base + 10);','const mapper = make(base);'],['mapped[1].calls','mapped[0].calls'],['return calls === 4 ? right : left;','return left;'],['return calls === 2;','return true;'],['seen = 1;','seen = 0;'],['sortable.sort(compare);',''],['if (typeof seed === "string") { calls += 1; }','if (typeof seed === "string") { calls += 2; }'],['.map(genericFactory(base, base + 90))','.map(genericFactory("string", base + 90))'],['const aliasValue = alias(base + 9);','const aliasValue = factory(base + 61)(base + 9);'],['(false ? factory : factory)(base + 70)','(false ? factory : factory)(base + 71)'],['const directSecond = direct(base + 6);','const directSecond = make(base + 40)(base + 6);'],['.map(make(base + 50))','.map(make(base + 51))'],['offset: '+(kind==='native'?'int':'number')+' = next()','offset: '+(kind==='native'?'int':'number')+' = 0'],['undefinedCalls = view(base, undefined).calls;','undefinedCalls = view(base, 0).calls;'],['const restAfter = rest("after", "one");','const restAfter = restFactory()("after", "one");'],['const reducedAfter = reduce("before", "fourth");','const reducedAfter = reducer()("before", "fourth");'],['const sortedAfter = compare("after", "unused");','const sortedAfter = comparator()("after", "unused");']]){
   assert.ok(source.includes(from));const changed=source.replace(from,to);
   assert.throws(()=>assertGenericAdapterSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedGenericAdapterValues(row.id,2,0,'adapters'),[[1,2,7,3,5,4,1,2,12,13,1,14,3,22,5,32,8,4,6,5,6,1,0,3,3,1,1,1,9,2,3,5,1,1,4,4,5,6,42,7,43,8,9,52,53,10,62,12,13,72,73,14,83,85,16,17,92,93,18,102,103,20,21,114,116,2,2,22,122,123,24,25,134,136,4,2,26,29,30,1,32,162,5,172,34,182,35,183,36,192,1,38,39],[2,3,7,4,5,5,2,3,13,14,1,15,4,23,5,33,8,5,6,6,6,1,0,4,4,1,2,1,9,2,3,5,2,1,4,4,5,7,43,8,44,9,10,53,54,11,63,13,14,73,74,15,84,86,17,18,93,94,19,103,104,21,22,115,117,2,2,23,123,124,25,26,135,137,4,2,27,30,31,1,33,163,5,173,35,183,36,184,37,193,1,39,40]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedGenericAdapterObservation(row.id,n,seed,'adapters');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'adapters'),expected);
  assert.equal(expectedChecksum(row.id,n,seed,'adapters'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertGenericAdapterSource('));assert.ok(source.includes("'generic-value-adapters-oracle.mjs'"));
 }
});
