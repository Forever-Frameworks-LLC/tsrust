import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertCallerBorrowedRecordStoresSource} from './caller-borrowed-record-stores-source-contract.mjs';
import {expectedCallerBorrowedRecordStoreValues,expectedCallerBorrowedRecordStoreObservation} from './caller-borrowed-record-stores-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=path=>fs.readFileSync(new URL(path,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const row=manifest.workloads.find(row=>row.id==='caller_borrowed_record_stores');

test('Caller-borrowed stores register independent observations and complete execution hooks',()=>{
 assert.equal(manifest.workloads.filter(row=>row.id==='caller_borrowed_record_stores').length,1);
 assert.equal(row.measured,false);
 assert.equal(row.implementationAccepted,false);
 assert.equal(row.measurementStatus,'pending-caller-borrow-family');
 assert.equal(reportCategory(row.id),'callbacks');
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 const fallible='fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<Array<NumericArray<i64>>,TsrFunctionError0<Record44>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(fallible,row,{source:read('./'+row.source)}).fallible,true);
 assert.throws(()=>assertABI(fallible,{...row,family:'unregistered-fallible-family'}));
 for(const [kind,path,key] of [['native',row.source,'sourceSha256'],['typescript',row.counterpart,'counterpartSha256']]) {
  const source=read('./'+path);
  assertCallerBorrowedRecordStoresSource(source,row,kind);
  assert.throws(()=>assertCallerBorrowedRecordStoresSource(source+' ',row,kind));
  assert.throws(()=>assertCallerBorrowedRecordStoresSource(source,{...row,operationsPerIteration:2},kind));
  for(const [from,to] of [
   ['const alias = source;','const alias = [0];'],
   ['installArray(holder, right);',''],
   ['holder.run = (): int => left() + right();','holder.run = (): int => left();'],
   ['base[0] += 1;',''],
   ['changedSecond = holder.run();',''],
   ['installDirect(callbackHolder, constant);',''],
   ['installPair(pairHolder, constant, action);',''],
   ['installMutableCallback(holder, secondAction);',''],
   ['holder.run = action;', 'holder.run = (): int => 0;'],
   ['const readCallback = (): int => callbackHolder.run();','const readCallback = (): int => 0;'],
   ['const invoke = (): int => holder.run();','const invoke = (): int => 0;']
  ]) {
   const changed=replaceCodeSnippet(source,from,to);
   assert.notEqual(changed,source);
   assert.throws(()=>assertCallerBorrowedRecordStoresSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedCallerBorrowedRecordStoreValues(row.id,2,0,row.inputText),[[1,3,4,1,10,3,4,5,6,7,7,9,10,13],[2,4,6,2,11,4,5,6,7,8,8,9,11,14]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]) {
  const expected=expectedCallerBorrowedRecordStoreObservation(row.id,n,seed,row.inputText);
  assert.deepEqual(expectedExactResult(row.id,n,seed,row.inputText),expected);
  assert.equal(expectedChecksum(row.id,n,seed,row.inputText),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']) {
  const source=read(path);
  assert.ok(source.includes('assertCallerBorrowedRecordStoresSource(source'));
  assert.ok(source.includes("'caller-borrowed-record-stores-oracle.mjs'"));
 }
});
