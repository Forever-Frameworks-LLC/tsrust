import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertStringCodeAtSource} from './string-code-at-source-contract.mjs';
import {stringCodeAtIds,expectedStringCodeAtObservation,expectedStringCodeAtValues} from './string-code-at-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='string-code-at');
test('string code at pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),stringCodeAtIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-string-methods-milestone');assert.equal(reportCategory(row.id),'strings');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertStringCodeAtSource(source,row,kind);
   assert.throws(()=>assertStringCodeAtSource(replaceCodeSnippet(source, 'text.charCodeAt(position)','0.0'),row,kind));
   assert.throws(()=>assertStringCodeAtSource(replaceCodeSnippet(source, 'text.codePointAt(position + 20)','undefined'),row,kind));
   assert.throws(()=>assertStringCodeAtSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertStringCodeAtSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertStringCodeAtSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertStringCodeAtSource\(source/);assert.ok(source.includes("'string-code-at-oracle.mjs'"));}
});
test('string code at independent results match hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives modes 0, 1, 2 and positions 0, 4, 8 in texts of 4, 8 and 12 characters.
 assert.deepEqual(expectedStringCodeAtValues('string_code_at_positions',3,0),[[97,97,-1,NaN,97],[97,97,-1,NaN,233],[97,97,-1,NaN,8364]]);
 // Seed 6: position 6 is past a 4-character text, then position 1 of an 8-character text. Seed 2: position 2 of a 12-character text.
 assert.deepEqual(expectedStringCodeAtValues('string_code_at_positions',2,6),[[-1,NaN,-1,NaN,97],[233,233,-1,NaN,233]]);
 assert.deepEqual(expectedStringCodeAtValues('string_code_at_positions',1,2),[[8364,8364,-1,NaN,8364]]);
 // The table agrees with the platform on every registered case.
 const all=expectedStringCodeAtValues('string_code_at_positions',2048,7);
 for(let i=0;i<2048;i++){const mode=(i+7)%3,position=(i*13+7)%9,text='aé€z'.repeat(mode+1);
  assert.deepEqual(all[i],[text.codePointAt(position)??-1,text.charCodeAt(position),text.codePointAt(position+20)??-1,text.charCodeAt(-1-mode),text.codePointAt(mode)??-1],String(i));}
 for(const row of rows){const want=observeSliceTree(expectedStringCodeAtValues(row.id,2048,7),{numbers:true});assert.deepEqual(expectedStringCodeAtObservation(row.id,2048,7),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
