import {manifestProtocolFingerprint} from './manifest-contract.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';import {createHash} from 'node:crypto';
import {assertCallSpreadSource} from './call-spread-source-contract.mjs';import {callSpreadIds,expectedCallSpreadObservation} from './call-spread-oracle.mjs';import {hands} from './call-spread-values-oracle.mjs';import {observeSliceTree} from './slice-observation.mjs';import {expectedChecksum,expectedExactResult} from './oracle.mjs';import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8'),manifest=JSON.parse(read('./manifest.json')),rows=manifest.workloads.filter(r=>r.family==='call-spread');
const parentDigest='b1f779e15d85a0eb11387f596e4f6d35f49c00bf5bd413c3505e450254857e88';
test('Gate100 source registration preserves prior rows and binds complete scenario sources',()=>{
 assert.equal(manifestProtocolFingerprint(manifest.workloads.slice(0,718)),parentDigest);assert.ok(manifest.gate>=100);assert.deepEqual(rows.map(r=>r.id),callSpreadIds);assert.equal(rows.length,13);
 for(const row of rows){assert.equal(row.measured,false);assert.equal(row.pendingCheckpoint,101);assert.equal(reportCategory(row.id),'callbacks');for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){const source=read('./'+name);assertCallSpreadSource(source,row,kind);assert.throws(()=>assertCallSpreadSource(replaceCodeSnippet(source, '...',''),row,kind));assert.throws(()=>assertCallSpreadSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));}assert.throws(()=>assertCallSpreadSource(read('./'+row.source),{...row,operationsPerIteration:2},'native'));}
 for(const p of ['./run.mjs','./profile-native.mjs']){const t=read(p);assert.match(t,/assertCallSpreadSource\(source/);assert.match(t,/assertCallSpreadSource\((?:ts|counterpart)/);assert.ok(t.includes("'call-spread-values-oracle.mjs'"));}
});
test('Gate100 independent complete literal outputs use unchanged observation and oracle routing',()=>{
 assert.equal(hands.length,43);for(const [id,n,seed,input,literal] of hands){const want=observeSliceTree(literal,{numbers:id==='call_spread_math_number'});assert.deepEqual(expectedCallSpreadObservation(id,n,seed,input),want);assert.deepEqual(expectedExactResult(id,n,seed,input),want);assert.equal(expectedChecksum(id,n,seed,input),want.checksum);}
 for(const row of rows){const want=expectedCallSpreadObservation(row.id,2048,7,row.inputText);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
