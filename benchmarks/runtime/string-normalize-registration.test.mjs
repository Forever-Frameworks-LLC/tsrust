import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertStringNormalizeSource} from './string-normalize-source-contract.mjs';
import {stringNormalizeIds,expectedStringNormalizeObservation,expectedStringNormalizeValues} from './string-normalize-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='string-normalize');
test('string normalization pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),stringNormalizeIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-string-methods-milestone');assert.equal(reportCategory(row.id),'strings');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertStringNormalizeSource(source,row,kind);
   assert.throws(()=>assertStringNormalizeSource(replaceCodeSnippet(source, 'text.normalize("NFC")','String(text)'),row,kind));
   assert.throws(()=>assertStringNormalizeSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertStringNormalizeSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertStringNormalizeSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertStringNormalizeSource\(source/);assert.ok(source.includes("'string-normalize-oracle.mjs'"));}
});
test('string normalization independent results match hand-written cells, the platform and oracle routing',()=>{
 assert.deepEqual(expectedStringNormalizeValues('string_normalize_forms',1,0),[['é','é','é','é','é','é','é']]);
 const all=expectedStringNormalizeValues('string_normalize_forms',2048,7),words=['é','Åﬁ①','각','à̕','é😀'],forms=['NFC','NFD','NFKC','NFKD'];
 for(let i=0;i<2048;i++){const text=words[i%5].repeat(((i*13+7)%97)%3+1);assert.deepEqual(all[i],[...forms.map(form=>text.normalize(form)),text.normalize(),text.normalize(undefined),text.normalize(forms[i%4])],String(i));}
 assert.throws(()=>expectedStringNormalizeValues('string_normalize_forms',2049,0));
 assert.deepEqual(expectedStringNormalizeValues('string_normalize_forms',0,0),[]);
 for(const row of rows){const want=observeSliceTree(expectedStringNormalizeValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedStringNormalizeObservation(row.id,2048,7,row.inputText),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
