import {replaceCodeSnippet} from './source-contract-tokens.mjs';
// Prepared only; execution deferred until explicit light-validation grant.
import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';import {expectedForEach,expectedVisitIndices,explicitExpected} from './for-each-oracle.mjs';
for(const cell of explicitExpected)test('manual '+JSON.stringify([cell.id,cell.n,cell.seed]),()=>assert.equal(expectedForEach(cell.id,cell.n,cell.seed),cell.value));
test('all source recipes/bounds and untouched scalar output domain',()=>{const entries=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url))).workloads.filter(x=>x.id.startsWith('array_for_each_'));for(const entry of entries){assert.equal(entry.category,'Arrays');for(const n of [0,1,2,17])for(const seed of [0,1,2,7,1000000])assert.equal(typeof expectedForEach(entry.id,n,seed,entry.inputText),'number');}for(const args of [['bad',1,1],['array_for_each_int',-1,1],['array_for_each_int',1.5,1],['array_for_each_int',1,-1],['array_for_each_string',1,1,'\ud800']])assert.throws(()=>expectedForEach(...args));});
test('ascending original positions empty singleton and multi',()=>assert.deepEqual(expectedVisitIndices(5,0),[[],[0],[0,1,2,3],[0,1,2,3,4],[0,1,2,3,4]]));
// Future actual-source adapters must equal independent scalar results for every
// engine, plus exact Number bits. Actual receiver/callback stdout must equal the
// checked-in fixture, including empty and singleton callback effects.

import {assertForEachSource,adaptTSDriver,assertABI} from './adapters.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {observeNumber,validateNumberObservation} from './number-observation.mjs';
import {reportCategory} from './report.mjs';
test('forEach scalar protocols, compact source guard and Arrays categories',()=>{
 const rows=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url))).workloads.filter(x=>x.id.startsWith('array_for_each_'));assert.equal(rows.length,16);
 for(const w of rows){const source=fs.readFileSync(new URL('workloads/'+w.id+'.tsr',import.meta.url),'utf8');assertForEachSource(source,w);assert.equal(reportCategory(w.id),'arrays');const value=expectedForEach(w.id,2,7,w.inputText);if(w.result==='number'){assert.equal(w.resultEncoding,'ieee-f64-bits-v1');validateNumberObservation(observeNumber(value),expectedExactResult(w.id,2,7));assert.equal(expectedChecksum(w.id,2,7),observeNumber(value).checksum);}else assert.equal(expectedChecksum(w.id,2,7,w.inputText),value);
 const wrongParameter=replaceCodeSnippet(source, 'function workload(n:int', 'function workload(n:number');assert.notEqual(wrongParameter,source);assert.throws(()=>assertForEachSource(wrongParameter,w));assert.throws(()=>assertForEachSource(source+'\nfunction workload(n:int,seed:int):int{}',w));
 const prepared=adaptTSDriver(fs.readFileSync(new URL('driver.ts',import.meta.url),'utf8'),w);assert.ok(prepared.includes('workload(iterations, seed'+(w.inputText===undefined?'':', argv[4]')+')'));
 const result=w.result==='number'?'f64':'i64';const emitted='fn function0(value0: i64, value1: i64'+(w.inputText===undefined?'':', value2: &String')+') -> '+result+' {}\nfn main() {}';assertABI(emitted,w);assert.throws(()=>assertABI(emitted.replace('value1: i64','value1: f64'),w));
 }
 assert.throws(()=>assertForEachSource('function workload(n:int,seed:int):int{}',{id:'array_for_each_unknown',result:'int'}));
});
