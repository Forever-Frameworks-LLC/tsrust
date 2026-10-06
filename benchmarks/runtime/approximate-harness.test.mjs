// Prepared SOURCE-ONLY tests. No generated positive numerical fixture is invented.
// Native ABI and full generated-reference tests remain mandatory after grant.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {adaptRustDriver,adaptTSDriver,rustChecksum} from './adapters.mjs';
import {isNumberResult} from './number-observation.mjs';
import {requiresExactResult} from './result-observation.mjs';
import {isApproximate,loadApproximateReference,restoreApproximateReference,assertApproximateCases,approximatePointProbeSources,approximateAdapterSources} from './approximate-harness.mjs';
const row={id:'math_sin',result:'number',resultEncoding:'approximate-f64-scalar-v1',approximateValidation:{protocol:'mpfr-directed-f64-scalar-v1',method:'sin',operationArity:1,policySha256:'1934384586677acf640c052c408b4adb77aa324810bf7397b6a4ca6e479416e1',referencePending:true}};
const exact={id:'old_number',result:'number',resultEncoding:'ieee-f64-bits-v1'};
test('explicit approximate adapters bypass checksum without changing ordinary Number classification',()=>{
 assert.equal(isNumberResult(row),true);assert.equal(requiresExactResult(row),true);assert.equal(isApproximate(row),true);assert.equal(isApproximate(exact),false);
 assert.equal(adaptRustDriver('unused',row),approximateAdapterSources['approximate-driver.rs']);assert.equal(adaptTSDriver('unused',row),approximateAdapterSources['approximate-driver.ts']);
 assert.match(adaptRustDriver('unused',exact),/number_checksum/);assert.match(adaptTSDriver('unused',exact),/checksum/);
 assert.throws(()=>rustChecksum(row),/no checksum/);assert.throws(()=>adaptRustDriver('unused',row,{probe:true}),/distinct untimed wrapper/);
 assert.throws(()=>isApproximate({...row,result:'number[]'}),/explicit scalar/);
});
test('artifact loading fails closed for pending reference, path hashes and missing historical snapshots',()=>{
 assert.throws(()=>loadApproximateReference(row,{tsSha256:'0'.repeat(64),tsrSha256:'0'.repeat(64)}),/generation pending/);
 const altered={...row,approximateValidation:{...row.approximateValidation,referencePending:false,policySha256:'../outside'}};
 const directory=fs.mkdtempSync(path.join(os.tmpdir(),'tsrust-approximate-artifacts-'));
 try{assert.throws(()=>loadApproximateReference(altered,{}, {directory}),/SHA256 required/);}finally{fs.rmSync(directory,{recursive:true});}
 assert.throws(()=>restoreApproximateReference(row,{},undefined),/snapshot required/);
 assert.throws(()=>assertApproximateCases({cases:new Map()},[[131072,7]]),/case missing/);
});
test('untimed probes use the actual separate helper, exact raw inputs and fail on signature drift',()=>{
 // Text-generation fixture only; this is not compiler ABI acceptance or MPFR evidence.
 const emitted='fn function0(value0: i64, value1: i64) -> f64 { 0.0 }\nfn function1(value0: f64) -> f64 { value0.sin() }\nfn main() {}';
 const ts='function workload(n: number, seed: number): number { return 0; }\nfunction operation(argument0: number): number { return Math.sin(argument0); }';
 const reference={bundle:{points:[{id:'mechanical-only',inputBits:['8000000000000000']}]}};
 const generated=approximatePointProbeSources(emitted,ts,row,reference);
 assert.match(generated.rs,/function1\(f64::from_bits\(0x8000000000000000_u64\)\)/);assert.match(generated.rs,/fn source_main\(\)/);assert.match(generated.rs,/r#"\{\{"protocol"/);
 assert.match(generated.ts,/operation\(probeFromBits\(point\[0\]\)\)\)/);assert.match(generated.ts,/8000000000000000/);assert.doesNotMatch(generated.ts,/performance\.now/);
 assert.throws(()=>approximatePointProbeSources(emitted.replace('value0: f64','value0: i64'),ts,row,reference),/native operation helper ABI/);
 assert.throws(()=>approximatePointProbeSources(emitted,ts.replace('argument0: number','argument0: string'),row,reference),/TypeScript operation helper ABI/);
});
test('approximate allocation wrapper retains standard allocator core and distinct raw untimed result',()=>{
 const old=fs.readFileSync(new URL('./allocation-driver.rs',import.meta.url),'utf8'),wrapper=approximateAdapterSources['approximate-allocation-driver.rs'];
 assert.equal(wrapper.split('fn main()')[0],old.split('fn main()')[0]);
 assert.match(wrapper,/let seed: i64 = arguments\[2\]/);assert.match(wrapper,/value\.to_bits\(\)/);assert.match(wrapper,/native-approximate-allocation-result-v1/);assert.doesNotMatch(wrapper,/checksum|Instant|durations/);
 assert.ok(wrapper.indexOf('let actual_bits')>wrapper.indexOf('let peak ='));
});
// Required later with genuine references: all22 helpers and native/TS wrapper O0/O3,
// approved input bits/order/case identity, every point probe on all8 engines,
// run verify-only and real samples, report regeneration from immutable snapshots,
// scalar error preservation and outside-endpoint tampering, profile3x disposal,
// and unchanged exact tests/golden output. No fabricated accepted numeric fixtures.
