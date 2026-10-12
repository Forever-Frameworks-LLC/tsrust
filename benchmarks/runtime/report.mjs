import {isCopying,assertCopyingPolicy,validateCopyingObservation} from './copying-array-observation.mjs';
import {assertCopyingSource,assertCopyingTSSource} from './copying-array-result-abi.mjs';
import {isRange,assertRangePolicy,validateRangeObservation} from './range-mutation-observation.mjs';
import {assertRangeSource,assertRangeTSSource} from './range-mutation-result-abi.mjs';
import {isHigher,assertHigherPolicy,validateHigherObservation} from './higher-order-observation.mjs';
import {isDeclared,assertDeclaredPolicy,validateDeclaredObservation} from './declared-value-observation.mjs';
import {isReduction,isReductionSchema} from './reduction-observation.mjs';
import {isConcat,assertConcatPolicy,validateConcatObservation} from './concat-observation.mjs';
import {isApproximate,restoreApproximateReference} from './approximate-harness.mjs';
import {validateApproximatePointProbe,validateApproximateMeasurement} from './approximate-scalar.mjs';
import {requiresExactResult,isSliceResult,validateExactObservation} from './result-observation.mjs';
import {isNumberResult,validateNumberObservation} from './number-observation.mjs';
import fs from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {loadReportRecords} from './report-records.mjs';
const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
const recordsFolder = path.join(here, 'results');
const modes = {
 native: 'TSRust native (generated Rust)',
 'node-tsc': 'Node after tsc',
 'node-ts': 'Node type stripping',
 tsx: 'tsx / Node',
 'ts-node-check': 'ts-node / Node (checking)',
 'ts-node-transpile': 'ts-node / Node (transpile only)',
 bun: 'Bun', deno: 'Deno'
};
const examples = {
 std_path_relative: 'values.push(relative(from, to));',
 std_path_join_ascii: 'values.push(join("/tmp", part, "archive.txt"));',
 std_path_join_empty_roots: 'values.push(join(first, "", "child", "..", ""));',
 std_path_join_many_unicode: "values.push(join(...paths));",
 std_os_endianness: "values.push(endianness());",
 std_os_platform: "values.push(platform());",
 std_os_arch: "values.push(arch());",
 string_char_at_unicode_middle_literal: 'literal.charAt(midpoint - 8 + (i * 13 + seed) % 17)',
 string_length: 'values[(i + seed) % 2].length',
 string_slice_ascii: 'input.slice(start, end)',
 string_slice_unicode: 'input.slice(start, end)',
 string_char_at: 'input.charAt(index)',
 scalar_control: 'state = (state * 17 + 13) % 1009;',
 ascii_strings: 'values.push(name.trim().toLowerCase());',
 record_mutation: 'increment(counter);',
 deep_clone: 'const saved = clone(user); user.details.scores[0] = i + 1;',
 scalar_arrays: 'values.push(seed + i + j); values[3] = i; const saved = clone(values);',
 owned_arrays: 'entries.push({name: " Jo ", score: i + seed}); entries[1].score = i + 1;',
 nested_arrays: 'rows[1].push(i + 2); const saved = clone(rows);',
 record_arrays: 'append(catalog, i); catalog.users[0].profile = {score: i + 1};',
 field_restore: 'const scores = user.details.scores; user.details.scores = [i + 2];',
 local_closures: 'const inner = (): int => { state = state + 2; return state; };',
 closure_factory: 'const action = counter(seed); sum = sum + action();',
 callback_contracts: 'sum = sum + forward(action, i); sum = sum + single(onceAction);',
 owned_callbacks: 'const moved = identity(first); const wrapped = retain(moved);',
 callable_replacement: 'action = counter(seed); const moved = action;',
 recursive_data: 'sum = sum + total(tree) + even(8, seed);',
 recursive_callbacks: 'sum = sum + visit(action, 8) + countdown(counter(i), 4);',
 recursive_factory: 'const action = counter(i, 8); const lazy = chain(seed, 4);',
 ordered_removal: 'values.remove(3); values.remove(values.length - 1); values.remove(0);',
 owned_arguments: 'readTwice(read, user); apply(mutate, user); const saved = takeOnce(take, user);',
 capture_replacement: 'saved = {name: "Sam", value: previous + step};',
 int_array_includes: 'values.includes(needle, from);',
 int_array_index_of: 'values.indexOf(needle, from);',
 boolean_array_includes: 'flags.includes(needle, from);',
 boolean_array_index_of: 'flags.indexOf(needle, from);',
 string_array_includes: 'words.includes(needle, from);',
 string_array_index_of: 'words.indexOf(needle, from);',
 dynamic_string_array_includes: 'words.push(input.trim().toLowerCase()); words.includes(needle, from);',
 dynamic_string_array_index_of: 'words.push(input.trim().toLowerCase()); words.indexOf(needle, from);',
 unit_string_array_includes: 'values.push(input.charAt(position)); values.includes(needle, from);',
 unit_string_array_index_of: 'values.push(input.charAt(position)); values.indexOf(needle, from);',
 string_char_at_ascii_long: 'input.charAt(index); // 64 KiB ASCII input',
 string_char_at_unicode_tail: 'input.charAt(index); // early reads, Unicode after 64 KiB'
};
const title = id => id.replaceAll('_', ' ');
const anchor = (kind, id) => `${kind}-${id.replaceAll('_', '-')}`;
export function median(values) {
 const sorted = [...values].sort((a,b) => a-b), middle = Math.floor(sorted.length/2);
 return sorted.length % 2 ? sorted[middle] : (sorted[middle-1]+sorted[middle])/2;
}
export function statistics(values, {signed = false} = {}) {
 if (!values.length || values.some(value => !Number.isFinite(value) || (!signed && value < 0))) throw new Error('Incomplete or invalid results');
 return {min: Math.min(...values), avg: values.reduce((sum, value) => sum + value, 0) / values.length, max: Math.max(...values), samples: values.length};
}
const cell = value => String(value).replaceAll('|', '\\|').replaceAll('\n', ' ');
const rawLink = file => `[Raw JSON](./benchmarks/runtime/results/${encodeURIComponent(path.basename(file))})`;
function throughputRow(label, durations, operations) {
 statistics(durations);
 if (!(operations > 0) || durations.some(ms => ms === 0)) return `| ${cell(label)} | ${durations.length} | — | — | — |`;
 const result = statistics(durations.map(ms => operations * 1000 / ms));
 const rate = value => value.toLocaleString('en-US', {minimumFractionDigits: value < 1000 ? 2 : 0, maximumFractionDigits: value < 1000 ? 2 : 0});
 return `| ${cell(label)} | ${result.samples} | ${rate(result.min)} | ${rate(result.avg)} | ${rate(result.max)} |`;
}
const tableHeader = ['| Runtime / tool | Samples | Min ops/s | Avg ops/s | Max ops/s |', '| --- | ---: | ---: | ---: | ---: |'];
const epoch = data => data.semanticEpoch ?? data.manifest?.semanticEpoch ?? 'historical-utf16-unversioned';
function runtimeSection(data, file, {includeAuxiliary=true}={}) {
 if (data.verifiedOnly) throw new Error('Verification is not a timed baseline');
 const ids = [...new Set(data.execution.map(record => record.engine))];
 const executionOnly=data.measurementScope==='execution-only';
 const startupIDs = [...new Set(data.startup.map(record => record.engine))];
 const workloads = data.manifest.workloads.filter(workload => data.sources.some(source => source.id === workload.id));
 if (data.execution.some(record => !workloads.some(workload => workload.id === record.workload)) || !ids.length || !workloads.length || (!executionOnly && (ids.length !== startupIDs.length || ids.some(id => !startupIDs.includes(id)))) || (executionOnly && (data.startup.length || data.compilation.length))) throw new Error('Incomplete or invalid results');
 const lines = ['<details>', `<summary>Run details — ${cell(data.timestamp)}, gate ${data.manifest.gate}${executionOnly?' · execution only':''}</summary>`, '', `${cell(data.machine.cpu)} / ${cell(data.machine.architecture)} · ${rawLink(file)} · contract ${cell(epoch(data))}`, '',
  `Node ${cell(data.versions.node)} · Bun ${cell(data.versions.bun)} · ${cell(data.versions.deno.split('\n')[0])} · tsc ${cell(data.versions.typescript)} · tsx ${cell(data.versions.tsx)} · ts-node ${cell(data.versions['ts-node'])} · ${cell(data.versions.rust.split(' ').slice(0,2).join(' '))}`, '', '</details>', ''];
 for (const workload of workloads) {
  lines.push(`<a id="${anchor('runtime', workload.id)}"></a>`, '', `### ${cell(title(workload.id))}`, '', '<details>', `<summary>Example and sources (n = ${workload.iterations.toLocaleString('en-US')})</summary>`, '', `\`${cell(workload.specification ?? examples[workload.id])}\``, '', `${epoch(data) === 'rust-native-utf8-v1' ? `[TSRust source](./benchmarks/runtime/workloads/${workload.id}.tsr) · [TypeScript source](./benchmarks/runtime/workloads/${workload.id}.ts)` : 'Historical source identity is recorded by the raw source hashes; current source files use a different string contract.'} · ${data.manifest.processes} processes × ${data.manifest.samples} samples · ${data.manifest.warmups} warmups/process`, '');
  if (workload.notes) {
   const notes = isConcat(workload) ? workload.notes.replace('Higher-rank/boolean/record ABI support is pending actual-emission guard, not assumed from declared source types.', 'Actual emitted ABI guards validate higher-rank, boolean and record representations before native adaptation.') : workload.notes;
   lines.push(cell(notes), '');
  }
  if (data.manifest.gate === 37 && /^string_(starts_with|ends_with|includes|index_of)_(ascii|unicode)$/.test(workload.id)) lines.push('Gate 37 timed coverage: omitted-position defaults and explicit byte positions 0, 3, 8, 11, 16, 22, and 999 (clamped). Alternating selectors reach seven table entries; negative and other positions are separate semantic tests. Unicode coordinate-map preparation occurs once per workload call and is included.', '');
  if(isCopying(workload)){assertCopyingPolicy(workload);const source=data.sources.find(s=>s.id===workload.id),abi=source?.copyingABI;if(abi?.protocol!=='actual-emission-copying-array-result-abi-v1'||abi.sourceSha256!==source.tsrSha256||abi.sourceSha256!==workload.sourceSha256||source.tsSha256!==workload.counterpartSha256||abi.emittedSha256!==source.emittedSha256||abi.sourceResult!==workload.result||abi.maxDepth!==16||data.copyingObserverProtocol!=='copying-array-natural-v1'||!Array.isArray(data.copyingAdapterInputs)||['copying-array-observation.mjs','copying-array-result-abi.mjs','copying-array-observer-policy.json'].some(name=>data.copyingAdapterInputs.filter(([key,sha])=>key===name&&/^[a-f0-9]{64}$/.test(sha)).length!==1))throw Error('Missing actual copying-array result ABI provenance');}
  if(isRange(workload)){assertRangePolicy(workload);const source=data.sources.find(s=>s.id===workload.id),abi=source?.rangeABI;if(abi?.protocol!=='actual-emission-range-mutation-result-abi-v1'||abi.sourceSha256!==source.tsrSha256||abi.sourceSha256!==workload.sourceSha256||source.tsSha256!==workload.counterpartSha256||abi.emittedSha256!==source.emittedSha256||abi.sourceResult!==workload.result||abi.maxDepth!==16||data.rangeObserverProtocol!=='range-array-natural-v1'||!Array.isArray(data.rangeAdapterInputs)||['range-mutation-observation.mjs','range-mutation-result-abi.mjs','range-mutation-observer-policy.json'].some(name=>data.rangeAdapterInputs.filter(([key,sha])=>key===name&&/^[a-f0-9]{64}$/.test(sha)).length!==1))throw Error('Missing actual range-mutation result ABI provenance');}
  if(isHigher(workload)){assertHigherPolicy(workload);const source=data.sources.find(s=>s.id===workload.id),abi=source?.higherABI;if(abi?.protocol!=='actual-emission-higher-order-result-abi-v1'||abi.sourceSha256!==source.tsrSha256||abi.sourceSha256!==workload.sourceSha256||source.tsSha256!==workload.counterpartSha256||abi.emittedSha256!==source.emittedSha256||abi.sourceResult!==workload.result||abi.maxDepth!==16||data.higherObserverProtocol!=='higher-order-natural-values-v1'||!Array.isArray(data.higherAdapterInputs)||['higher-order-observation.mjs','higher-order-result-abi.mjs','higher-order-observer-policy.json'].some(name=>data.higherAdapterInputs.filter(([key,sha])=>key===name&&/^[a-f0-9]{64}$/.test(sha)).length!==1))throw Error('Missing actual higher-order result ABI provenance');}
  if(isDeclared(workload)){assertDeclaredPolicy(workload);const source=data.sources.find(s=>s.id===workload.id),abi=source?.declaredABI;if(abi?.protocol!=='actual-emission-declared-function-abi-v1'||abi.sourceSha256!==source.tsrSha256||abi.sourceSha256!==workload.declaredSourceSha256||source.tsSha256!==workload.declaredTSSha256||abi.emittedSha256!==source.emittedSha256||data.declaredObserverProtocol!=='declared-function-values-v1'||abi.targets?.length!==workload.declaredTargets.length||abi.targets.some((t,i)=>t.source!==workload.declaredTargets[i].name||t.environment!=='read'||t.captureFields!==0))throw Error('Missing actual declared-value ABI provenance');}
  if(isConcat(workload)){assertConcatPolicy(workload);const source=data.sources.find(s=>s.id===workload.id),abi=source?.concatABI;if(abi?.protocol!=='actual-emission-concat-abi-v1'||abi.sourceSha256!==source.tsrSha256||abi.sourceSha256!==workload.concatSourceSha256||source.tsSha256!==workload.concatTSSha256||abi.emittedSha256!==source.emittedSha256||abi.sourceResult!==workload.result||abi.maxDepth!==16||data.concatObserverProtocol!=='declared-concat-v1')throw Error('Missing actual concat ABI provenance');}
  if(isReduction(workload)){const source=data.sources.find(s=>s.id===workload.id),abi=source?.reductionABI;if(abi?.protocol!=='actual-emission-reduction-abi-v1'||abi.sourceSha256!==source.tsrSha256||abi.emittedSha256!==source.emittedSha256)throw Error('Missing actual reduction ABI provenance');}
  const approximate=isApproximate(workload),stored=data.approximateInputs?.workloads?.[workload.id];
  const reference=approximate?restoreApproximateReference(workload,data.sources.find(source=>source.id===workload.id),stored?.artifacts):undefined;
  if(approximate){
   if(data.approximateInputs?.protocol!=='mpfr-directed-f64-scalar-v1'||!stored?.probes?.engines)throw Error('Missing approximate point-probe provenance');
   const checkedCases=data.approximateInputs.correctness?.filter(record=>record.workload===workload.id),caseCoordinates=[[0,0],[1,7],[2,19],[17,3],[workload.iterations,data.manifest.seed]];
   if(!Array.isArray(checkedCases)||checkedCases.length!==caseCoordinates.length*ids.length)throw Error('Incomplete approximate correctness cases');
   for(const [iterations,seed]of caseCoordinates)for(const id of ids){const actual=checkedCases.filter(record=>record.engine===id&&record.iterations===iterations&&record.seed===seed);if(actual.length!==1)throw Error('Missing/duplicate approximate correctness case');const probe=stored.probes.engines[id];if(!probe)throw Error('Missing approximate correctness engine probe');validateApproximatePointProbe(probe.actual,reference);validateApproximateMeasurement(actual[0],reference,{iterations,seed,warmups:0,samples:1,pointProbeAccepted:true});}

   lines.push('Approximate criterion: '+reference.approvedPolicy.policy.maxOperationRankSteps+' adjacent binary64 ranks per independently probed operation; final scalar accepted by ordered-add interval. Bounded finite domain; not a universal Rust precision promise. Untimed probes validate their own call sites, not every timed intermediate output. Raw per-process bits and errors remain in the linked record.','');
  }
  lines.push('</details>', '', ...tableHeader);
  for (const id of ids) {
   const records = data.execution.filter(record => record.workload === workload.id && record.engine === id);
   if(approximate){
    if(records.length!==data.manifest.processes||new Set(records.map(record=>record.processIndex)).size!==data.manifest.processes||records.some(record=>!Number.isInteger(record.processIndex)||record.processIndex<0||record.processIndex>=data.manifest.processes))throw Error('Incomplete approximate process records');
    const probe=stored.probes.engines[id];if(!probe||!Array.isArray(probe.command))throw Error('Missing approximate engine probe');
    validateApproximatePointProbe(probe.actual,reference);
    for(const record of records)validateApproximateMeasurement(record,reference,{iterations:workload.iterations,seed:data.manifest.seed,warmups:data.manifest.warmups,samples:data.manifest.samples,pointProbeAccepted:true});
   }else{
   if (records.length !== data.manifest.processes || new Set(records.map(record => record.processIndex)).size !== data.manifest.processes || records.some(record => !Number.isInteger(record.processIndex) || record.processIndex < 0 || record.processIndex >= data.manifest.processes || record.checksum !== data.correctness.checksums[workload.id] || record.durations.length !== data.manifest.samples)) throw new Error('Incomplete or invalid results');
   if(requiresExactResult(workload)) {
    if(typeof data.correctness.exactBits?.[workload.id]!=='string' || isNumberResult(workload)&&data.numberResultEncoding!=='ieee-f64-bits-v1') throw new Error('Missing exact Number correctness reference');
    if(isReductionSchema(workload)&&data.reductionResultEncoding!=='reduction-schema-v1')throw Error('Missing exact reduction result encoding');
    if(isSliceResult(workload)&&!data.arrayResultEncodings?.includes(workload.resultEncoding)) throw new Error('Missing exact array result encoding');
    for(const record of records) {const expected={checksum:data.correctness.checksums[workload.id],exactBits:data.correctness.exactBits[workload.id]};if(isCopying(workload))validateCopyingObservation(record,expected,workload);else if(isRange(workload))validateRangeObservation(record,expected,workload);else if(isHigher(workload))validateHigherObservation(record,expected,workload);else if(isDeclared(workload))validateDeclaredObservation(record,expected,workload);else if(isConcat(workload))validateConcatObservation(record,expected,workload);else validateExactObservation(record,expected);}
   }
   }
   lines.push(throughputRow(modes[id] ?? id, records.flatMap(record => record.durations), workload.iterations * (workload.operationsPerIteration ?? 1)));
  }
  if(approximate){
   lines.push('Observed final-scalar error (distance from rounded ideal accumulator; separate from throughput):','', '| Runtime | Outputs including warmups | Maximum binary64 rank distance |','| --- | ---: | ---: |');
   for(const id of ids){const probe=stored.probes.engines[id];validateApproximatePointProbe(probe.actual,reference);const observed=data.execution.filter(record=>record.workload===workload.id&&record.engine===id).flatMap(record=>{const checked=validateApproximateMeasurement(record,reference,{iterations:workload.iterations,seed:data.manifest.seed,warmups:data.manifest.warmups,samples:data.manifest.samples,pointProbeAccepted:true});return [...checked.warmups,...checked.samples];});const maximum=observed.reduce((max,value)=>BigInt(value.binary64RankDistance)>max?BigInt(value.binary64RankDistance):max,0n);lines.push('| '+cell(modes[id]??id)+' | '+observed.length+' | '+maximum+' |');}
  }
  lines.push('');
 }
 if(executionOnly||!includeAuxiliary) return lines.join('\n');
 lines.push('<a id="startup"></a>', '', '### Startup', '', '<details>', '<summary>Example and measurement</summary>', '', '`console.log(0);` — one operation is a cached process launch through output and exit.', '', '</details>', '', ...tableHeader);
 for (const id of ids) {
  const records = data.startup.filter(record => record.engine === id);
  if (records.length !== data.manifest.startupSamples || new Set(records.map(record => record.sample)).size !== data.manifest.startupSamples) throw new Error('Incomplete or invalid results');
  lines.push(throughputRow(modes[id] ?? id, records.map(record => record.ms), 1));
 }
 lines.push('', '<a id="compilation"></a>', '', '### Compilation and checking', '', '<details>', '<summary>Compilation phases</summary>', '', 'TSRust: source to emitted Rust to optimized executable. TypeScript: source to tsc to JavaScript. One operation is one completed phase.', '', '</details>', '', '| Workload | Tool phase | Samples | Min ops/s | Avg ops/s | Max ops/s |', '| --- | --- | ---: | ---: | ---: | ---: |');
 const phases = {'frontend-check': 'TSRust frontend check', 'frontend-emit': 'TSRust frontend emit', 'rustc-optimized': 'Rust optimized compilation', 'tsc-check-emit': 'TypeScript tsc check + emit'};
 for (const workload of workloads) for (const [phase, label] of Object.entries(phases)) {
  const records = data.compilation.filter(record => record.workload === workload.id && record.phase === phase);
  if (records.length !== 1 || records[0].durations.length !== data.manifest.compileSamples) throw new Error('Incomplete or invalid results');
  lines.push(throughputRow(label, records[0].durations, 1).replace('| ', `| ${cell(workload.id)} | `));
 }
 lines.push('');
 return lines.join('\n');
}
function allocationSection(data, file) {
 if (!data.rows.length) throw new Error('Incomplete or invalid results');
 const lines = ['<details>', `<summary>Allocation run — ${cell(data.timestamp)}</summary>`, '', `${cell(data.machine.cpu)} / ${cell(data.machine.architecture)} · ${rawLink(file)} · contract ${cell(epoch(data))}`, '',
  'Three samples per case. Native allocator storage and Node retained heap are different metrics. Small Node deltas can be measurement overhead; see empty control.', '', '</details>', ''];
 const metrics = {
  allocations: ['New allocations', 'calls'], reallocations: ['Reallocations', 'calls'],
  retainedRequestedBytes: ['Retained requested storage', 'bytes']
 };
 const ids = [...new Set(data.rows.map(record => record.probe))];
 for (const id of ids) {
  const probe = data.probes.find(item => item.id === id);
  if (!probe) throw new Error('Incomplete or invalid results');
  lines.push(`<a id="${anchor('allocation', id)}"></a>`, '', `### ${cell(title(id))}`, '', '<details>', '<summary>Example and sources</summary>', '', cell(probe.specification), '', `[TSRust source](./benchmarks/runtime/allocations/${id}.tsr) · [TypeScript source](./benchmarks/runtime/allocations/${id}.ts)`, '', '</details>', '', '| Runtime | n | Metric | Unit | Min | Avg | Max |', '| --- | ---: | --- | --- | ---: | ---: | ---: |');
  for (const record of data.rows.filter(item => item.probe === id)) {
   if (record.samples.length !== 3 || record.samples.some(sample => !Number.isSafeInteger(sample.native.checksum) || sample.native.checksum !== sample.node.checksum || sample.native.liveBytesAfterDrop !== 0)) throw new Error('Incomplete or invalid results');
   if(isNumberResult(probe)) for(const sample of record.samples) { validateNumberObservation(sample.native,sample.node); validateNumberObservation(sample.node,sample.native); }
   const memoryMax = Math.max(...record.samples.flatMap(sample => [Math.abs(sample.native.retainedRequestedBytes), Math.abs(sample.node.retainedHeapDelta)]));
   const divisor = memoryMax >= 2**20 ? 2**20 : memoryMax >= 2**10 ? 2**10 : 1;
   const memoryUnit = divisor === 2**20 ? 'MiB' : divisor === 2**10 ? 'KiB' : 'B';
   const number = value => value === 0 ? '0' : Number.isInteger(value) ? value.toLocaleString('en-US') : value.toFixed(Math.abs(value) < 0.1 ? 6 : 3);
   for (const [key, [label, unit]] of Object.entries(metrics)) {
    const result = statistics(record.samples.map(sample => sample.native[key] / (unit === 'bytes' ? divisor : 1)));
    lines.push(`| TSRust native | ${record.n} | ${label} | ${unit === 'bytes' ? memoryUnit : unit} | ${number(result.min)} | ${number(result.avg)} | ${number(result.max)} |`);
   }
   const result = statistics(record.samples.map(sample => sample.node.retainedHeapDelta / divisor), {signed: true});
   lines.push(`| Node after tsc | ${record.n} | Retained V8 heap delta | ${memoryUnit} | ${number(result.min)} | ${number(result.avg)} | ${number(result.max)} |`);
  }
  lines.push('');
 }
 return lines.join('\n');
}
function section(data, file) {
 return Array.isArray(data.rows) ? allocationSection(data, file) : runtimeSection(data, file);
}
// Add new operations here; deliberately keep unclassified measured IDs in Other.
export const reportCategories = [
 {id:'json',label:'JSON',workloads:['native_json_roundtrip','native_json_namespace','native_json_scalars','native_json_typeof','native_json_inspection','native_json_views','native_json_refinement','native_json_string_reads','native_json_string_views','native_json_import'],probes:[]},
 {id:'time',label:'Monotonic time',workloads:['std_time_monotonic_now'],probes:[]},
 {id:'os',label:'Operating system',workloads:['std_os_platform','std_os_arch','std_os_endianness'],probes:[]},
 {id:'environment',label:'Environment',workloads:['std_env_get'],probes:[]},
 {id:'resources',label:'File resources',workloads:['native_file_read_close','native_file_scope_drop','native_file_write','native_file_write_all','native_file_full_write_read','native_file_buffered_write_read'],probes:[]},
 {id:'collections',label:'Owned collections',workloads:['native_deque_clone','native_heap','native_ordered_set','native_ordered_map'],probes:[]},
 {id:'bytes',label:'Owned bytes',workloads:['native_byte_views_deque','native_byte_views_deque_callbacks','native_bytes','native_byte_access','native_byte_growth','native_byte_copy_append','native_byte_clone','native_byte_utf8','native_byte_views','native_byte_views_maps','native_byte_views_recursive','native_byte_views_linked', 'native_byte_views_callbacks', 'native_byte_views_stored_callbacks','native_byte_views_captured_only','native_byte_views_nullable_callbacks','native_byte_views_parameter_stores'],probes:[]},
 {id:'paths',label:'Lexical paths',workloads:['std_path_join_ascii','std_path_join_empty_roots','std_path_join_many_unicode','std_path_relative','std_path_dirname_ascii','std_path_extname_ascii','std_path_is_absolute_ascii','std_path_normalize_ascii'],probes:[]},
 {id:'numbers',label:'Numbers',workloads:['numeric_string_conversion','scalar_unary_plus','comma_contexts','comma_expressions','numeric_literals','optional_chain_int_field','optional_chain_number_field','nullish_assign_int_value','nullish_assign_number_value','nullish_assign_nullable_int_value','nullish_int_value','nullish_number_value','nullish_nullable_fallback_int','narrowing_int','narrowing_number','nullable_int_null','nullable_number_null','number_parse_float_decimal','number_parse_float_exponent','number_parse_float_whitespace','number_parse_float_trailing','number_parse_float_nonfinite_invalid','number_parse_float_mixed','number_to_fixed_default','number_to_fixed_2','number_to_exponential_default','number_to_exponential_2','number_to_precision_default','number_to_precision_5','int_to_string_decimal','int_to_string_binary','int_to_string_hex','int_to_string_base36','number_value_of','int_value_of','conditional_int_value','conditional_number_value','compound_value_int_add','compound_value_int_subtract','compound_value_int_multiply','compound_value_int_divide','compound_value_int_remainder','compound_value_int_power','compound_value_int_and','compound_value_int_or','compound_value_int_xor','compound_value_int_shift_left','compound_value_int_shift_right','compound_value_int_shift_right_unsigned','compound_value_number_add','compound_value_number_subtract','compound_value_number_multiply','compound_value_number_divide','compound_value_number_remainder','compound_value_number_power','compound_value_number_and','compound_value_number_or','compound_value_number_xor','compound_value_number_shift_left','compound_value_number_shift_right','compound_value_number_shift_right_unsigned','int_assignment_value','number_assignment_value','int_native_unary_plus','int_native_bit_not','int_native_bit_and','int_native_bit_or','int_native_bit_xor','int_native_shift_left','int_native_shift_right','int_native_shift_right_unsigned','int_native_power','int_native_compound_bit_and','int_native_compound_bit_or','int_native_compound_bit_xor','int_native_compound_shift_left','int_native_compound_shift_right','int_native_compound_shift_right_unsigned','int_native_compound_power','update_int_prefix_increment','update_int_prefix_decrement','update_int_postfix_increment','update_int_postfix_decrement','update_number_prefix_increment','update_number_prefix_decrement','update_number_postfix_increment','update_number_postfix_decrement','compound_int_add','compound_int_subtract','compound_int_multiply','compound_int_divide','compound_int_remainder','compound_number_add','compound_number_subtract','compound_number_multiply','compound_number_divide','compound_number_remainder','compound_number_power','compound_number_and','compound_number_or','compound_number_xor','compound_number_shift_left','compound_number_shift_right','compound_number_shift_right_unsigned','scalar_control','scalar_arrays','retained_numbers','fixed_int_array_reverse','growable_int_array_reverse','int_array_includes','int_array_index_of','int_divide','int_negate','int_less_equal','int_greater_equal','int_not_equal','number_add','number_subtract','number_multiply','number_divide','number_remainder','number_negate','number_equal','number_not_equal','number_less','number_less_equal','number_greater','number_greater_equal','number_from_int','number_from_boolean','number_parse_decimal','number_array_operations','number_is_finite','number_is_nan','number_is_integer','number_is_safe_integer','math_abs','math_floor','math_ceil','math_trunc','math_sqrt','number_direct_literal_index','number_fixed_local_operations'],probes:['int_value','int_array','number_value']},
 {id:'booleans',label:'Booleans',workloads:['optional_chain_boolean_field','nullish_assign_boolean_value','nullish_boolean_value','narrowing_boolean','nullable_boolean_undefined','boolean_string_truthiness','boolean_to_string','boolean_value_of','conditional_boolean_value','boolean_assignment_value','boolean_logical_and_assign','boolean_logical_or_assign','boolean_array_includes','boolean_array_index_of','boolean_equal','boolean_not_equal','boolean_or','boolean_short_circuit'],probes:['boolean_value','boolean_array']},
 {id:'strings',label:'Strings',workloads:['truthiness','owned_assignment_values','string_concatenation','string_iteration','logical_nullable_string_values','std_path_basename_ascii','optional_chain_string_length','nullish_assign_string_value','nullish_assign_nullable_string_value','nullish_string_value','nullish_null_undefined_value','narrowing_string','nullable_string_undefined','string_explicit_int','string_array_conversion','string_html_anchor','string_html_big','string_html_blink','string_html_bold','string_html_fixed','string_html_fontcolor','string_html_fontsize','string_html_italics','string_html_link','string_html_small','string_html_strike','string_html_sub','string_html_sup','string_substr_default_ascii','string_substr_default_unicode','string_substr_bounded_ascii','string_substr_bounded_unicode','string_cow_split_pieces','string_scalar_split_unicode','template_primitive_ascii','template_string_unicode','quoted_string_literals_ascii','quoted_string_literals_unicode','string_is_well_formed_ascii','string_is_well_formed_unicode','string_to_well_formed_ascii','string_to_well_formed_unicode','string_to_string_ascii','string_to_string_unicode','string_value_of_ascii','string_value_of_unicode','string_substring_default_ascii','string_substring_default_unicode','string_substring_bounded_ascii','string_substring_bounded_unicode','string_substring_swapped_ascii','string_substring_swapped_unicode','string_concat_zero_ascii','string_concat_zero_unicode','string_concat_variadic_ascii','string_concat_variadic_unicode','string_pad_start_default','string_pad_end_default','string_pad_start_ascii','string_pad_end_ascii','string_pad_start_unicode','string_pad_end_unicode','conditional_string_ascii','conditional_string_unicode','compound_string_add_ascii','compound_string_add_unicode','ascii_strings','literal_normalization','dynamic_normalization','dynamic_string_clone','string_array_includes','string_array_index_of','dynamic_string_array_includes','dynamic_string_array_index_of','string_length','string_slice_ascii','string_slice_unicode','string_char_at','unit_string_array_includes','unit_string_array_index_of','string_char_at_ascii_long','string_char_at_unicode_tail','string_char_at_unicode_late','string_char_at_unicode_middle','string_construct_slice_unicode_long','string_construct_slice_ascii_long','string_char_at_unicode_middle_literal','string_equality','string_ordering_ascii','literal_ascii_uppercase','dynamic_unicode_uppercase','owned_string_array_reverse','string_starts_with_ascii','string_ends_with_ascii','string_includes_ascii','string_index_of_ascii','string_starts_with_unicode','string_ends_with_unicode','string_includes_unicode','string_index_of_unicode','string_repeat_plain','string_repeat_unicode','string_repeat_empty_zero','string_replace_first_plain','string_replace_first_tokens','string_replace_first_empty','string_replace_all_plain','string_replace_all_tokens','string_replace_all_empty','string_split_normal','string_split_limits','string_split_empty_delimiter'],probes:['string_value','string_array']},
 {id:'records',label:'Records and ownership',workloads:['generic_record_nested_int','generic_function_record_pipeline','optional_chain_record_field','optional_chain_nullable_chain','nullish_assign_record_value','nullish_record_value','narrowing_record','nullable_record_undefined','conditional_record_value','record_mutation','deep_clone','field_restore','owned_arguments','capture_replacement','retained_rows'],probes:['record_value']},
 {id:'arrays',label:'Array operations',workloads:['map_group_by','iterable_call_spread','iterable_array_spread','array_from_collections','optional_chain_array_index','optional_chain_array_join','nullish_assign_array_value','nullish_array_value','array_of_int','array_of_number','array_of_boolean','array_of_string','array_of_nested','array_of_record','array_of_null','array_of_undefined','array_flat_int','array_flat_number','array_flat_boolean','array_flat_string','array_flat_nested','array_flat_record','array_flat_null','array_flat_undefined','array_flat_map_int','array_flat_map_number','array_flat_map_boolean','array_flat_map_string','array_flat_map_nested','array_flat_map_record','array_flat_map_null','array_flat_map_undefined','array_sort_default_int','array_sort_default_number','array_sort_default_boolean','array_sort_default_string','array_sort_default_null','array_sort_default_undefined','array_sort_compare_int','array_sort_compare_number','array_sort_compare_boolean','array_sort_compare_string','array_sort_compare_nested','array_sort_compare_record','array_sort_compare_null','array_sort_compare_undefined','array_to_sorted_default_int','array_to_sorted_default_number','array_to_sorted_default_boolean','array_to_sorted_default_string','array_to_sorted_default_null','array_to_sorted_default_undefined','array_to_sorted_compare_int','array_to_sorted_compare_number','array_to_sorted_compare_boolean','array_to_sorted_compare_string','array_to_sorted_compare_nested','array_to_sorted_compare_record','array_to_sorted_compare_null','array_to_sorted_compare_undefined','array_splice_int','array_splice_number','array_splice_boolean','array_splice_string','array_splice_nested','array_splice_record','array_splice_null','array_splice_undefined','array_unshift_int','array_unshift_number','array_unshift_boolean','array_unshift_string','array_unshift_nested','array_unshift_record','array_unshift_null','array_unshift_undefined','array_find_int','array_find_number','array_find_boolean','array_find_string','array_find_nested','array_find_record','array_find_null','array_find_undefined','array_find_last_int','array_find_last_number','array_find_last_boolean','array_find_last_string','array_find_last_nested','array_find_last_record','array_find_last_null','array_find_last_undefined','array_at_int','array_at_number','array_at_boolean','array_at_string','array_at_nested','array_at_record','array_at_null','array_at_undefined','array_pop_int','array_pop_number','array_pop_boolean','array_pop_string','array_pop_nested','array_pop_record','array_pop_null','array_pop_undefined','array_shift_int','array_shift_number','array_shift_boolean','array_shift_string','array_shift_nested','array_shift_record','array_shift_null','array_shift_undefined','narrowing_array','nullable_array_null','array_to_string_int','array_to_string_boolean','array_to_string_string','array_to_string_nested','array_to_string_number','array_to_string_nullish','array_join_int','array_join_boolean','array_join_string','array_join_nested','array_join_number','array_join_nullish','conditional_array_int','conditional_array_number','conditional_array_boolean','conditional_array_string','owned_arrays','nested_arrays','record_arrays','ordered_removal','array_churn','record_array_churn','array_slice_int','array_slice_number','array_slice_string','array_slice_nested','array_temporary_index_int','array_temporary_index_number','array_temporary_index_string','array_temporary_index_nested','array_map_int','array_filter_int','array_map_number','array_filter_number','array_map_string','array_filter_string','array_map_nested','array_filter_nested','array_some_int','array_some_number','array_some_boolean','array_some_string','array_some_record','array_some_nested','array_every_int','array_every_number','array_every_boolean','array_every_string','array_every_record','array_every_nested','array_find_index_int','array_find_index_number','array_find_index_boolean','array_find_index_string','array_find_index_record','array_find_index_nested','array_find_last_index_int','array_find_last_index_number','array_find_last_index_boolean','array_find_last_index_string','array_find_last_index_record','array_find_last_index_nested'],probes:[]},
 {id:'callbacks',label:'Closures and callbacks',workloads:['result_match','caller_borrowed_record_stores','fallible_record_callbacks','named_function_expressions','generic_callable_replacement','scoped_record_composition','scoped_callable_records','generic_value_adapters','generic_function_results','generic_function_captures','generic_function_deferred','generic_function_contexts','nested_function_rest','nested_function_defaults','nested_function_results','nested_function_owned_escape','once_factory_results','default_conditional_callable_fields','owned_callable_fields','annotated_once_locals','callable_array_mutable_pipeline','callable_array_transfer_pipeline','callable_array_once_transfer','callable_array_reduction_pipeline','nested_function_captures','local_closures','closure_factory','callback_contracts','owned_callbacks','callable_replacement'],probes:[]},
 {id:'recursion',label:'Recursive operations',workloads:['recursive_result_inference','nested_function_declarations','recursive_data','recursive_callbacks','recursive_factory','recursive_growth','recursive_clone'],probes:[]},
 {id:'other',label:'Other operations and controls',workloads:['interfaces','optional_fields','type_aliases','classic_for_control','do_while_control','else_if_control','object_is_values','object_keys_order','data_union_values','union_array_values','object_values_entries','destructuring_patterns','borrowed_optional_fallback','destructuring_defaults','object_has_own','destructuring_rest','nested_rest_patterns','for_pattern_initializers','declaration_lists','destructuring_assignment','braceless_bodies','switch_statements','function_expressions','global_number_names','deferred_initialization','in_operator','labeled_control','callable_expressions','exception_completion','type_assertions','enum_values','set_values','map_values','map_snapshots','set_entries','set_for_each','map_iteration','set_algebra','set_relations','map_for_each','array_callback_errors','testing_scalar_assertions','testing_expectation_predicates'],probes:['empty_control']}
];
export function reportCategory(id, {allocation=false}={}) {
  if(!allocation&&(id==='array_keys_int'||id==='array_values_int'))return 'arrays';
  if(!allocation&&id.startsWith('tuple_types_'))return 'arrays';
  if(!allocation&&id.startsWith('array_from_'))return 'arrays';
  if(!allocation&&id.startsWith('array_literal_spread_'))return 'arrays';
  if(!allocation&&id.startsWith('call_spread_'))return 'callbacks';
  if(!allocation&&id.startsWith('rest_parameter_'))return 'callbacks';
 const explicit = reportCategories.find(category=>category[allocation?'probes':'workloads'].includes(id))?.id;
 if (explicit) return explicit;
 if(!allocation&&/^(?:optional|default)_parameter_/.test(id))return 'callbacks';
 if(!allocation&&id.startsWith('nullable_callable_'))return 'callbacks';
 if(!allocation&&id.startsWith('higher_'))return 'callbacks';
 if(!allocation&&id.startsWith('declared_'))return 'callbacks';
 if (!allocation && /^(?:array_entries_|array_locale_string_|array_reduce(?:_right)?_|array_for_each_|array_concat_|array_fill_|array_copy_within_|array_to_reversed_|array_with_|array_to_spliced_|array_last_index_of_)/.test(id)) return 'arrays';
 if (!allocation && /^(number_|math_)/.test(id)) return 'numbers';
 if (!allocation && id.startsWith('string_')) return 'strings';
 return 'other';
}
export function buildReport(records, {semanticEpoch} = {}) {
 const sorted = [...records].sort((a,b) => a.data.timestamp.localeCompare(b.data.timestamp) || a.file.localeCompare(b.file));
 for (const record of sorted) if (!record.data.verifiedOnly) section(record.data, record.file);
 const measured = sorted.filter(record => !record.data.verifiedOnly);
 const currentEpoch = semanticEpoch ?? (measured.length ? epoch(measured.at(-1).data) : 'unmeasured');
 const runtimeRecords = sorted.filter(record => !record.data.verifiedOnly && !Array.isArray(record.data.rows) && epoch(record.data) === currentEpoch);
 const allocationRecords = sorted.filter(record => !record.data.verifiedOnly && Array.isArray(record.data.rows) && epoch(record.data) === currentEpoch);
 // A partial new run replaces only the operations it measured. Never merge samples.
 const latestWorkloads = new Map(), latestProbes = new Map(), latestCompilation = new Map();
 for (const record of runtimeRecords) {
  runtimeSection(record.data, record.file);
  for (const source of record.data.sources) { latestWorkloads.set(source.id, record); if(record.data.measurementScope!=='execution-only') latestCompilation.set(source.id,record); }
 }
 for (const record of allocationRecords) {
  allocationSection(record.data, record.file);
  for (const probe of record.data.rows) latestProbes.set(probe.probe, record);
 }
 const lines = ['# Benchmarks', '',
  `Current semantic epoch: **${cell(currentEpoch)}**. Results from earlier contracts remain in the history and are excluded from these current tables.`, '',
  'Latest results. Execution: ops/s using each workload’s stated operation count. Avg: arithmetic mean of sample rates. Memory: B/KiB/MiB; allocation counts remain counts.', '',
  '[Methodology and limits](./docs/runtime-comparisons.md) · [Sources and reproduction](./benchmarks/runtime/README.md) · [Foundation audit](./docs/foundation-audit.md) · [Run history](#run-records)', '',
  '## Contents', ''];
 const groups = reportCategories.map(category=>({
  ...category,
  workloadIDs:[...latestWorkloads.keys()].filter(id=>reportCategory(id)===category.id),
  probeIDs:[...latestProbes.keys()].filter(id=>reportCategory(id,{allocation:true})===category.id)
 })).filter(category=>category.workloadIDs.length||category.probeIDs.length);
 for(const group of groups) {
  lines.push(`- [${group.label}](#group-${group.id})`);
  if(group.workloadIDs.length) lines.push(`  - Runtime: ${group.workloadIDs.map(id=>`[${cell(title(id))}](#${anchor('runtime',id)})`).join(' · ')}`);
  if(group.probeIDs.length) lines.push(`  - Allocations: ${group.probeIDs.map(id=>`[${cell(title(id))}](#${anchor('allocation',id)})`).join(' · ')}`);
 }
 const latestRuntime = runtimeRecords.filter(record=>record.data.measurementScope!=='execution-only').at(-1);
 if(latestRuntime) lines.push('- [Startup](#startup) · [Compilation and checking](#compilation)');
 lines.push('- [Run history](#run-records)','');
 if(!runtimeRecords.length) lines.push('No timed runtime results have been recorded for this epoch.','');
 if(!allocationRecords.length) lines.push('No allocation results have been recorded for this epoch.','');
 for(const group of groups) {
  lines.push(`<a id="group-${group.id}"></a>`,'',`## ${group.label}`,'');
  for(const record of runtimeRecords) {
   const ids=group.workloadIDs.filter(id=>latestWorkloads.get(id)===record);
   if(!ids.length) continue;
   const data={...record.data,sources:record.data.sources.filter(source=>ids.includes(source.id)),execution:record.data.execution.filter(row=>ids.includes(row.workload)),compilation:record.data.compilation.filter(row=>ids.includes(row.workload))};
   lines.push(runtimeSection(data,record.file,{includeAuxiliary:false}));
  }
  if(group.probeIDs.length) lines.push('### Primitive allocations','');
  for(const record of allocationRecords) {
   const rows=record.data.rows.filter(row=>group.probeIDs.includes(row.probe)&&latestProbes.get(row.probe)===record);
   if(rows.length) lines.push(allocationSection({...record.data,rows},record.file));
  }
 }
 if (latestRuntime) {
  lines.push('## Startup and compilation','',`Startup baseline: ${cell(latestRuntime.data.timestamp)} · ${rawLink(latestRuntime.file)}`, '');
  const text = runtimeSection(latestRuntime.data, latestRuntime.file);
  lines.push(text.slice(text.indexOf('<a id="startup"></a>'), text.indexOf('<a id="compilation"></a>')));
  lines.push('<a id="compilation"></a>', '', '### Compilation and checking', '', '<details>', '<summary>Compilation phases</summary>', '', 'One operation is one completed frontend/check/compile phase. Each workload uses its latest separately measured compilation run; execution-only runs retain dated auxiliary baselines. Individual samples are never merged across runs.', '', '</details>', '', '| Workload | Tool phase | Samples | Min ops/s | Avg ops/s | Max ops/s |', '| --- | --- | ---: | ---: | ---: | ---: |');
  const phases = {'frontend-check': 'TSRust frontend check', 'frontend-emit': 'TSRust frontend emit', 'rustc-optimized': 'Rust optimized compilation', 'tsc-check-emit': 'TypeScript tsc check + emit'};
  for (const [id, record] of latestCompilation) for (const [phase, label] of Object.entries(phases)) {
   const durations = record.data.compilation.find(row => row.workload === id && row.phase === phase).durations;
   lines.push(throughputRow(label, durations, 1).replace('| ', `| ${cell(id)} | `));
  }
  lines.push('', '<details>', '<summary>Compilation baseline dates and sources</summary>', '');
  for(const record of [...new Set(latestCompilation.values())]) lines.push(`- ${cell(record.data.timestamp)} · ${rawLink(record.file)} · ${[...latestCompilation].filter(([,source])=>source===record).map(([id])=>cell(id)).join(', ')}`);
  lines.push('', '</details>', '');
  lines.push('');
 }
 lines.push('<a id="run-records"></a>', '', '## Run records', '', 'All true records are retained unchanged. Regenerate this file with `npm run report` in `benchmarks/runtime`; SQLite is canonical; Markdown generation is optional and benchmark runs skip it by default.', '', '| Timestamp (UTC) | Contract | Kind | Raw record |', '| --- | --- | --- | --- |');
 for (const {data,file} of sorted) lines.push(`| ${cell(data.timestamp)} | ${cell(epoch(data))} | ${data.verifiedOnly ? 'Correctness only' : Array.isArray(data.rows) ? 'Allocations' : data.measurementScope==='execution-only' ? 'Execution only' : 'Runtime / startup / compilation'} | ${rawLink(file)} |`);
 return lines.join('\n') + '\n';
}
export function updateReport({recordsDirectory = recordsFolder, output = path.join(root, 'BENCHMARKS.md'), archive, archivePrefix = 'runtime/results'} = {}) {
 const canonicalArchive = path.join(here, '../records.sqlite');
 if(archive === undefined && path.resolve(recordsDirectory) === recordsFolder && fs.existsSync(canonicalArchive))archive = canonicalArchive;
 const records = loadReportRecords({recordsDirectory, archive, archivePrefix});
 // Validate and assemble every section before replacing the existing report.
 const semanticEpoch = JSON.parse(fs.readFileSync(path.join(here, 'manifest.json'), 'utf8')).semanticEpoch;
 const text = buildReport(records, {semanticEpoch});
 const temporary = `${output}.tmp`;
 fs.writeFileSync(temporary, text);
 fs.renameSync(temporary, output);
 return text;
}
export function render(file, {append = true} = {}) {
 const data = JSON.parse(fs.readFileSync(file, 'utf8'));
 const text = section(data, file);
 if (!append) return text;
 if (path.dirname(path.resolve(file)) !== recordsFolder) throw new Error('Save the true run record in benchmarks/runtime/results before updating the report');
 updateReport();
 console.log(`Updated ${path.join(root, 'BENCHMARKS.md')}`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
 if (process.argv[2]) render(path.resolve(process.argv[2]));
 else { updateReport(); console.log(`Updated ${path.join(root, 'BENCHMARKS.md')}`); }
}
