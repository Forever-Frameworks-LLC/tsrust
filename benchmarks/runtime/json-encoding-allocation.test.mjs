import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {nativeEncodingAllocation,directEncodingAllocation,nodeEncodingAllocation} from './json-encoding-allocation.mjs';
const driver=fs.readFileSync(new URL('./allocation-driver.rs',import.meta.url),'utf8');
test('encoding probes prepare input before counters and verify exact output',()=>{
 const emitted='fn tsr_result_function0(value0: i64, value1: i64, value2: &NumericArray<i64>) -> Result<String,TsrFunctionError0> {}\nfn main() {}';
 const text=nativeEncodingAllocation(emitted,driver,'encode_typed');
 assert.ok(text.indexOf('let mut input')<text.indexOf('ALLOCS.store(0'));
 assert.match(text,/assert_eq!\(&\*value, arguments\[2\].as_str\(\)\)/);
 assert.throws(()=>nativeEncodingAllocation(emitted.replace('&NumericArray<i64>','NumericArray<i64>'),driver,'encode_typed'));
 assert.match(directEncodingAllocation(driver,'encode_dynamic'),/serde_json::Value/);
 const nodeDriver=fs.readFileSync(new URL('./allocation-driver.ts',import.meta.url),'utf8');
 assert.match(nodeEncodingAllocation('function workload(n:number,seed:number,input:number[]):string{return JSON.stringify(input);}',nodeDriver),/value!==inputText/);
});
