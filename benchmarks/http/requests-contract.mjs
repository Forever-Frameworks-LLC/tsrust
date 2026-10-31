import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
export const protocol=Object.freeze({processes:3,warmups:8,samples:31,requestsPerBatch:128,concurrency:[1,16],keepAlive:true,bodyLimit:65536,sqliteRows:128});
const scenarios=[
 {scenario:'text',name:'Text response',method:'GET',path:'/text'},
 {scenario:'json',name:'JSON response',method:'GET',path:'/json'},
 {scenario:'parse_json',name:'Parse JSON → JSON',method:'POST',path:'/parse'},
 {scenario:'sqlite_json',name:'Parse JSON → SQLite → JSON',method:'POST',path:'/sqlite'},
];
export const workloads=Object.freeze(scenarios.flatMap(row=>protocol.concurrency.map(concurrency=>({...row,id:`http_${row.scenario}_c${concurrency}`,concurrency}))));
export const sourceFiles={'native':'requests.tsr','node-tsc':'requests.ts','rust-hand':'requests.rs'};
export const hash=bytes=>createHash('sha256').update(bytes).digest('hex');
export function sourceHashes(){return Object.fromEntries(Object.entries(sourceFiles).map(([engine,file])=>[engine,hash(fs.readFileSync(new URL(file,import.meta.url)))]));}
export function assertManifest(manifest){assert.deepEqual(manifest.protocol,protocol);assert.deepEqual(manifest.workloads,workloads);assert.deepEqual(manifest.sources,sourceHashes());}
export function cases(workload){
 assert(workloads.some(row=>row.id===workload.id));
 return Array.from({length:protocol.requestsPerBatch},(_,index)=>{
  const id=index+1;
  let input,expected;
  if(workload.scenario==='text')expected='héllo 🦀';
  if(workload.scenario==='json')expected=JSON.stringify({id:7,label:'héllo 🦀',value:42});
  if(workload.scenario==='parse_json'){input=JSON.stringify({id,label:'héllo 🦀',value:index*3});expected=JSON.stringify({id,label:'héllo 🦀!',value:index*3+1});}
  if(workload.scenario==='sqlite_json'){input=JSON.stringify({id});expected=JSON.stringify({id,label:`row héllo ${id}`,value:id*3});}
  return {method:workload.method,path:workload.path,body:input===undefined?undefined:Buffer.from(input),status:200,type:workload.scenario==='text'?'text/plain':'application/json',expected:Buffer.from(expected)};
 });
}
export const rejectionCases=Object.freeze([
 {method:'POST',path:'/parse',body:Buffer.from('{'),status:400,type:'text/plain',expected:Buffer.from('invalid')},
 {method:'POST',path:'/parse',body:Buffer.from('{"id":1,"label":7,"value":3}'),status:400,type:'text/plain',expected:Buffer.from('invalid')},
 {method:'POST',path:'/parse',body:Buffer.from('{"id":1,"label":"\\ud800","value":3}'),status:400,type:'text/plain',expected:Buffer.from('invalid')},
 {method:'POST',path:'/sqlite',body:Buffer.from('{"id":0}'),status:400,type:'text/plain',expected:Buffer.from('invalid')},
 {method:'POST',path:'/sqlite',body:Buffer.from('{"id":999}'),status:404,type:'text/plain',expected:Buffer.from('missing')},
]);
export function validateResponse(test,status,headers,body){
 assert.equal(status,test.status,'HTTP status');assert(String(headers['content-type']).startsWith(test.type),'Content-Type');assert.deepEqual(body,test.expected,'Complete HTTP response bytes');
}
