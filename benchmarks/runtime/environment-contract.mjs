import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {observeSliceTree} from './slice-observation.mjs';
export const environmentID='std_env_get';
export function environmentFor(row,base=process.env) {
 if(row?.id!==environmentID)return base;
 const env={...base,TSRUST_BENCH_ENV_TEXT:'héllo 🦀',TSRUST_BENCH_ENV_EMPTY:''};
 delete env.TSRUST_BENCH_ENV_MISSING;return env;
}
export function expectedEnvironmentObservation(n,seed) {
 assert(Number.isSafeInteger(n)&&n>=0&&n<=20000);assert(Number.isSafeInteger(seed)&&seed>=0&&seed<=1000000);
 // Fixed protocol values, independent of process.env and both implementations.
 const cycle=['héllo 🦀','','<missing>'];
 return observeSliceTree(Array.from({length:n},(_,i)=>cycle[(i+seed)%3]));
}
export function assertEnvironmentSource(source,row,kind) {
 assert.equal(row.id,environmentID);assert.equal(row.family,'environment');assert.equal(row.iterations,20000);
 assert.equal(row.operationsPerIteration,1);assert.equal(row.result,'string[]');assert.equal(row.resultEncoding,'array-shape-scalar-v1');
 assert.deepEqual(row.parameters,['int','int']);assert(['native','typescript'].includes(kind));
 assert.equal(createHash('sha256').update(source).digest('hex'),row[kind==='native'?'sourceSha256':'counterpartSha256']);
 for(const snippet of ['const names = ["TSRUST_BENCH_ENV_TEXT", "TSRUST_BENCH_ENV_EMPTY", "TSRUST_BENCH_ENV_MISSING"];','while (i < n)','names[(i + seed) % 3]','values.push(value === undefined ? "<missing>" : value);','i = i + 1;','return values;'])assert(containsCodeSnippet(source,snippet),snippet);
 for(const snippet of kind==='native'?['import { get } from "std:env";','match (get(names[(i + seed) % 3]))','throw error.message;']:['require("node:process").env','const value = environment[names[(i + seed) % 3]];'])assert(containsCodeSnippet(source,snippet),snippet);
}
