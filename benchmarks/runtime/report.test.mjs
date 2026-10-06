import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {render, median, statistics, updateReport, buildReport} from './report.mjs';
function fixture() {
 const ids = ['native', 'node-tsc'];
 const data = {
  timestamp: 'test', verifiedOnly: false,
  manifest: {gate: 31, processes: 3, samples: 31, warmups: 8, startupSamples: 3, compileSamples: 3, workloads: [{id: 'scalar_control', iterations: 1}]},
  machine: {cpu: 'Test', architecture: 'arm64', logicalCPUs: 1, totalMemoryBytes: 2**30, platform: 'test', release: 'test'},
  versions: {node: 'test', bun: 'test', deno: 'test', typescript: 'test', tsx: 'test', 'ts-node': 'test', rust: 'test'},
  correctness: {casesPerWorkload: 5, checksums: {scalar_control: 7}}, sources: [{id: 'scalar_control'}], execution: [], compilation: [], startup: []
 };
 for (const id of ids) {
  for (let processIndex=0; processIndex<3; processIndex++) data.execution.push({workload: 'scalar_control', engine: id, processIndex, checksum: 7, durations: Array(31).fill(id === 'native' ? [1,2,50][processIndex] : 1)});
  for(let sample=0; sample<3; sample++) data.startup.push({engine: id, sample, ms: 1});
 }
 for(const phase of ['frontend-check','frontend-emit','rustc-optimized','tsc-check-emit']) data.compilation.push({workload: 'scalar_control', phase, durations: [1,2,3]});
 return data;
}
function report(data) {
 const folder = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-report-test-'));
 try { const file = path.join(folder, 'test.json'); fs.writeFileSync(file, JSON.stringify(data)); return render(file, {append: false}); }
 finally { fs.rmSync(folder, {recursive: true}); }
}
test('ops/s converts each raw batch before computing min/avg/max', () => {
 assert.equal(median([1,2,50]), 2);
 assert.equal(median([2,4]), 3);
 const text=report(fixture());
 assert.match(text, /TSRust native \(generated Rust\) \| 93 \| 20\.00 \| 506\.67 \| 1,000/);
 assert.match(text, /Node after tsc \| 93 \| 1,000 \| 1,000 \| 1,000/);
});
test('report rejects changed output, missing batches, duplicate processes, and negative timings', () => {
 for(const mutate of [
  d=>d.execution[0].checksum++,
  d=>d.execution[0].durations.pop(),
  d=>d.execution[0].processIndex=1,
  d=>d.execution[0].durations[0]=-1
 ]) { const data=fixture(); mutate(data); assert.throws(()=>report(data), /Incomplete or invalid results/); }
});
test('a selected-workload snapshot does not claim unmeasured manifest entries', () => {
 const data=fixture();data.manifest.workloads.push({id: 'unmeasured', iterations: 99});
 const text=report(data);
 assert.match(text,/id="runtime-scalar-control"/);assert.doesNotMatch(text,/unmeasured/);
});
test('verification-only runs cannot produce performance claims', () => {
 const data=fixture();data.verifiedOnly=true;
 assert.throws(()=>report(data),/Verification is not a timed baseline/);
});

