import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import {hash,CONTROL_SOURCE_FILES} from './guards.mjs';
import {checkMatrix,checkSse,checkLifetime,checkWs} from './oracle.mjs';
import {DEFAULT_LIMIT} from '../../quiet-machine.mjs';
const controls=['rust-backend','node-backend'];
const labels={'native':'native','rust-backend':'rust-hand','node-backend':'node-ws'};
export const workloadId=id=>'http_realtime_'+id.replace('-c','_c');
export function commonResult(report,engines,timestamp,identities){
 assert.equal(new Date(timestamp).toISOString(),timestamp);
 assert.deepEqual(report.protocol,{processes:3,warmups:8,samples:31,iterations:128,concurrency:[1,16],resourceBatches:31,rssSampleMs:50});
 const cases=[...new Set(report.network.filter(row=>engines.includes(row.engine)).map(row=>row.case))];
 assert(cases.length&&cases.every(id=>/^(sse|text|binary)-c(1|16)$/.test(id)));
 const expected=Object.fromEntries(cases.map(id=>[id,128*(id.startsWith('sse')?90:id.startsWith('text')?7:4)]));
 const resources=new Map();for(const row of report.resources){if(!engines.includes(row.engine))continue;const key=JSON.stringify([row.engine,row.case,row.process]);assert(!resources.has(key),'Duplicate resource row');assert(Number.isFinite(row.cpuMs)&&row.cpuMs>=0&&row.requests===31*128&&row.peakRssBytes>0&&row.rssSamples>=2&&Number.isFinite(row.retainedRssDelta));resources.set(key,row);}
 const execution=[];
 for(const engine of engines){checkMatrix(report.network.filter(row=>row.engine===engine),{cases,processes:3,samples:31,iterations:128,expected});for(const row of report.network.filter(row=>row.engine===engine)){const key=JSON.stringify([engine,row.case,row.process]);const resource=resources.get(key);assert(resource,'Missing resource counterpart');resources.delete(key);execution.push({workload:workloadId(row.case),engine:labels[engine],processIndex:row.process,durations:row.durations,latencies:row.latencies,resources:resource});}}
 assert.equal(resources.size,0,'Unmatched resource rows');
 assert(report.quietLoads.length&&report.quietLoads.every(load=>Number.isFinite(load)&&load<DEFAULT_LIMIT),'Missing or busy quiet-machine evidence');
 return{schemaVersion:1,timestamp,verifiedOnly:false,semanticEpoch:'http-realtime-v1',measuredEngines:engines.map(engine=>labels[engine]),manifest:{workloads:cases.map(id=>({id:workloadId(id),iterations:128,operationsPerIteration:1}))},protocol:report.protocol,measurementScope:'128 full six-event SSE bodies or typed WebSocket echoes; build/startup excluded; CPU/RSS separate. Rust control retains backend Utf8Bytes; native materializes owned strings.',quietMachine:{allowBusy:false,loadLimit:DEFAULT_LIMIT,maxOneMinuteLoad:Math.max(...report.quietLoads)},identities,execution};
}
export function referenceControls(file,{root,prepared,nodeProof,protocol,selected}){
 const proof=JSON.parse(fs.readFileSync(file));assert.equal(proof.kind,'realtime-control-reference');
 const report=JSON.parse(fs.readFileSync(proof.result));const scope=JSON.parse(fs.readFileSync(proof.scope));
 assert.equal(hash(fs.readFileSync(proof.result)),proof.resultSha256);assert.equal(hash(fs.readFileSync(proof.scope)),proof.scopeSha256);
 assert.equal(scope.status,'terminal-success');const stage=scope.stages.filter(row=>row.label==='profile');assert.equal(stage.length,1);assert.equal(stage[0].code,0);assert.equal(stage[0].stop,null);assert.deepEqual(stage[0].remaining,[]);
 assert.equal(proof.timestamp,new Date(scope.started*1000).toISOString(),'Original scope timestamp required');
 assert.equal(report.status,'terminal-success-with-unresolved-sse-semantics');assert.equal(report.publicBackendAccepted,false);assert.deepEqual(report.protocol,protocol);assert.equal(report.node,process.version);assert.equal(report.ws,'8.18.3');assert.equal(hash(fs.readFileSync(nodeProof)),proof.nodeProofSha256);assert.equal(hash(fs.readFileSync(prepared.binary)),proof.binarySha256);assert.deepEqual(report.closure,prepared.graph);
 for(const name of CONTROL_SOURCE_FILES)assert.equal(report.sourceSeal.sources[name],hash(fs.readFileSync(path.join(root,name))),'Changed control source '+name);
 const fixture=JSON.parse(fs.readFileSync(path.join(root,'fixtures.json')));assert.equal(report.groups.length,5);
 for(const name of ['sse-format','sse-lifetime','ws-controls']){const rows=report.groups.filter(row=>row.group===name);assert.equal(rows.length,1);if(name==='sse-format')checkSse(rows[0].rows,fixture);else if(name==='sse-lifetime')checkLifetime(rows[0].rows);else checkWs(rows[0].rows);}
 for(const engine of controls){const rows=report.groups.filter(row=>row.group==='active-pending-send-shutdown'&&row.engine===engine);assert.equal(rows.length,1);assert.equal(rows[0].pending_send,true);assert.equal(rows[0].active,0);}
 assert.equal(report.network.length,36);assert.equal(report.resources.length,36);assert(report.network.every(row=>controls.includes(row.engine))&&report.resources.every(row=>controls.includes(row.engine)));
 assert.deepEqual([...new Set(report.network.map(row=>row.case))].sort(),['binary-c1','binary-c16','sse-c1','sse-c16','text-c1','text-c16']);
 commonResult(report,controls,proof.timestamp,{historicalResultSha256:proof.resultSha256});
 const filtered={...report,network:report.network.filter(row=>selected.includes(row.case)),resources:report.resources.filter(row=>selected.includes(row.case))};
 const common=commonResult(filtered,controls,proof.timestamp,{historicalResultSha256:proof.resultSha256,historicalScopeSha256:proof.scopeSha256,binarySha256:proof.binarySha256,nodeProofSha256:proof.nodeProofSha256,sourceSeal:report.sourceSeal});
 return{report:filtered,common,proof,snapshot:Object.fromEntries([file,proof.result,proof.scope].map(name=>[path.resolve(name),hash(fs.readFileSync(name))]))};
}
export function selfCheckPublicResults(){
 const protocol={processes:3,warmups:8,samples:31,iterations:128,concurrency:[1,16],resourceBatches:31,rssSampleMs:50};
 const network=Array.from({length:3},(_,process)=>({engine:'native',case:'binary-c1',process,durations:Array(31).fill(1),latencies:Array(31*128).fill(1),checksums:Array(31).fill(512),expected:512}));
 const resources=network.map(row=>({engine:row.engine,case:row.case,process:row.process,cpuMs:1,requests:31*128,peakRssBytes:1,rssSamples:2,retainedRssDelta:0}));
 const good={protocol,network,resources,quietLoads:[0]},at='2026-11-07T00:00:00.000Z';assert.equal(commonResult(good,['native'],at,{}).execution.length,3);
 for(const mutate of [v=>v.resources.pop(),v=>v.resources.push(v.resources[0]),v=>v.network[0].latencies.pop(),v=>v.network[0].checksums[0]++,v=>v.network[0].case='unknown',v=>v.quietLoads=[Infinity]]){const value=structuredClone(good);mutate(value);assert.throws(()=>commonResult(value,['native'],at,{}));}
}
