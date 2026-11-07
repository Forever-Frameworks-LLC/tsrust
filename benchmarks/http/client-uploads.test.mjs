import test from 'node:test';import assert from 'node:assert/strict';import fs from 'node:fs';
import {protocol,workloads,cases,assertManifest,validateResponse,expected} from './client-uploads-contract.mjs';
test('client upload peers preserve original HTTP protocol and exact independent observations',()=>{
 assertManifest(JSON.parse(fs.readFileSync(new URL('./client-uploads-manifest.json',import.meta.url))));assert.equal(workloads.length,6);assert.deepEqual([protocol.processes,protocol.warmups,protocol.samples],[3,8,31]);
 for(const workload of workloads){const rows=cases(workload);assert.equal(rows.length,128);for(let i=0;i<rows.length;i++){assert.equal(rows[i].path,`${workload.path}/${i+1}`);validateResponse(rows[i],200,{'content-type':'text/plain'},expected(i+1));const changed=Buffer.from(rows[i].expected);changed[changed.length-1]^=1;assert.throws(()=>validateResponse(rows[i],200,{'content-type':'text/plain'},changed));}}
 assert.notDeepEqual(expected(1),expected(2));for(const file of ['client-uploads.tsr','client-uploads.ts','client-uploads.rs']){const source=fs.readFileSync(new URL(file,import.meta.url),'utf8');for(const token of ['65537','251','131','1000000007'])assert(source.replaceAll('_','').includes(token),`${file}: ${token}`);}
});
