import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {observeSliceTree} from './slice-observation.mjs';
export const monotonicTimeID='std_time_monotonic_now';
export function expectedMonotonicTimeObservation(n,seed) {
 assert(Number.isSafeInteger(n)&&n>=0&&n<=20000);assert(Number.isSafeInteger(seed)&&seed>=0&&seed<=1000000);
 // Clock readings differ across engines. The independent contract is that each
 // observed reading is finite, nonnegative and no earlier than its predecessor.
 return observeSliceTree(Array(n).fill(true));
}
export function assertMonotonicTimeSource(source,row,kind) {
 assert.equal(row.id,monotonicTimeID);assert.equal(row.family,'monotonic-time');assert.equal(row.iterations,20000);
 assert.equal(row.operationsPerIteration,1);assert.equal(row.result,'boolean[]');assert.equal(row.resultEncoding,'array-shape-scalar-v1');
 assert.deepEqual(row.parameters,['int','int']);assert(['native','typescript'].includes(kind));
 assert.equal(createHash('sha256').update(source).digest('hex'),row[kind==='native'?'sourceSha256':'counterpartSha256']);
 for(const snippet of ['while (i < n)','values.push(Number.isFinite(current) && current >= previous && current >= 0);','previous = current;','i = i + 1;','return values;'])assert(containsCodeSnippet(source,snippet),snippet);
 for(const snippet of kind==='native'?['import { monotonicNow } from "std:time";','let previous = monotonicNow();','const current = monotonicNow();']:['require("node:perf_hooks").performance','let previous = clock.now();','const current = clock.now();'])assert(containsCodeSnippet(source,snippet),snippet);
}
