import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';
import {taskErgonomicsIDs,taskErgonomicsChecksum,assertTaskErgonomicsReference,assertTaskErgonomicsSource,taskErgonomicsRustDriver,taskErgonomicsABI} from './task-ergonomics-contract.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url),'utf8'));
test('task async support keeps all six independently observed domains and guards',()=>{
 for(const id of taskErgonomicsIDs){const row=manifest.workloads.find(row=>row.id===id);assert.ok(row);for(const[engine,key]of[['native','source'],['typescript','counterpart']]){const source=fs.readFileSync(new URL(row[key],import.meta.url),'utf8');assertTaskErgonomicsSource(source,row,engine);assert.throws(()=>assertTaskErgonomicsSource(source.replaceAll('4096','4095').replaceAll('trySend','sendNow').replaceAll('Promise.all','Promise.any').replaceAll('all(','race(').replaceAll('shared.cancel()','token.isCancelled()').replaceAll('shared.abort()','token.signal.aborted').replaceAll('await sender.send','sender.send').replaceAll('await queue.send','queue.send'),row,engine));}assert.equal(taskErgonomicsChecksum(id,0,7),0);assert.ok(Number.isSafeInteger(taskErgonomicsChecksum(id,3,7)));}
 const ordered=[7,8,9,10,8,9,10,11,9,10,11,12].reduce((sum,value)=>(sum*131+value)%1000000007,0);assert.equal(taskErgonomicsChecksum('task_all_ready',3,7),ordered);assert.equal(taskErgonomicsChecksum('async_channel_try',3,7),ordered);
});
test('async ABI fails closed and runtime startup surrounds the sample loop',()=>{
 const row=manifest.workloads.find(row=>row.id==='task_all_ready');const source=fs.readFileSync(new URL(row.source,import.meta.url),'utf8');
 const emitted='fn function0(value0: i64, value1: i64) -> Closure0 { todo!() }\nimpl Closure0 {\n async fn call(self) -> Result<i64,TsrClosureError0> { todo!() }\n}';
 const abi=taskErgonomicsABI(emitted,row,source);assert.equal(abi.runtimeOnce,true);assert.match(abi.call,/match function0/);
 const driver=taskErgonomicsRustDriver(row,emitted,source);assert.equal(driver.match(/block_on\(/g).length,1);assert.ok(driver.indexOf('block_on(')<driver.indexOf('for _ in 0..samples'));assert.match(driver,/unexpected checked async benchmark failure/);
 assert.throws(()=>taskErgonomicsABI(emitted.replace('value1: i64','value1: f64'),row,source));assert.throws(()=>taskErgonomicsABI(emitted.replace('Result<i64,TsrClosureError0>','f64'),row,source));
});

test('ordered fold rejects swapped observations across group and channel sources',()=>{
 const ids=['task_all_ready','task_all_yield','async_channel_try','async_channel_await'];
 const normal=[7,8,9,10].reduce((sum,value)=>(sum*131+value)%1000000007,0);
 const swapped=[8,7,9,10].reduce((sum,value)=>(sum*131+value)%1000000007,0);
 assert.notEqual(swapped,normal);
 for(const id of ids){
  const row=manifest.workloads.find(row=>row.id===id);assert.equal(taskErgonomicsChecksum(id,1,7),normal);assert.notEqual(taskErgonomicsChecksum(id,1,7),swapped);
  for(const[engine,key]of[['native','source'],['typescript','counterpart']]){
   const source=fs.readFileSync(new URL(row[key],import.meta.url),'utf8');assertTaskErgonomicsSource(source,row,engine);
   const altered=id.startsWith('task_all_')?replaceCodeSnippet(source,'value(seed+i),value(seed+i+1)','value(seed+i+1),value(seed+i)'):replaceCodeSnippet(source,'seed+i+j','seed+i+(3-j)');
   assert.throws(()=>assertTaskErgonomicsSource(altered,row,engine));
   assert.throws(()=>assertTaskErgonomicsSource(replaceCodeSnippet(source,'sum*131+','sum+'),row,engine));
  }
  const reference=fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8');assertTaskErgonomicsReference(reference,row);
  assert.throws(()=>assertTaskErgonomicsReference(replaceCodeSnippet(reference,'sum*131+','sum+'),row));
  const altered=id.startsWith('task_all_')?replaceCodeSnippet(reference,'values.0,values.1','values.1,values.0'):replaceCodeSnippet(reference,'seed+i+j','seed+i+(3-j)');
  assert.throws(()=>assertTaskErgonomicsReference(altered,row));
 }
});
