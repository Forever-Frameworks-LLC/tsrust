import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertOptionalFieldsSource} from './optional-fields-source-contract.mjs';
import {expectedOptionalFieldsValues,expectedOptionalFieldsObservation} from './optional-fields-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='optional_fields');
test('Optional fields retain paired source forms, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertOptionalFieldsSource(source,row,kind);
  assert.throws(()=>assertOptionalFieldsSource(source+' ',row,kind));
  assert.throws(()=>assertOptionalFieldsSource(source,{...row,operationsPerIteration:2},kind));
  for(const changed of [replaceCodeSnippet(source, 'y?: int;','y: int | undefined;'),replaceCodeSnippet(source, 'const flat:Point={x:k};','const flat:Point={x:k,y:undefined};'),replaceCodeSnippet(source, 'return output;','return [];')]){
   const key=kind==='native'?'sourceSha256':'counterpartSha256';
   assert.throws(()=>assertOptionalFieldsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 // Written by hand for seed 3: k is 3 then 4.
 assert.deepEqual(expectedOptionalFieldsValues(row.id,2,3,'tail'),[['3','6','1','9','tail'],['4','8','2','9','tail']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedOptionalFieldsObservation(row.id,2,3,'tail'));
 assert.equal(expectedChecksum(row.id,2,3,'tail'),expectedOptionalFieldsObservation(row.id,2,3,'tail').checksum);
 // The direct forms agree with platform objects on every registered case.
 const width=point=>point.x+(point.y??0);
 for(const input of ['dynamic é🙂 text','abc','']){
  const all=expectedOptionalFieldsValues(row.id,2048,7,input);
  for(let i=0;i<2048;i++){const k=i+7;const flat={x:k};const full={x:k,y:k%7};const moving={x:1};moving.y=k%3;const {y:taken=9}=flat;
   assert.deepEqual(all[i],[String(width(flat)),String(width(full)),String(width(moving)),String(taken),input],input+':'+i);}
 }
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertOptionalFieldsSource(source'));assert.ok(source.includes("'optional-fields-oracle.mjs'"));}
});
