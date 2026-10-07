import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertStringRawSource} from './string-raw-source-contract.mjs';
import {stringRawIds,expectedStringRawObservation,expectedStringRawValues} from './string-raw-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='string-raw');
test('String.raw pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),stringRawIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-string-methods-milestone');assert.equal(reportCategory(row.id),'strings');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertStringRawSource(source,row,kind);
   assert.throws(()=>assertStringRawSource(replaceCodeSnippet(source, 'String.raw(template, base, true)','String(template)'),row,kind));
   assert.throws(()=>assertStringRawSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertStringRawSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertStringRawSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertStringRawSource\(source/);assert.ok(source.includes("'string-raw-oracle.mjs'"));}
});
test('String.raw independent results match hand-written cells, the platform and oracle routing',()=>{
 assert.deepEqual(expectedStringRawValues('string_raw_segments',1,0,'z'),[['a0étruez\\n','aXéz\\n','aXéYz\\n','only','']]);
 const all=expectedStringRawValues('string_raw_segments',2048,7,'z');
 for(let i=0;i<2048;i++){const template={raw:['a','é','z\\n']},inserts=['X','Y'],base=(i*13+7)%97;assert.deepEqual(all[i],[String.raw(template,base,true),String.raw(template,'X'),String.raw(template,...inserts),String.raw({raw:['only']},...inserts),String.raw({raw:[]},base)],String(i));}
 assert.throws(()=>expectedStringRawValues('string_raw_segments',2049,0,''));
 assert.deepEqual(expectedStringRawValues('string_raw_segments',0,0,''),[]);
 for(const row of rows){const want=observeSliceTree(expectedStringRawValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedStringRawObservation(row.id,2048,7,row.inputText),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
