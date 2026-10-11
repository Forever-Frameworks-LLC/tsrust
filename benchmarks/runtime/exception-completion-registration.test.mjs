import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertExceptionCompletionSource} from './exception-completion-source-contract.mjs';
import {expectedExceptionCompletionValues,expectedExceptionCompletionObservation} from './exception-completion-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='exception_completion');
test('exception registration retains matched source and oracle',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertExceptionCompletionSource(source,row,kind);
  assert.throws(()=>assertExceptionCompletionSource(source+' ',row,kind));
  assert.throws(()=>assertExceptionCompletionSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedExceptionCompletionValues(row.id,2,0,'ab'),[['2','4','0,7','ab0'],['14','17','13,7','ab13']]);
 assert.deepEqual(expectedExactResult(row.id,2,0,'ab'),expectedExceptionCompletionObservation(row.id,2,0,'ab'));
});

test('exception source mutations and registration history stay guarded',async()=>{
 const {createHash}=await import('node:crypto');
 const manifest=JSON.parse(read('./manifest.json'));
 
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);
  for(const text of ['throw value;','return value+1;','return error+2;','finally{trace.push(value);}','finally{return value+4;}','trace.join(",")','if(early){return value;}','finally{trace.push(7);}','const owned=retained(input+String(value),value%2===0,trace);']){
   const changed=replaceCodeSnippet(source, text,'');
   const key=kind==='native'?'sourceSha256':'counterpartSha256';
   assert.throws(()=>assertExceptionCompletionSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 for(const file of ['./run.mjs','./profile-native.mjs']){
  const source=read(file);assert.ok(source.includes('assertExceptionCompletionSource(source'));
  assert.ok(source.includes("'exception-completion-oracle.mjs'"));
 }
});
test('platform completion behavior agrees with independent oracle',()=>{
 for(const n of [0,1,97,2048])for(const seed of [0,7,1000000]){
  const actual=[];
  for(let i=0;i<n;i++){
   const value=(i*13+seed)%97,trace=[];
   const attempt=()=>{try{if(value%3===0)throw value;return value+1;}catch(error){return error+2;}finally{trace.push(value);}};
   const replaced=()=>{try{return value;}finally{return value+4;}};
   const result=attempt();
   const retained=(owner)=>{try{if(value%2===0)return owner;}finally{trace.push(7);}return owner;};
   const owned=retained('é🙂'+value);
   actual.push([String(result),String(replaced()),trace.join(','),owned]);
  }
  assert.deepEqual(actual,expectedExceptionCompletionValues(row.id,n,seed,'é🙂'));
 }
});
