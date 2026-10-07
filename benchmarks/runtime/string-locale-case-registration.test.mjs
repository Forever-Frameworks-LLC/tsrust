import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertStringLocaleCaseSource} from './string-locale-case-source-contract.mjs';
import {stringLocaleCaseIds,expectedStringLocaleCaseObservation,expectedStringLocaleCaseValues} from './string-locale-case-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='string-locale-case');
test('string locale case pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),stringLocaleCaseIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-string-methods-milestone');assert.equal(reportCategory(row.id),'strings');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertStringLocaleCaseSource(source,row,kind);
   assert.throws(()=>assertStringLocaleCaseSource(replaceCodeSnippet(source, 'text.toLocaleLowerCase()','text.toLowerCase()'),row,kind));
   assert.throws(()=>assertStringLocaleCaseSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertStringLocaleCaseSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertStringLocaleCaseSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertStringLocaleCaseSource\(source/);assert.ok(source.includes("'string-locale-case-oracle.mjs'"));}
});
test('string locale case independent results match hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0, 13, 26, 39 and so 1, 2, 3, 1 copies.
 assert.deepEqual(expectedStringLocaleCaseValues('string_locale_case_text',4,0),[['ab','AB'],['éééé','ÉÉÉÉ'],['ßsßsßs','SSSSSSSSS'],['ωω','ΩΩ']]);
 // The written forms agree with the platform's locale-independent mapping on every registered case.
 const all=expectedStringLocaleCaseValues('string_locale_case_text',2048,7),words=['Ab','Éé','ßs','Ωω'];
 for(let i=0;i<2048;i++){const text=words[i%4].repeat(((i*13+7)%97)%3+1);assert.deepEqual(all[i],[text.toLowerCase(),text.toUpperCase()],String(i));}
 for(const row of rows){const want=observeSliceTree(expectedStringLocaleCaseValues(row.id,2048,7));assert.deepEqual(expectedStringLocaleCaseObservation(row.id,2048,7),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
