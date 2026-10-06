import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertEnumsSource} from './enums-source-contract.mjs';
import {expectedEnumsValues,expectedEnumsObservation} from './enums-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='enum_values');
test('enums have matched source, oracle, denominator and hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertEnumsSource(source,row,kind);
  assert.throws(()=>assertEnumsSource(source+' ',row,kind));
  assert.throws(()=>assertEnumsSource(source,{...row,operationsPerIteration:2},kind));
  for(const text of ['function workload(','const phase=choose(i+seed);','const copy=phase;','String(Number(copy))','input+String(Number(phase))']){
   const changed=source.replace(text,'0');const key=kind==='native'?'sourceSha256':'counterpartSha256';
   assert.throws(()=>assertEnumsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedEnumsValues(row.id,2,0,'ab'),[['0','s','ab0'],['4','long','ab4']]);
 assert.deepEqual(expectedExactResult(row.id,2,0,'ab'),expectedEnumsObservation(row.id,2,0,'ab'));
 for(const file of ['./run.mjs','./profile-native.mjs']){
  const source=read(file);assert.ok(source.includes('assertEnumsSource(source'));
  assert.ok(source.includes("'enums-oracle.mjs'"));
 }
 
});
