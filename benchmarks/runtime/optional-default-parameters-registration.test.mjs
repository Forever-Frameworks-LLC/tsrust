import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';import {createHash} from 'node:crypto';
import {assertOptionalDefaultParameterSource} from './optional-default-parameters-source-contract.mjs';
import {optionalDefaultParameterIds,expectedOptionalDefaultParameterObservation} from './optional-default-parameters-oracle.mjs';
import {hands} from './optional-default-parameter-hands.mjs';import {observeSliceTree} from './slice-observation.mjs';import {expectedChecksum,expectedExactResult} from './oracle.mjs';import {reportCategory} from './report.mjs';
const read=path=>fs.readFileSync(new URL(path,import.meta.url),'utf8'),manifest=JSON.parse(read('./manifest.json')),rows=manifest.workloads.filter(row=>row.family==='optional-default-parameters');
const oldPrefix='dfa08413ff0d79889c220e165720c5d7dd11ceb7c553a324409c94c3852eef52';
test('Gate98 registration preserves earlier rows and binds actual optional/default scenario sources',()=>{
 assert.equal(createHash('sha256').update(JSON.stringify(manifest.workloads.slice(0,691))).digest('hex'),oldPrefix);assert.ok(manifest.gate>=98);assert.deepEqual(rows.map(row=>row.id),optionalDefaultParameterIds);assert.equal(rows.length,13);
 for(const row of rows){assert.equal(row.measured,false);assert.equal(row.pendingCheckpoint,101);assert.equal(reportCategory(row.id),'callbacks');for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){const source=read('./'+name);assertOptionalDefaultParameterSource(source,row,kind);assert.throws(()=>assertOptionalDefaultParameterSource(source.replace('output.push(','void('),row,kind));assert.throws(()=>assertOptionalDefaultParameterSource(source.replace('return output;','return [];'),row,kind));}assert.throws(()=>assertOptionalDefaultParameterSource(read('./'+row.source),{...row,operationsPerIteration:2},'native'));}
 for(const file of ['./run.mjs','./profile-native.mjs']){const text=read(file);assert.match(text,/assertOptionalDefaultParameterSource\(source/);assert.match(text,/assertOptionalDefaultParameterSource\((?:ts|counterpart)/);assert.ok(text.includes("'optional-default-parameters-values-oracle.mjs'"));}
});
test('Gate98 independent literal complete outputs route through existing observer and harness oracle',()=>{
 assert.equal(hands.length,35);for(const [id,n,seed,input,literal] of hands){const expected=observeSliceTree(literal,{numbers:id==='default_parameter_number_value'});assert.deepEqual(expectedOptionalDefaultParameterObservation(id,n,seed,input),expected);assert.deepEqual(expectedExactResult(id,n,seed,input),expected);assert.equal(expectedChecksum(id,n,seed,input),expected.checksum);}
 for(const row of rows){const expected=expectedOptionalDefaultParameterObservation(row.id,2048,7,row.inputText);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),expected);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),expected.checksum);}
});