test('statistics reject missing/invalid timings and preserve signed heap deltas', () => {
 for (const values of [[], [NaN], [Infinity], [-1]]) assert.throws(() => statistics(values), /Incomplete or invalid results/);
 assert.deepEqual(statistics([-8, 0, 2], {signed: true}), {min: -8, avg: -2, max: 2, samples: 3});
});
test('regeneration is idempotent, preserves raw records, and fails before replacing a valid report', () => {
 const folder = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-records-test-'));
 try {
  const data=fixture();data.schemaVersion=1;
  const file=path.join(folder,'run.json'), output=path.join(folder,'BENCHMARKS.md');
  const raw=JSON.stringify(data);fs.writeFileSync(file,raw);
  updateReport({recordsDirectory: folder, output});
  const original=fs.readFileSync(output,'utf8');
  updateReport({recordsDirectory: folder, output});
  assert.equal(fs.readFileSync(output,'utf8'), original);
  assert.equal(fs.readFileSync(file,'utf8'),raw);
  data.execution[0].durations.pop();fs.writeFileSync(file,JSON.stringify(data));
  assert.throws(()=>updateReport({recordsDirectory: folder,output}),/Incomplete or invalid results/);
  assert.equal(fs.readFileSync(output,'utf8'),original);
 } finally {fs.rmSync(folder,{recursive:true});}
});
test('allocation tables keep native counts and signed Node heap deltas as separate metrics', () => {
 const folder=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-alloc-report-test-'));
 try {
  const data={timestamp:'test', schemaVersion:1, machine:{cpu:'Test',architecture:'arm64',release:'test'}, versions:{node:'test',typescript:'test',rust:'test'}, probes:[{id:'empty_control', specification:'Empty control'}], rows:[{probe:'empty_control',n:1,samples:[-8,0,2].map(delta=>({native:{checksum:0,allocations:0,reallocations:0,deallocationsDuringWork:0,requestedBytes:0,retainedRequestedBytes:0,peakRequestedBytes:0,liveBytesAfterDrop:0},node:{checksum:0,retainedHeapDelta:delta}}))}]};
  const file=path.join(folder,'allocation.json');fs.writeFileSync(file,JSON.stringify(data));
  const text=render(file,{append:false});
  assert.match(text,/Node after tsc \| 1 \| Retained V8 heap delta \| B \| -8 \| -2 \| 2/);
  assert.match(text,/TSRust native \| 1 \| New allocations \| calls \| 0/);
 } finally {fs.rmSync(folder,{recursive:true});}
});

test('partial newer runs keep unmeasured workloads and compilation results without duplicate anchors', () => {
 const old = fixture(); old.timestamp='2026-01-01'; old.schemaVersion=1;
 const second='closure_factory';
 old.sources.push({id:second});old.manifest.workloads.push({id:second,iterations:1});old.correctness.checksums[second]=7;
 old.execution.push(...old.execution.map(row=>({...row,workload:second})));
 old.compilation.push(...old.compilation.map(row=>({...row,workload:second})));
 const next=fixture();next.timestamp='2026-01-02';next.schemaVersion=1;
 const text=buildReport([{file:'old.json',data:old},{file:'new.json',data:next}]);
 assert.equal((text.match(/id="runtime-scalar-control"/g)||[]).length,1);
 assert.equal((text.match(/id="runtime-closure-factory"/g)||[]).length,1);
 assert.equal((text.match(/id="startup"/g)||[]).length,1);
 assert.match(text,/closure_factory \| Rust optimized compilation/);
 assert.match(text,/scalar_control \| Rust optimized compilation/);
 assert.match(text,/<details>/);
});
test('zero-duration samples do not invent infinite throughput', () => {
 const data=fixture();data.execution[0].durations[0]=0;
 const text=report(data);
 assert.match(text,/TSRust native \(generated Rust\) \| 93 \| — \| — \| —/);
 assert.doesNotMatch(text,/Infinity/);
});

test('semantic epochs exclude old workload and allocation rates while preserving raw history', () => {
 const old=fixture(); old.timestamp='2026-01-01';
 const fresh=fixture(); fresh.timestamp='2026-01-02'; fresh.semanticEpoch='rust-native-utf8-v1';
 old.sources[0].id='older_only'; old.manifest.workloads[0].id='older_only'; old.correctness.checksums.older_only=7;
 for(const row of [...old.execution,...old.compilation]) row.workload='older_only';
 const records=[{file:'old.json',data:old},{file:'new.json',data:fresh}];
 const text=buildReport(records);
 assert.match(text,/Current semantic epoch: \*\*rust-native-utf8-v1\*\*/);
 assert.doesNotMatch(text,/id="runtime-older-only"/);
 assert.match(text,/old.json/); assert.match(text,/historical-utf16-unversioned/);
 const pending=buildReport([{file:'old.json',data:old}],{semanticEpoch:'rust-native-utf8-v1'});
 assert.match(pending,/No timed runtime results/); assert.doesNotMatch(pending,/id="runtime-older-only"/);
});

