import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertScopedCallableRecordsSource} from './scoped-callable-records-source-contract.mjs';
import {expectedScopedCallableRecordValues,expectedScopedCallableRecordObservation} from './scoped-callable-records-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const row=manifest.workloads.find(x=>x.id==='scoped_callable_records');
test('Scoped callable records retain sources, independent observations and execution hooks',()=>{
 assert.equal(manifest.workloads.at(-1).id,'scoped_callable_records');
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'callbacks');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertScopedCallableRecordsSource(source,row,kind);
  assert.throws(()=>assertScopedCallableRecordsSource(source+' ',row,kind));
  assert.throws(()=>assertScopedCallableRecordsSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['replace(holder, value + 7);',''],['const run = holder.run;','const run = (): int => 0;'],['stepped = step.next();','stepped = count;'],['base[0] = 0;','']]){
   assert.ok(source.includes(from));const changed=replaceCodeSnippet(source, from, to);
   assert.throws(()=>assertScopedCallableRecordsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedScopedCallableRecordValues(row.id,2,0,'records'),[[1,2,3,8,4,2,2],[2,4,5,9,6,4,3]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedScopedCallableRecordObservation(row.id,n,seed,'records');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'records'),expected);assert.equal(expectedChecksum(row.id,n,seed,'records'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertScopedCallableRecordsSource(source'));assert.ok(source.includes("'scoped-callable-records-oracle.mjs'"));
 }
});
