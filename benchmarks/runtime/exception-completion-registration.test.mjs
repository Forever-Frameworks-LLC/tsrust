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
 assert.deepEqual(expectedExceptionCompletionValues(row.id,2,0,'ab'),[['2','4','0,7,11,4,20,31','alternate'],['14','17','13,7,11,32,8,20,30,31','alternate']]);
 assert.deepEqual(expectedExactResult(row.id,2,0,'ab'),expectedExceptionCompletionObservation(row.id,2,0,'ab'));
});

test('exception source mutations and registration history stay guarded',async()=>{
 const {createHash}=await import('node:crypto');
 const manifest=JSON.parse(read('./manifest.json'));
 
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);
  for(const text of ['throw value;','return value+1;','return error+2;','finally{trace.push(value);}','finally{return value+4;}','trace.join(",")','if(early){return value;}','finally{trace.push(7);}','const owned=retained(input+String(value),value%2===0,trace);','const field=retainedField({text:owned,stamp:11},value%2===0,trace);','if(early){return value.text;}','finally{trace.push(row.run());}','const mapped=retainedMap({text:field,stamp:1},value%2===0,trace);','if(early){return value.text;}handlers.clear();','handlers.clear();','trace.push(item.run()+value.stamp);','if(early){return pair.first;}','if(replace){return pair.second;}','if(mode===1){break;}','if(mode===2){continue;}','trace.push(20+i);','throw 31;','trace.push(29);','trace.push(30);','trace.push(31);','throw 32;','finally{}}catch(error){trace.push(32);}','throw mode;','const stamp=[value.stamp];','const run=():string=>']){
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
   const retainedField=(owner)=>{const stamp=[owner.stamp];const row={text:owner.text,run:()=>stamp[0]};try{if(value%2===0)return row.text;}finally{trace.push(row.run());}return row.text;};
   const field=retainedField({text:owned,stamp:11});
   const retainedMap=(owner)=>{const old=[3],current=[7],handlers=new Map([['old',{run:()=>old[0]}]]);try{try{if(value%2===0)return owner.text;handlers.clear();handlers.set('new',{run:()=>current[0]});throw 32;}finally{}}catch(error){trace.push(32);}finally{const item=handlers.get(value%2===0?'old':'new');if(item!==undefined)trace.push(item.run()+owner.stamp);}return owner.text;};
   const mapped=retainedMap({text:field,stamp:1});
   const conditionalCleanup=(owner)=>{const pair={first:owner,second:'alternate'};try{if(value%2===0)return pair.first;}finally{if(value%3===0)return pair.second;}return pair.second;};
   const retainedLoop=(owner)=>{for(let step=0;step<2;step+=1){try{if(value%4===0)return owner.text;if(value%4===1||value%4===2)throw value%4;}catch(error){if(value%4===1)break;if(value%4===2)continue;}finally{trace.push(20+step);}}return owner.text;};
   const looped=retainedLoop({text:mapped,stamp:0});
   const retainedCatch=(owner)=>{const run=()=>{try{if(value%3===0)return owner.text;if(value%3===1)throw 31;trace.push(29);}catch(error){trace.push(30);if(value%3===1)return owner.text;}finally{trace.push(31);}return owner.text;};return run();};
   const caught=retainedCatch({text:looped,stamp:0});
   const selected=conditionalCleanup(caught);
   actual.push([String(result),String(replaced()),trace.join(','),selected]);
  }
  assert.deepEqual(actual,expectedExceptionCompletionValues(row.id,n,seed,'é🙂'));
 }
});
