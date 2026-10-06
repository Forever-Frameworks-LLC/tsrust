// SOURCE ONLY. Prepared synthetic harness checks; no actual native ABI acceptance.
import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {expectedHigherObservation} from './higher-order-oracle.mjs';
import {assertHigherPolicy,validateHigherObservation} from './higher-order-observation.mjs';
import {validationProtocol} from './approximate-scalar.mjs';
import {requiresExactResult} from './result-observation.mjs';
import {adaptTSDriver} from './adapters.mjs';import {reportCategory} from './report.mjs';
const rows=JSON.parse(fs.readFileSync(new URL('higher-order-manifest-entries.json',import.meta.url)));
test('all24 higher rows select closed exact protocols and independent natural oracle',()=>{for(const row of rows){assertHigherPolicy(row);assert.equal(validationProtocol(row),'legacy-exact');assert.equal(reportCategory(row.id),'callbacks');assert.equal(requiresExactResult(row),row.result!=='int');const expected=expectedHigherObservation(row.id,2,7,row.inputText);assert.equal(expectedChecksum(row.id,2,7,row.inputText),expected.checksum);assert.deepEqual(expectedExactResult(row.id,2,7,row.inputText),expected);validateHigherObservation(expected,expected,row);assert.throws(()=>validateHigherObservation({...expected,checksum:expected.checksum+1},expected,row));assert.throws(()=>validationProtocol({...row,id:'higher_unknown'}));assert.throws(()=>validationProtocol({...row,approximateValidation:{protocol:'mpfr-directed-f64-scalar-v1'}}));}});
test('natural TS wrappers include optional data input without changing source kernels',()=>{const base=fs.readFileSync(new URL('driver.ts',import.meta.url),'utf8');for(const row of rows){const adapted=adaptTSDriver(base,row);assert.ok(adapted.includes(row.inputText===undefined?'workload(iterations, seed)':'workload(iterations, seed, argv[4])'));if(row.result==='boolean'||row.result.endsWith('[]'))assert.ok(adapted.includes('observeDeclared'));if(row.result==='number')assert.ok(adapted.includes('observeNumber'));}});
