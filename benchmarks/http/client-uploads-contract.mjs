import assert from 'node:assert/strict';import fs from 'node:fs';import {createHash} from 'node:crypto';
import {validateResponse,requestEngines} from './requests-contract.mjs';export {validateResponse,requestEngines};
export const protocol=Object.freeze({processes:3,warmups:8,samples:31,requestsPerBatch:128,concurrency:[1,16],keepAlive:true,serverWorkers:1,resourceBatches:31,rssSampleMs:50});
export const workloads=Object.freeze(['bytes','stream','channel'].flatMap(mode=>protocol.concurrency.map(concurrency=>({scenario:`client_upload_${mode}`,name:`Owned ${mode} client upload`,method:'POST',path:`/${mode}`,id:`http_client_upload_${mode}_c${concurrency}`,concurrency}))));
export const sourceFiles={'native':'client-uploads.tsr','node-tsc':'client-uploads.ts','rust-hand':'client-uploads.rs'};
export const hash=bytes=>createHash('sha256').update(bytes).digest('hex');export function sourceHashes(){return Object.fromEntries(Object.entries(sourceFiles).map(([engine,file])=>[engine,hash(fs.readFileSync(new URL(file,import.meta.url)))]));}
export function assertManifest(manifest){assert.deepEqual(manifest.protocol,protocol);assert.deepEqual(manifest.workloads,workloads);assert.deepEqual(manifest.sources,sourceHashes());}
export function expected(id){const count=id%3===1?64:id%3===2?4096:65537;let sum=0n;for(let offset=0;offset<count;offset++)sum=(sum*131n+BigInt((offset*17+id)%251))%1000000007n;return Buffer.from(`${count},${sum}`);}
export function cases(workload){assert(workloads.some(row=>row.id===workload.id));return Array.from({length:128},(_,index)=>({method:'POST',path:`${workload.path}/${index+1}`,status:200,type:'text/plain',expected:expected(index+1)}));}
export const rejectionCases=Object.freeze([]);
