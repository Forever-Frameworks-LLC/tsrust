import {containsCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';import vm from 'node:vm';import assert from 'node:assert/strict';import fs from 'node:fs';
import {isCopying,assertCopyingPolicy,validateCopyingObservation} from './copying-array-observation.mjs';import {assertCopyingSource,assertCopyingTSSource} from './copying-array-result-abi.mjs';import {validationProtocol} from './approximate-scalar.mjs';import {requiresExactResult} from './result-observation.mjs';import {adaptTSDriver} from './adapters.mjs';import {expectedChecksum,expectedExactResult} from './oracle.mjs';import {expectedCopyingObservation} from './copying-array-oracle.mjs';import {reportCategory} from './report.mjs';
const rows=JSON.parse(fs.readFileSync(new URL('copying-array-manifest-entries.json',import.meta.url)));
test('sixteen copying rows select closed exact complete-value policies',()=>{for(const row of rows){assertCopyingPolicy(row);assert.equal(validationProtocol(row),'legacy-exact');assert.equal(requiresExactResult(row),true);assert.throws(()=>validationProtocol({...row,result:'int'}));assert.throws(()=>validationProtocol({...row,approximateValidation:{}}));assert.throws(()=>validationProtocol({...row,id:'array_with_unknown'}));const expected=expectedCopyingObservation(row.id,2,7,row.inputText);assert.equal(expectedChecksum(row.id,2,7,row.inputText),expected.checksum);assert.deepEqual(expectedExactResult(row.id,2,7,row.inputText),expected);validateCopyingObservation(expected,expected,row);assert.throws(()=>validateCopyingObservation({...expected,checksum:expected.checksum+1},expected,row));}});
test('source bindings and natural prepared counterpart wrappers fail closed',()=>{for(const row of rows){const source=fs.readFileSync(new URL('workloads/'+row.id+'.tsr',import.meta.url),'utf8'),ts=fs.readFileSync(new URL('workloads/'+row.id+'.ts',import.meta.url),'utf8');assert.equal(reportCategory(row.id),'arrays');assertCopyingSource(source,row);assertCopyingTSSource(ts,row);assert.throws(()=>assertCopyingSource(source+' ',row));assert.throws(()=>assertCopyingTSSource(ts+' ',row));const driver=adaptTSDriver('',row);assert.ok(driver.includes('observeCopying(output'));assert.ok(driver.includes(row.inputText===undefined?'workload(iterations, seed)':'workload(iterations, seed, argv[4])'));}});

test('copying runtime preparation selects one source ABI guard',()=>{
 const runtime=fs.readFileSync(new URL('run.mjs',import.meta.url),'utf8');
 const start=runtime.indexOf('// Refuse emitter changes instead of silently timing the wrong function.');
 const end=runtime.indexOf(' if(isApproximate(w))',start);
 assert.ok(start>=0&&end>start);const actualProductionGuards=runtime.slice(start,end);
 for(const row of rows){
  const source=fs.readFileSync(new URL('workloads/'+row.id+'.tsr',import.meta.url),'utf8');
  const ts=fs.readFileSync(new URL('workloads/'+row.id+'.ts',import.meta.url),'utf8');
  const context={w:row,source,ts,isCopying,assertCopyingSource,assertCopyingTSSource,isRange:()=>false,isHigher:()=>false,isDeclared:()=>false,isConcat:()=>false,isReduction:()=>false};
  assert.doesNotThrow(()=>vm.runInNewContext(actualProductionGuards,{...context, containsCodeSnippet}),row.id);
  assert.throws(()=>vm.runInNewContext(actualProductionGuards,{containsCodeSnippet, ...context,source:source+' '}),row.id);
 }
});
