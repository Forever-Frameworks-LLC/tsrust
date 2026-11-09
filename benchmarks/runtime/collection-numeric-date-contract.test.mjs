import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {adaptRustDriver} from './adapters.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {completionSpecs,completionChecksum,assertCompletionSource,assertCompletionReference,completionReferenceOperations} from './collection-numeric-date-contract.mjs';
import {expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url),'utf8'));
test('B14 workloads retain all observed domains and unchanged timing protocol',()=>{
 assert.equal(Object.keys(completionSpecs).length,7);assert.equal(manifest.warmups,8);assert.equal(manifest.samples,31);assert.equal(manifest.processes,3);
 const first={collection_endpoints:43,ordered_set_snapshots:294,deque_values_snapshot:43,heap_sorted_snapshot:472,numeric_strict_parse:128,numeric_checked_arithmetic:99,date_completion_access:123456940};
 for(const id of Object.keys(completionSpecs)){
  const row=manifest.workloads.find(row=>row.id===id);assert(row);assert.equal(row.measured,false);assert.equal(completionChecksum(id,0,7),7);assert.equal(completionChecksum(id,1,7),first[id]);
  for(const [n,seed] of [[0,0],[1,7],[2,19],[17,3],[2048,7]])assert.equal(expectedChecksum(id,n,seed),completionChecksum(id,n,seed));
  for(const [kind,key]of [['native','source'],['typescript','counterpart']]){
   const source=fs.readFileSync(new URL(row[key],import.meta.url),'utf8');assertCompletionSource(source,row,kind);
   assert.throws(()=>assertCompletionSource(source.replace('checksum','missing'),row,kind));assert.throws(()=>assertCompletionSource(source,{...row,iterations:2047},kind));
  }
  const reference=fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8');assert.match(reference,/fn function0\(value0: i64, value1: i64\) -> i64/);
 }
});
test('B14 scalar ABI supports checked errors but rejects mismatched output',()=>{
 const row=manifest.workloads.find(row=>row.id==='numeric_strict_parse');
 assertABI('fn function0(value0: i64, value1: i64) -> Result<i64,TsrFunctionError0> { unreachable!() } fn main(){}',row);
 assert.throws(()=>assertABI('fn function0(value0: i64, value1: i64) -> Result<f64,TsrFunctionError0> { unreachable!() } fn main(){}',row));
});

test('B14 operation guards reject semantic mutations with refreshed source identities',()=>{
 const hash=source=>createHash('sha256').update(source).digest('hex');
 const driver=fs.readFileSync(new URL('driver.rs',import.meta.url),'utf8');
 for(const id of Object.keys(completionSpecs)){
  const row=manifest.workloads.find(row=>row.id===id);
  for(const [kind,key,fingerprint]of [['native','source','sourceSha256'],['typescript','counterpart','counterpartSha256']]){
   const source=fs.readFileSync(new URL(row[key],import.meta.url),'utf8');
   const changed=replaceCodeSnippet(source,completionSpecs[id][kind],'omitted_operation');assert.notEqual(changed,source);
   assert.throws(()=>assertCompletionSource(changed,{...row,[fingerprint]:hash(changed)},kind));
  }
  const source=fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8');assertCompletionReference(source,row);
  adaptRustDriver(driver,row,{emitted:source,reference:true});
  assert.throws(()=>assertCompletionReference(source+' ',row));
  // Each independently observed Rust backend operation remains necessary even when a reviewed hash changes.
  for(const operation of completionReferenceOperations[id]){
   const changed=replaceCodeSnippet(source,operation,'omitted_operation');assert.notEqual(changed,source,operation);
   const altered={...row,rustReferenceSha256:hash(changed)};assert.throws(()=>assertCompletionReference(changed,altered));
   assert.throws(()=>adaptRustDriver(driver,altered,{emitted:changed,reference:true}));
  }
 }
 const row=manifest.workloads.find(row=>row.id==='date_completion_access');const source=fs.readFileSync(new URL(row.counterpart,import.meta.url),'utf8');
 const changed=source.replace('nanoseconds:Number(matched[4])','nanoseconds:123456789');assert.notEqual(changed,source);
 assert.throws(()=>assertCompletionSource(changed,{...row,counterpartSha256:hash(changed)},'typescript'));
});