test('known workloads and primitive probes have one stable category; unknown operations use Other', async () => {
 const {reportCategories,reportCategory}=await import('./report.mjs');
 const manifest=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url)));
 const probes=JSON.parse(fs.readFileSync(new URL('./allocations/manifest.json',import.meta.url)));
 for(const row of manifest.workloads) {
  const assigned=reportCategory(row.id);
  if (/^(number_|math_)/.test(row.id)) assert.equal(assigned,'numbers',row.id);
  else if (row.id.startsWith('string_')) assert.equal(assigned,'strings',row.id);
  else if (/^(higher_|declared_|nullable_callable_|optional_parameter_|default_parameter_|rest_parameter_|call_spread_)/.test(row.id)) assert.equal(assigned,'callbacks',row.id);
  else if (/^(?:tuple_types_|array_entries_|array_keys_int$|array_values_int$|array_from_|array_locale_string_|array_literal_spread_|array_reduce(?:_right)?_|array_for_each_|array_concat_|array_fill_|array_copy_within_|array_to_reversed_|array_with_|array_to_spliced_|array_last_index_of_)/.test(row.id)) assert.equal(assigned,'arrays',row.id);
  else assert.equal(reportCategories.filter(group=>group.id===assigned&&group.workloads.includes(row.id)).length,1,row.id);
 }
 for(const probe of probes) assert.equal(reportCategories.filter(group=>group.probes.includes(probe.id)).length,1,probe.id);
 for(const id of ['array_fill_int','array_copy_within_recursive','array_to_reversed_int','array_with_recursive','array_to_spliced_recursive','array_last_index_of_bounded_string']) {
  assert.equal(reportCategory(id),'arrays',id);
  assert.equal(reportCategory(id,{allocation:true}),'other',id);
 }
 assert.equal(reportCategory('new_unclassified_operation'),'other');
 assert.equal(reportCategory('new_unclassified_probe',{allocation:true}),'other');
});
test('grouping places primitive storage beside its operation type without losing dates, rates or unknown rows', () => {
 const number=fixture();number.timestamp='2026-01-01';
 const string=fixture();string.timestamp='2026-01-02';
 string.sources[0].id='string_equality';string.manifest.workloads[0].id='string_equality';string.correctness.checksums.string_equality=7;
 for(const row of [...string.execution,...string.compilation]) row.workload='string_equality';
 const unknown=fixture();unknown.timestamp='2026-01-03';unknown.sources[0].id='unknown_operation';unknown.manifest.workloads[0].id='unknown_operation';unknown.correctness.checksums.unknown_operation=7;
 for(const row of [...unknown.execution,...unknown.compilation]) row.workload='unknown_operation';
 const allocation={timestamp:'2026-01-04',machine:{cpu:'Test',architecture:'arm64'},probes:[{id:'int_value',specification:'Integer storage'},{id:'string_value',specification:'String storage'}],rows:['int_value','string_value'].map(probe=>({probe,n:1,samples:Array.from({length:3},()=>({native:{checksum:7,liveBytesAfterDrop:0,allocations:0,reallocations:0,retainedRequestedBytes:1024},node:{checksum:7,retainedHeapDelta:-8}}))}))};
 const text=buildReport([{file:'numbers.json',data:number},{file:'strings.json',data:string},{file:'unknown.json',data:unknown},{file:'allocation.json',data:allocation}]);
 const numberStart=text.indexOf('<a id="group-numbers"'),stringStart=text.indexOf('<a id="group-strings"'),otherStart=text.indexOf('<a id="group-other"');
 assert.ok(numberStart<text.indexOf('<a id="runtime-scalar-control"')&&text.indexOf('<a id="allocation-int-value"')<stringStart);
 assert.ok(stringStart<text.indexOf('<a id="runtime-string-equality"')&&text.indexOf('<a id="allocation-string-value"')<otherStart);
 assert.ok(otherStart<text.indexOf('<a id="runtime-unknown-operation"'));
 for(const id of ['scalar-control','string-equality','unknown-operation']) assert.equal((text.match(new RegExp(`id="runtime-${id}"`,'g'))||[]).length,1);
 for(const id of ['int-value','string-value']) assert.equal((text.match(new RegExp(`id="allocation-${id}"`,'g'))||[]).length,1);
 assert.match(text,/2026-01-01/);assert.match(text,/2026-01-02/);assert.match(text,/Min ops\/s \| Avg ops\/s \| Max ops\/s/);
 assert.match(text,/Retained requested storage \| KiB \| 1 \| 1 \| 1/);
 assert.match(text,/TSRust native \(generated Rust\) \| 93 \| 20\.00 \| 506\.67 \| 1,000/);
});

