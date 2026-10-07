import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertArrayLocaleStringSource} from './array-locale-string-source-contract.mjs';
import {arrayLocaleStringIds,expectedArrayLocaleStringObservation,expectedArrayLocaleStringValues} from './array-locale-string-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='array-locale-string');
test('array toLocaleString pair appends to the manifest and binds actual guarded sources',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),arrayLocaleStringIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-array-methods-milestone');assert.equal(reportCategory(row.id),'arrays');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertArrayLocaleStringSource(source,row,kind);
   assert.throws(()=>assertArrayLocaleStringSource(replaceCodeSnippet(source, 'ints.toLocaleString()','ints.toString()'),row,kind));
   assert.throws(()=>assertArrayLocaleStringSource(replaceCodeSnippet(source, 'return output;','return [];'),row,kind));
   assert.throws(()=>assertArrayLocaleStringSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertArrayLocaleStringSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertArrayLocaleStringSource\(source/);assert.ok(source.includes("'array-locale-string-oracle.mjs'"));}
});
test('array toLocaleString independent results match hand-written cells, the platform and oracle routing',()=>{
 assert.deepEqual(expectedArrayLocaleStringValues('array_locale_string_values',1,0,'x'),[['0,0','1,000.5,-0,1.235','x,é😀,','true,false',',',',','[object Object],[object Object]','1,000.5,-0,,2,000.235']]);
 const all=expectedArrayLocaleStringValues('array_locale_string_values',2048,7,'x');
 for(let i=0;i<2048;i++){const base=(i*13+7)%97,whole=base*1001;assert.deepEqual(all[i],[[whole,0-whole],[1000.5+base,-0,1.2345],['x','é😀',''],[true,false],[null,null],[undefined,undefined],[{n:base},{n:whole}],[[1000.5,-0],[],[2000.2345+base]]].map(values=>values.toLocaleString('en-US')),String(i));}
 assert.throws(()=>expectedArrayLocaleStringValues('array_locale_string_values',2049,0,''));
 assert.deepEqual(expectedArrayLocaleStringValues('array_locale_string_values',0,0,''),[]);
 for(const row of rows){const want=observeSliceTree(expectedArrayLocaleStringValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedArrayLocaleStringObservation(row.id,2048,7,row.inputText),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
