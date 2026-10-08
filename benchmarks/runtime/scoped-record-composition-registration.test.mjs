import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertScopedRecordCompositionSource} from './scoped-record-composition-source-contract.mjs';
import {expectedScopedRecordCompositionValues,expectedScopedRecordCompositionObservation} from './scoped-record-composition-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const row=manifest.workloads.find(x=>x.id==='scoped_record_composition');
test('Scoped record composition retain sources, independent observations and execution hooks',()=>{
 assert.equal(manifest.workloads.filter(x=>x.id==='scoped_record_composition').length,1);
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 assert.equal(row.measured,false);assert.equal(row.implementationAccepted,false);assert.equal(row.measurementStatus,'pending-scoped-aggregate-family');assert.equal(reportCategory(row.id),'callbacks');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertScopedRecordCompositionSource(source,row,kind);
  assert.throws(()=>assertScopedRecordCompositionSource(source+' ',row,kind));
  assert.throws(()=>assertScopedRecordCompositionSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['const removed = items.pop();',''],['boxes[0].handlers.clear();',''],['boxes[0].handlers = replacementMap;',''],['mapSecond = second.run();','mapSecond = 0;'],['iterated += holder.run();',''],['peerValue = peer.run();',''],['base[0] = 0;',''],['refillBox.handlers.clear();',''],['oldCapture[0] = value + 5;',''],['newCapture[0] = value + 6;',''],['const moved = refillBox.handlers;',''],['movedSize = moved.size;','']]){
   assert.ok(source.includes(from));const changed=replaceCodeSnippet(source, from, to);
   assert.throws(()=>assertScopedRecordCompositionSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedScopedRecordCompositionValues(row.id,2,0,'composition'),[[1,2,1,2,3,3,4,2,0,2,19,0],[2,3,2,3,4,5,5,2,0,3,22,0]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedScopedRecordCompositionObservation(row.id,n,seed,'composition');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'composition'),expected);assert.equal(expectedChecksum(row.id,n,seed,'composition'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertScopedRecordCompositionSource(source'));assert.ok(source.includes("'scoped-record-composition-oracle.mjs'"));
 }
});
