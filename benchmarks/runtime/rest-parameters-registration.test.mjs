import {manifestProtocolFingerprint} from './manifest-contract.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';import {createHash} from 'node:crypto';
import {assertRestParameterSource} from './rest-parameters-source-contract.mjs';import {restParameterIds,expectedRestParameterObservation} from './rest-parameters-oracle.mjs';import {hands} from './rest-parameters-values-oracle.mjs';import {observeSliceTree} from './slice-observation.mjs';import {expectedChecksum,expectedExactResult} from './oracle.mjs';import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),manifest=JSON.parse(read('./manifest.json')),rows=manifest.workloads.filter(r=>r.family==='rest-parameters');
const parentDigest='cdbe4c8aba548de2c7d8934acff7e36f149b11d23fca7402912ca3d153a9abeb';
test('Gate99 source registration preserves prior rows and binds complete scenario sources',()=>{
 assert.equal(manifestProtocolFingerprint(manifest.workloads.slice(0,704)),parentDigest);assert.ok(manifest.gate>=99);assert.deepEqual(rows.map(r=>r.id),restParameterIds);assert.equal(rows.length,13);
 for(const row of rows){assert.equal(row.measured,false);assert.equal(row.pendingCheckpoint,101);assert.equal(reportCategory(row.id),'callbacks');for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){const source=read('./'+name);assertRestParameterSource(source,row,kind);assert.throws(()=>assertRestParameterSource(replaceCodeSnippet(source, '...',''),row,kind));assert.throws(()=>assertRestParameterSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));}assert.throws(()=>assertRestParameterSource(read('./'+row.source),{...row,operationsPerIteration:2},'native'));}
 for(const p of ['./run.mjs','./profile-native.mjs']){const t=read(p);assert.match(t,/assertRestParameterSource\(source/);assert.match(t,/assertRestParameterSource\((?:ts|counterpart)/);assert.ok(t.includes("'rest-parameters-values-oracle.mjs'"));}
});
test('Gate99 independent complete literal outputs use unchanged observation and oracle routing',()=>{
 assert.equal(hands.length,42);for(const [id,n,seed,input,literal] of hands){const want=observeSliceTree(literal,{numbers:id==='rest_parameter_number_pack'});assert.deepEqual(expectedRestParameterObservation(id,n,seed,input),want);assert.deepEqual(expectedExactResult(id,n,seed,input),want);assert.equal(expectedChecksum(id,n,seed,input),want.checksum);}
 for(const row of rows){const want=expectedRestParameterObservation(row.id,2048,7,row.inputText);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
