import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {protocol,workloads,sourceFiles,sourceHashes,assertManifest,cases,rejectionCases,validateResponse} from './requests-contract.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('./requests-manifest.json',import.meta.url)));
test('HTTP scenarios retain three source identities and the original sampling protocol',()=>{
 assertManifest(manifest);assert.deepEqual(protocol.concurrency,[1,16]);assert.equal(protocol.processes*protocol.samples,93);assert.equal(protocol.warmups,8);assert.equal(protocol.requestsPerBatch,128);assert.equal(workloads.length,8);assert.equal(new Set(workloads.map(row=>row.scenario)).size,4);
 assert.throws(()=>assertManifest({...manifest,sources:{...manifest.sources,native:'changed'}}));assert.throws(()=>assertManifest({...manifest,protocol:{...protocol,requestsPerBatch:1}}));
 for(const file of Object.values(sourceFiles)){const source=fs.readFileSync(new URL(file,import.meta.url),'utf8');for(const witness of ['/text','/json','/parse','/sqlite','CREATE TABLE items','SELECT id, label, value FROM items WHERE id = ?',':memory:','65536'])assert(source.includes(witness),`${file}: ${witness}`);}
});
test('every response has independent complete bytes and varying JSON/database inputs',()=>{
 for(const row of workloads){const inputs=cases(row);assert.equal(inputs.length,128);for(const input of inputs)validateResponse(input,200,{'content-type':input.type},Buffer.from(input.expected));
 if(row.scenario==='parse_json'){assert.equal(JSON.parse(inputs[0].body).value,0);assert.equal(JSON.parse(inputs[127].body).value,381);assert.deepEqual(JSON.parse(inputs[127].expected),{id:128,label:'héllo 🦀!',value:382});}
 if(row.scenario==='sqlite_json'){assert.deepEqual(JSON.parse(inputs[0].body),{id:1});assert.deepEqual(JSON.parse(inputs[127].expected),{id:128,label:'row héllo 128',value:384});}
 }
});
test('status, content type and every response byte are required, including errors',()=>{
 for(const input of [...cases(workloads[0]),...rejectionCases]){validateResponse(input,input.status,{'content-type':input.type},input.expected);assert.throws(()=>validateResponse(input,500,{'content-type':input.type},input.expected));assert.throws(()=>validateResponse(input,input.status,{'content-type':'wrong'},input.expected));const changed=Buffer.from(input.expected);changed[0]^=1;assert.throws(()=>validateResponse(input,input.status,{'content-type':input.type},changed));}
 assert.deepEqual([...new Set(rejectionCases.map(row=>row.status))],[400,404]);
});
test('prepared sources, build/timing boundaries, common client and owned server cleanup stay explicit',()=>{
 const runner=fs.readFileSync(new URL('./requests-run.mjs',import.meta.url),'utf8');assert(runner.indexOf('Build Tsrust')<runner.indexOf('const execution='));assert(runner.includes('waitForQuietMachine()'));assert(runner.includes('maxSockets:concurrency'));assert(runner.includes("child.kill('SIGINT')"));assert(runner.includes("child.kill('SIGKILL')"));assert(runner.includes('agent.destroy()'));assert(runner.includes('validateResponse(test,response.statusCode'));assert.deepEqual(Object.keys(sourceHashes()),['native','node-tsc','rust-hand']);
});
