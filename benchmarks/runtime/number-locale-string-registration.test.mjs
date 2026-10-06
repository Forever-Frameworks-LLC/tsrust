import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertNumberLocaleStringSource} from './number-locale-string-source-contract.mjs';
import {numberLocaleStringIds,expectedNumberLocaleStringObservation,expectedNumberLocaleStringValues} from './number-locale-string-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='number-locale-string');
const original=JSON.parse(read('../../archive/number-locale-string-start/benchmarks/runtime/manifest.json'));
test('numeric toLocaleString pair appends to the manifest and binds actual guarded sources',()=>{
 assert.deepEqual(manifest.workloads.slice(0,original.workloads.length),original.workloads);
 assert.deepEqual(rows.map(row=>row.id),numberLocaleStringIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(row.measurementStatus,'pending-number-methods-milestone');assert.equal(reportCategory(row.id),'numbers');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertNumberLocaleStringSource(source,row,kind);
   assert.throws(()=>assertNumberLocaleStringSource(source.replace('value.toLocaleString()','value.toString()'),row,kind));
   assert.throws(()=>assertNumberLocaleStringSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertNumberLocaleStringSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertNumberLocaleStringSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertNumberLocaleStringSource\(source/);assert.ok(source.includes("'number-locale-string-oracle.mjs'"));}
});
test('numeric toLocaleString independent results match hand-written cells, the platform and oracle routing',()=>{
 assert.deepEqual(expectedNumberLocaleStringValues('number_locale_string_decimal',3,0),[['0','0','-0','-0','1,000,000,000,000,000,000,000'],['13,013','13,013.235','-13,013.235','-0.001','1,000,000,000,000,000,000,000'],['26,026','26,027','-26,027','NaN','1,000,000,000,000,000,000,000']]);
 const all=expectedNumberLocaleStringValues('number_locale_string_decimal',2048,7),fractions=[0,.2345,.9995],specials=[-.0004,-.0005,NaN,Infinity,-Infinity];
 for(let i=0;i<2048;i++){const whole=((i*13+7)%97)*1001,value=whole+fractions[i%3];assert.deepEqual(all[i],[whole.toLocaleString('en-US'),value.toLocaleString('en-US'),(-value).toLocaleString('en-US'),specials[i%5].toLocaleString('en-US'),(1e21).toLocaleString('en-US')],String(i));}
 assert.throws(()=>expectedNumberLocaleStringValues('number_locale_string_decimal',2049,0));
 assert.deepEqual(expectedNumberLocaleStringValues('number_locale_string_decimal',0,0),[]);
 for(const row of rows){const want=observeSliceTree(expectedNumberLocaleStringValues(row.id,2048,7,row.inputText));assert.deepEqual(expectedNumberLocaleStringObservation(row.id,2048,7,row.inputText),want);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
