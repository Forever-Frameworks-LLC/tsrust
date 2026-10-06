import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertStringFromCodesSource} from './string-from-codes-source-contract.mjs';
import {stringFromCodesIds,expectedStringFromCodesObservation,expectedStringFromCodesValues,textFromCodes} from './string-from-codes-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='string-from-codes');
test('string from codes pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),stringFromCodesIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-string-methods-milestone');assert.equal(reportCategory(row.id),'strings');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertStringFromCodesSource(source,row,kind);
   assert.throws(()=>assertStringFromCodesSource(source.replace('String.fromCodePoint(...codes)','"x"'),row,kind));
   assert.throws(()=>assertStringFromCodesSource(source.replace('String.fromCharCode(','String.fromCodePoint('),row,kind));
   assert.throws(()=>assertStringFromCodesSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertStringFromCodesSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertStringFromCodesSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertStringFromCodesSource\(source/);assert.ok(source.includes("'string-from-codes-oracle.mjs'"));}
});
test('string from codes independent results match hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0, first iteration has base 0.
 assert.equal(textFromCodes([65,233,19968,128512]),'Aé一😀');
 assert.deepEqual(expectedStringFromCodesValues('string_from_codes_text',1,0),[['Aé一😀','Hi€','😀a']]);
 // The byte encoder agrees with the platform on every registered case.
 const all=expectedStringFromCodesValues('string_from_codes_text',2048,7);
 for(let i=0;i<2048;i++){const base=(i*13+7)%97,codes=[65+base%26,233+base,19968+base,128512+base%64];
  assert.deepEqual(all[i],[String.fromCodePoint(...codes),String.fromCharCode(72+base%20,105,8364),String.fromCodePoint(codes[3],97+base%26)],String(i));}
 for(const row of rows){const want=observeSliceTree(expectedStringFromCodesValues(row.id,2048,7));assert.deepEqual(expectedStringFromCodesObservation(row.id,2048,7),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
