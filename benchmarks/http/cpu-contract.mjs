import assert from 'node:assert/strict';
import fs from 'node:fs';
import {hash,validateResponse} from './requests-contract.mjs';
export {hash,validateResponse};
export const protocol=Object.freeze({processes:3,warmups:8,samples:31,requestsPerBatch:128,concurrency:[1,16],keepAlive:true,bodyLimit:65536,width:96,height:64,maxIterations:256,serverWorkers:1,resourceBatches:31,rssSampleMs:50,cpuClockResolutionMs:10,latency:'request start to complete body; nearest-rank percentiles'});
export const workloads=Object.freeze(protocol.concurrency.map(concurrency=>({id:'http_cpu_render_c'+concurrency,scenario:'cpu_render',name:'CPU rendering',method:'POST',path:'/render',concurrency})));
export const sourceFiles={'native':'cpu.tsr','node-tsc':'cpu.ts','rust-hand':'cpu.rs'};
export function sourceHashes(){return Object.fromEntries(Object.entries(sourceFiles).map(([e,f])=>[e,hash(fs.readFileSync(new URL(f,import.meta.url)))]));}
const oracle=JSON.parse(fs.readFileSync(new URL('./cpu-oracle.json',import.meta.url)));
export function oracleHashes(){return {oracle:hash(fs.readFileSync(new URL('./cpu-oracle.json',import.meta.url))),oracleGenerator:hash(fs.readFileSync(new URL('./cpu-oracle.py',import.meta.url)))};}
export function assertManifest(m){assert.equal(m.semanticEpoch,'http-cpu-render-v1');assert.deepEqual(m.protocol,protocol);assert.deepEqual(m.workloads,workloads);assert.deepEqual(m.sources,sourceHashes());assert.deepEqual(m.oracles,oracleHashes());assert.equal(oracle.length,128);}
export function cases(w){assert(workloads.some(x=>x.id===w.id));return oracle.map((row,index)=>{assert.equal(row.id,index+1);return {method:'POST',path:'/render',body:Buffer.from(JSON.stringify({id:row.id})),status:200,type:'application/json',expected:Buffer.from(JSON.stringify(row))};});}
export const rejectionCases=Object.freeze(['{','{"id":0}','{"id":129}','{"id":"1"}'].map(body=>({method:'POST',path:'/render',body:Buffer.from(body),status:400,type:'text/plain',expected:Buffer.from('invalid')})));
