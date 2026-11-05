import fs from 'node:fs';
import {codeTokens,containsCodeSnippet} from './source-contract-tokens.mjs';
export const taskErgonomicsIDs=['task_all_ready','task_all_yield','async_channel_try','async_channel_await','cancellation_fanout','thread_pool_async'];
export const taskErgonomicsSupport=Object.fromEntries(['async-driver.rs','async-driver.ts','task-ergonomics-contract.mjs'].map(name=>[name,fs.readFileSync(new URL(name,import.meta.url),'utf8')]));
export function taskErgonomicsChecksum(id,n,seed){
 if(!taskErgonomicsIDs.includes(id))throw Error(`Unknown task workload ${id}`);
 let total=0n;const modulus=1000000007n;
 for(let i=0n;i<BigInt(n);i++){
  let value=BigInt(seed)+i;
  if(id==='thread_pool_async'){for(let j=0n;j<4096n;j++)value=(value*17n+j)%modulus;total=(total+value)%modulus;}
  else if(id.startsWith('task_all_')||id.startsWith('async_channel_')){for(let j=0n;j<4n;j++)total=(total*131n+value+j)%modulus;}
  else total=(total+4n*value+6n)%modulus;
 }
 return Number(total);
}
export function assertTaskErgonomicsSource(source,w,engine){
 if(!taskErgonomicsIDs.includes(w.id)||w.result!=='int'||w.parameters.join(',')!=='int,int')throw Error('Task workload domain changed');
 const native=engine==='native';const signature=native?'async function workload(n:int,seed:int):Task<int>':'async function workload(n:number,seed:number):Promise<number>';
 if(!containsCodeSnippet(source,signature))throw Error(`Async workload ABI changed: ${w.id}`);
 const witnesses=w.id.startsWith('task_all_')?[native?'await all(':'await Promise.all(',native?'for(const value of values)':'for(const value of values)']:
  w.id.startsWith('async_channel_')?[w.id.endsWith('_try')?'trySend(':'await '+(native?'sender':'queue')+'.send(',w.id.endsWith('_try')?'tryReceive(':'await queue.receive(']:
  w.id==='cancellation_fanout'?[native?'token.child()':'new AbortController()',native?'shared.cancel()':'shared.abort()',native?'await a.cancelled()':'child.signal.aborted']:
  [native?'pool.taskAsync(':'new Worker(', native?'4096':'worker.postMessage(seed+i)',native?'pool.shutdown()':'worker.terminate()'];
 for(const witness of witnesses)if(!containsCodeSnippet(source,witness))throw Error(`Task semantic guard missing ${witness}`);
 if(w.id==='thread_pool_async'&&!native){
  const tokens=codeTokens(source),index=tokens.findIndex((token,index)=>containsCodeSnippet(tokens.slice(index,index+3),'new Worker('));
  const literal=tokens[index+3]?.text;
  if(index<0||!literal?.startsWith('`')||!literal.endsWith('`')||literal.includes('${')||!containsCodeSnippet(literal.slice(1,-1),'j < 4096'))throw Error('Worker CPU loop changed');
 }
 if(w.id.startsWith('task_all_')){
  const group=native?'all':'Promise.all';
  if(!containsCodeSnippet(source,`${group}([value(seed+i),value(seed+i+1),value(seed+i+2),value(seed+i+3)])`)||!containsCodeSnippet(source,'sum=(sum*131+value)%1000000007'))throw Error('Ordered group observation changed');
 }
 if(w.id.startsWith('async_channel_')){
  const counter=native?'for(let j:int=0;j<4;j++)':'for(let j=0;j<4;j++)';
  const send=w.id.endsWith('_try')?(native?'unwrap(sender.trySend(seed+i+j))':'queue.trySend(seed+i+j)'):(native?'await sender.send(seed+i+j)':'await queue.send(seed+i+j)');
  if(!containsCodeSnippet(source,counter)||!containsCodeSnippet(source,send)||!containsCodeSnippet(source,'sum=(sum*131+'))throw Error('Ordered channel observation changed');
 }
 if(w.id==='task_all_yield'&&!containsCodeSnippet(source,native?'await yield()':'setImmediate(resolve)'))throw Error('Yield disappeared');
}
export function assertTaskErgonomicsReference(source,w){
 if(w.id.startsWith('task_all_')){
  if(!containsCodeSnippet(source,'[values.0,values.1,values.2,values.3]')||!containsCodeSnippet(source,'sum=(sum*131+value)%1000000007'))throw Error('Ordered reference group observation changed');
 }
 if(w.id.startsWith('async_channel_')){
  const send=w.id.endsWith('_try')?'tx.try_send(seed+i+j)':'tx.send(seed+i+j).await';
  if(!containsCodeSnippet(source,'for j in 0..4')||!containsCodeSnippet(source,send)||!containsCodeSnippet(source,'sum=(sum*131+'))throw Error('Ordered reference channel observation changed');
 }
}
export function taskErgonomicsABI(emitted,w,source,{reference=false}={}){
 if(reference){assertTaskErgonomicsReference(emitted,w);if(!/async fn function0\(n:i64,seed:i64\)->i64/.test(emitted))throw Error('Direct async reference ABI changed');return {functionName:'function0',call:'function0(n,seed).await',result:'i64',runtimeOnce:true};}
 assertTaskErgonomicsSource(source,w,'native');
 const creators=[...emitted.matchAll(/^fn function0\(([^\n]*)\) -> (Closure\d+) \{/gm)];
 if(creators.length!==1||creators[0][1]!=='value0: i64, value1: i64')throw Error('Task workload creator ABI changed');
 const closure=creators[0][2];const start=emitted.indexOf(`impl ${closure} {`);if(start<0)throw Error('Concrete task implementation missing');
 const method=/async fn call\([^\n]*\) -> ([^\n]+) \{/.exec(emitted.slice(start));if(!method)throw Error('Task poll ABI changed');
 const output=method[1];if(output!=='i64'&&!/^Result<i64,\s*TsrClosureError\d+>$/.test(output))throw Error(`Task completion ABI changed: ${output}`);
 const poll='function0(n,seed).call().await';const call=output==='i64'?poll:`match ${poll}{Ok(value)=>value,Err(_)=>{eprintln!("unexpected checked async benchmark failure");std::process::exit(1)}}`;
 return {functionName:'function0',closure,result:'i64',output,call,runtimeOnce:true};
}
export function taskErgonomicsRustDriver(w,emitted,source,options){const abi=taskErgonomicsABI(emitted,w,source,options);return taskErgonomicsSupport['async-driver.rs']+`\nasync fn benchmark_async_workload(n:i64,seed:i64)->i64{${abi.call}}\n`;}
