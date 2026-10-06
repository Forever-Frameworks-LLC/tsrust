import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertTypeAssertionsSource} from './type-assertions-source-contract.mjs';
import {expectedTypeAssertionsValues,expectedTypeAssertionsObservation} from './type-assertions-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='type_assertions');
test('checked assertions have matched source, oracle, denominator and hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertTypeAssertionsSource(source,row,kind);
  assert.throws(()=>assertTypeAssertionsSource(source+' ',row,kind));
  assert.throws(()=>assertTypeAssertionsSource(source,{...row,operationsPerIteration:2},kind));
  for(const text of ['function workload(','value satisfies number','value as number','String(widened+0.5)','input+rendered']){
   const changed=source.replace(text,'0');const key=kind==='native'?'sourceSha256':'counterpartSha256';
   assert.throws(()=>assertTypeAssertionsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 assert.deepEqual(expectedTypeAssertionsValues(row.id,2,0,'ab'),[['0','0.5','0','ab0'],['13','13.5','13','ab13']]);
 assert.deepEqual(expectedExactResult(row.id,2,0,'ab'),expectedTypeAssertionsObservation(row.id,2,0,'ab'));
 for(const file of ['./run.mjs','./profile-native.mjs']){
  const source=read(file);assert.ok(source.includes('assertTypeAssertionsSource(source'));
  assert.ok(source.includes("'type-assertions-oracle.mjs'"));
 }
 
});
