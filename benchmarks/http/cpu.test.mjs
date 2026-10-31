import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {protocol,workloads,sourceFiles,cases,rejectionCases,validateResponse,assertManifest} from './cpu-contract.mjs';
import {parseSnapshot} from './telemetry.mjs';
test('CPU rendering preserves independent varying inputs, complete outputs and one-worker budget',()=>{
 const m=JSON.parse(fs.readFileSync(new URL('./cpu-manifest.json',import.meta.url)));assertManifest(m);
 assert.equal(protocol.serverWorkers,1);assert.equal(protocol.processes*protocol.samples,93);
 for(const w of workloads){const all=cases(w);assert.equal(all.length,128);assert(new Set(all.map(x=>JSON.parse(x.expected).checksum)).size>1);for(const x of all)validateResponse(x,200,{'content-type':x.type},x.expected);}
 for(const file of Object.values(sourceFiles)){const s=fs.readFileSync(new URL(file,import.meta.url),'utf8');assert(s.includes('/render'));assert(s.includes('256'));assert(!s.includes('cpu-oracle'));}
 for(const x of rejectionCases)validateResponse(x,400,{'content-type':'text/plain'},x.expected);
});
test('process CPU time and resident memory have explicit units',()=>{
 assert.deepEqual(parseSnapshot('0:01.23 2048'),{cpuMs:1230,rssBytes:2097152});
 assert.deepEqual(parseSnapshot('1:02:03.50 1'),{cpuMs:3723500,rssBytes:1024});
 assert.deepEqual(parseSnapshot('2-01:02:03.50 1'),{cpuMs:176523500,rssBytes:1024});
 assert.throws(()=>parseSnapshot('missing'));
});
test('latency records individual completed bodies and resource polling is a separate pass',()=>{
 const s=fs.readFileSync(new URL('./requests-run.mjs',import.meta.url),'utf8');
 assert(s.includes('latencies.push(elapsed)'));assert(s.includes('resources={cpuMs:after.cpuMs-before.cpuMs'));
 assert(s.indexOf('const timer=setInterval')>s.indexOf('durations.push'));
 assert(s.includes('TOKIO_WORKER_THREADS=String(protocol.serverWorkers)'));
});