test('execution-only updates rates while retaining actual auxiliary dates without invented compilation rows',()=>{
 const full=fixture();full.timestamp='2026-01-01';
 const execution=fixture();execution.timestamp='2026-01-02';execution.measurementScope='execution-only';execution.startup=[];execution.compilation=[];
 for(const row of execution.execution) row.durations=Array(31).fill(1);
 execution.sources.push({id:'new_execution'});execution.manifest.workloads.push({id:'new_execution',iterations:1});execution.correctness.checksums.new_execution=7;
 execution.execution.push(...execution.execution.map(row=>({...row,workload:'new_execution'})));
 const text=buildReport([{file:'full.json',data:full},{file:'execution.json',data:execution}]);
 assert.match(text,/execution only/);assert.match(text,/Startup baseline: 2026-01-01/);
 assert.match(text,/Compilation baseline dates and sources/);
 assert.match(text,/scalar_control \| Rust optimized compilation/);
 assert.doesNotMatch(text,/new_execution \| Rust optimized compilation/);
 assert.match(text,/TSRust native \(generated Rust\) \| 93 \| 1,000 \| 1,000 \| 1,000/);
 assert.equal((text.match(/id="runtime-scalar-control"/g)||[]).length,1);
 assert.equal((text.match(/id="runtime-new-execution"/g)||[]).length,1);
 const standalone=buildReport([{file:'execution.json',data:execution}]);
 assert.doesNotMatch(standalone,/id="startup"|id="compilation"/);
 for(const key of ['startup','compilation']) {
  const invalid=structuredClone(execution);invalid[key]=full[key];
  assert.throws(()=>buildReport([{file:'invalid.json',data:invalid}]),/Incomplete or invalid results/);
 }
});

test('Number scalar memory stays in Numbers with normalized actual storage and signed Node deltas',()=>{
 const data={timestamp:'number-test',semanticEpoch:'rust-native-utf8-v1',machine:{cpu:'Test',architecture:'arm64'},probes:[{id:'number_value',result:'number',specification:'Runtime f64 value'}],rows:[{probe:'number_value',n:8,samples:[-16,0,16].map(delta=>({native:{checksum:7,exactBits:'n:402f000000000000',allocations:0,reallocations:0,retainedRequestedBytes:0,liveBytesAfterDrop:0},node:{checksum:7,exactBits:'n:402f000000000000',retainedHeapDelta:delta}}))}]};
 const text=buildReport([{file:'number-memory.json',data}],{semanticEpoch:'rust-native-utf8-v1'});
 assert.match(text,/#group-numbers/);assert.equal((text.match(/<a id="allocation-number-value">/g)||[]).length,1);
 assert.match(text,/TSRust native \| 8 \| Retained requested storage \| B \| 0 \| 0 \| 0/);
 assert.match(text,/Node after tsc \| 8 \| Retained V8 heap delta \| B \| -16 \| 0 \| 16/);
});

test('Number runtime report rejects missing or changed bit tokens even when numeric checksum agrees',()=>{
 const data=fixture();data.measurementScope='execution-only';data.startup=[];data.compilation=[];
 data.sources[0].id='number_add';data.manifest.workloads[0]={id:'number_add',result:'number',iterations:1};
 data.correctness.checksums={number_add:7};data.correctness.exactBits={number_add:'n:8000000000000000'};data.numberResultEncoding='ieee-f64-bits-v1';
 for(const row of data.execution){row.workload='number_add';row.exactBits='n:8000000000000000';}
 assert.match(report(data),/number add/);
 data.execution[0].exactBits='n:0000000000000000';assert.throws(()=>report(data),/Number/);
 data.execution[0].exactBits='n:8000000000000000';delete data.correctness.exactBits;assert.throws(()=>report(data),/Number/);
});

test('complete-shape array report rejects changedrank or missingencoding despite equal digest',()=>{
 const data=fixture();data.measurementScope='execution-only';data.startup=[];data.compilation=[];
 data.sources[0].id='array_slice_nested';data.manifest.workloads[0]={id:'array_slice_nested',result:'int[][][]',resultEncoding:'array-shape-scalar-v1',iterations:1};
 data.correctness.checksums={array_slice_nested:7};data.correctness.exactBits={array_slice_nested:'t:[[],[[]]]'};data.arrayResultEncodings=['array-shape-scalar-v1'];
 for(const row of data.execution){row.workload='array_slice_nested';row.exactBits='t:[[],[[]]]';}
 assert.match(report(data),/array slice nested/);
 data.execution[0].exactBits='t:[[[]],[]]';assert.throws(()=>report(data),/array/);
 data.execution[0].exactBits='t:[[],[[]]]';delete data.arrayResultEncodings;assert.throws(()=>report(data),/array/);
});
