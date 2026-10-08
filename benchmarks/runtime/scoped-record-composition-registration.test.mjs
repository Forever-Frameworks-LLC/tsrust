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
 const fallible='fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<Array<NumericArray<i64>>,TsrFunctionError0<Record44>> { todo!() }\nfn main() {}';
 const binding=assertABI(fallible,row,{source:read('./'+row.source)});
 assert.equal(binding.fallible,true);assert.equal(binding.functionName,'tsr_result_function0');
 assert.throws(()=>assertABI(fallible,{...row,family:'unregistered-fallible-family'}));
 assert.equal(row.measured,false);assert.equal(row.implementationAccepted,false);assert.equal(row.measurementStatus,'pending-scoped-aggregate-family');assert.equal(reportCategory(row.id),'callbacks');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertScopedRecordCompositionSource(source,row,kind);
  assert.throws(()=>assertScopedRecordCompositionSource(source+' ',row,kind));
  assert.throws(()=>assertScopedRecordCompositionSource(source,{...row,operationsPerIteration:2},kind));
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  for(const [from,to] of [['const removed = items.pop();',''],['boxes[0].handlers.clear();',''],['boxes[0].handlers = replacementMap;',''],['mapSecond = second.run();','mapSecond = 0;'],['iterated += holder.run();',''],['peerValue = peer.run();',''],['base[0] = 0;',''],['refillBox.handlers.clear();',''],['oldCapture[0] = value + 5;',''],['newCapture[0] = value + 6;',''],['const moved = refillBox.handlers;',''],['movedSize = moved.size;',''],['throw value;',''],['caughtValue = caughtMap.size;',''],['caughtValue += current.run();',''],['beforeThrow[0] = value + 9;',''],['prefixValue = prefixResult(false);',''],['prefixValue = prefixResult(true);',''],['prefixValue += prefixMap.size;',''],['prefixValue += entry.run();',''],['normalRenewed = prefixValue + prefixOld[0] + prefixNew[0];',''],['caughtRenewed = prefixValue + prefixOld[0] + prefixNew[0];',''],['normalCleanup = prefixMap.size;',''],['caughtCleanup = prefixMap.size;',''],['branchTrace += 1;',''],['branchTrace += 2;',''],['const moved = consumedMap;',''],['consumedSize = moved.size;',''],['consumedOld[0] = value + 20;',''],['normalConsumed = consumedSize + consumedPath + consumedOld[0];',''],['caughtConsumed = consumedSize + consumedPath + consumedOld[0];',''],['const beforeTransfer = prefixResult(true);',''],['const beforeTransfer = prefixResult(false);',''],['const transferred = restoredMap;',''],['restoredSize = transferred.size;',''],['throw 7;',''],['restoredOld[0] = value + 30;',''],['earlyRestored = restoredSize + restoredMap.size + restoredOld[0];',''],['lateRestored = restoredSize + restoredMap.size + restoredOld[0];',''],['expressionMap.clear();',''],['expressionResult = (prefixResult(true), expressionSaved = expressionMap, 1);',''],['expressionResult = (expressionSaved = expressionMap, prefixResult(true));',''],['throw 8;',''],['expressionOld[0] = value + 40;',''],['expressionOld[0] = value + 41;',''],['earlyExpression = expressionResult + expressionSaved.size + expressionMap.size + expressionOld[0];',''],['lateExpression = expressionResult + expressionSaved.size + expressionMap.size + expressionOld[0];',''],['conditionalBox.handlers.clear();',''],['const conditionalMoved = conditionalBox.handlers;',''],['conditionalSize = conditionalMoved.size;',''],['conditionalSize = conditionalBox.handlers.size;',''],['throw 9;',''],['conditionalBox.handlers = new Map<string, Holder>();',''],['conditionalOld[0] = value + 50;',''],['conditionalRenewed = conditionalSize + conditionalBox.handlers.size + conditionalOld[0];',''],['const survivorPrefix = prefixResult(false);',''],['survivorMap.clear();',''],['const survivorMoved = survivorMap;',''],['survivorSize = survivorMoved.size;',''],['survivorSize = survivorMap.size;',''],['normalSurvivor = survivorSize + survivorOld[0];',''],['survivorOld[0] = value + 51;',''],['normalSurvivor += survivorOld[0];',''],['outsideMap.clear();',''],['const outsideMoved = outsideMap;',''],['outsideSize = outsideMoved.size;',''],['outsideSize = outsideMap.size;',''],['outsideOld[0] = value + 60;',''],['outsideRestored = outsideSize + outsideMap.size + outsideOld[0];','']]){
   assert.ok(source.includes(from));const changed=replaceCodeSnippet(source, from, to);
   assert.throws(()=>assertScopedRecordCompositionSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedScopedRecordCompositionValues(row.id,2,0,'composition'),[[1,2,1,2,3,3,4,2,0,2,19,0,31,43,45,1,1,4,22,23,30,31,40,41,52,54,62],[2,3,2,3,4,5,5,2,0,3,22,0,34,46,48,1,1,2,23,24,31,32,41,42,52,55,62]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedScopedRecordCompositionObservation(row.id,n,seed,'composition');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'composition'),expected);assert.equal(expectedChecksum(row.id,n,seed,'composition'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertScopedRecordCompositionSource(source'));assert.ok(source.includes("'scoped-record-composition-oracle.mjs'"));
 }
});
