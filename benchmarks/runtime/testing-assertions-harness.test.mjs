import {withCurrentSourceFingerprints} from './manifest-metadata-contract.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {expectedTestingAssertionChecksum} from './testing-assertions-oracle.mjs';
import {assertTestingAssertionSource} from './testing-assertions-source-contract.mjs';
import {assertABI,adaptRustDriver} from './adapters.mjs';
test('assertion observer preserves independent hand cells',()=>{
 assert.equal(expectedTestingAssertionChecksum(0,0),0);
 assert.equal(expectedTestingAssertionChecksum(1,3),9);
 assert.equal(expectedTestingAssertionChecksum(2,3),1179);
 assert.throws(()=>expectedTestingAssertionChecksum(-1,3));
});
test('assertion workload uses real APIs and exact current sources',()=>{
 const [row]=withCurrentSourceFingerprints(JSON.parse(fs.readFileSync(new URL('testing-assertions-manifest-entries.json',import.meta.url))));
 for(const [engine,extension] of [['native','tsr'],['typescript','ts']]){
  const source=fs.readFileSync(new URL('workloads/'+row.id+'.'+extension,import.meta.url),'utf8');
  assert.doesNotThrow(()=>assertTestingAssertionSource(source,row,engine));
  assert.throws(()=>assertTestingAssertionSource(replaceCodeSnippet(source, 'strictEqual(actual, expected);', ''),row,engine));
 }
 const emitted='fn function12(value0: i64, value1: i64) -> i64 {0}\nfn main() {}';
 assert.equal(assertABI(emitted,row).functionName,'function12');
 assert.match(adaptRustDriver('fn benchmark_checksum(value: i64) -> i64 { value }\nfn main(){function0(1,2);}',row,{emitted}),/function12\(1,2\)/);
});

test('deep assertion controls retain both operations and exact reference sources',async()=>{
 const {assertTestingDeepReference,testingDeepReferenceOperations}=await import('./testing-assertions-source-contract.mjs');const {createHash}=await import('node:crypto');const hash=source=>createHash('sha256').update(source).digest('hex');const manifest=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url)));assert.equal(manifest.processes,3);assert.equal(manifest.samples,31);assert.equal(manifest.warmups,8);
 for(const id of ['testing_deep_data','testing_deep_collections']){const row=manifest.workloads.find(row=>row.id===id);assert(row);assert.equal(row.measured,false);for(const [engine,key,fingerprint]of [['native','source','sourceSha256'],['typescript','counterpart','counterpartSha256']]){const source=fs.readFileSync(new URL(row[key],import.meta.url),'utf8');assertTestingAssertionSource(source,row,engine);assert.throws(()=>assertTestingAssertionSource(source+' ',row,engine));const operation=row.methodExpressions[engine][0];const changed=replaceCodeSnippet(source,operation,'omitted_assertion');assert.notEqual(changed,source);assert.throws(()=>assertTestingAssertionSource(changed,{...row,[fingerprint]:hash(changed)},engine));}const source=fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8');assertTestingDeepReference(source,row);adaptRustDriver('fn benchmark_checksum(value: i64) -> i64 {value} fn main(){function0(1,2);}',row,{reference:true,emitted:source});for(const operation of testingDeepReferenceOperations[id]){const changed=replaceCodeSnippet(source,operation,'omitted_assertion');assert.notEqual(changed,source);assert.throws(()=>assertTestingDeepReference(changed,{...row,rustReferenceSha256:hash(changed)}));}}
});
test('reporter oracle requires complete ordered output and original call sites',async()=>{
 const {assertFilteredReporter}=await import('./testing-everyday-reporter-oracle.mjs');const source='console.log("setup"); console.error("fixture"); console.log("teardown");';const calls=[...source.matchAll(/console\.(?:log|error)\(/gu)];const outputs=[['stdout','setup\n'],['stderr','é\n\t"quoted"\n'],['stdout','teardown\n']].map(([stream,text],index)=>({event:'output',name:'selected',stream,text,location:`input.tsr:1:${calls[index].index+1}`}));const rows=[outputs[0],{event:'test',name:'selected > literal[.]',status:'passed',messages:[],durationMs:1},outputs[1],outputs[2],{event:'summary',passed:1,failed:0,hookFailures:0,skipped:2,filesPassed:1,filesFailed:0,durationMs:1}];const encode=rows=>rows.map(row=>JSON.stringify(row)).join('\n');assertFilteredReporter(encode(rows),source);for(const mutate of [r=>r.splice(0,1),r=>r.splice(0,0,r[0]),r=>r[2].stream='stdout',r=>r[2].location='input.tsr:1:1',r=>r[1].name='excluded > literal[.]']){const changed=structuredClone(rows);mutate(changed);assert.throws(()=>assertFilteredReporter(encode(changed),source));}
});
