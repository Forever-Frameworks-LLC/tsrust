import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertOwnedAssignmentValuesSource} from './owned-assignment-values-source-contract.mjs';
import {expectedOwnedAssignmentValuesValues,expectedOwnedAssignmentValuesObservation} from './owned-assignment-values-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const row=manifest.workloads.find(x=>x.id==='owned_assignment_values');
test('Owned assignment values retains sources, independent observations and execution hooks',()=>{
 assert.equal(manifest.workloads.filter(x=>x.id==='owned_assignment_values').length,1);
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 assert.equal(row.measured,false);assert.equal(row.implementationAccepted,true);assert.equal(reportCategory(row.id),'strings');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertOwnedAssignmentValuesSource(source,row,kind);
  assert.throws(()=>assertOwnedAssignmentValuesSource(source+' ',row,kind));
  assert.throws(()=>assertOwnedAssignmentValuesSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [["(text = \"x\".repeat(value)).length", "\"x\".repeat(value).length"], ["size(text = \"four\" + value)", "size(\"four\" + value)"], ["(values = [value, value + 1, value + 2]).length", "3"], ["}).x", "}, point).x"], ["(point.label = \"q\".repeat(value + 1)).length", "value + 1"], ["(current = rounds < value ? \"go\" : \"\") !== \"\"", "rounds < value"]]){
   assert.ok(source.includes(from),from);const changed=source.replace(from,to);
   assert.throws(()=>assertOwnedAssignmentValuesSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind),from);
  }
 }
 assert.deepEqual(expectedOwnedAssignmentValuesValues(row.id,3,0,'assignment'),[[1, 5, 3, 3, 2, 2, 2, 1, 5], [2, 5, 3, 4, 4, 3, 3, 2, 5], [3, 5, 3, 5, 6, 4, 4, 3, 5]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedOwnedAssignmentValuesObservation(row.id,n,seed,'assignment');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'assignment'),expected);assert.equal(expectedChecksum(row.id,n,seed,'assignment'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertOwnedAssignmentValuesSource(source'));assert.ok(source.includes("'owned-assignment-values-oracle.mjs'"));
 }
});
