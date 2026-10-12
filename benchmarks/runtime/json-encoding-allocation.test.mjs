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

test('Unicode typed encoding keeps strings owned and rejects numeric ABI',()=>{
 const emitted='fn tsr_result_function0(value0: i64, value1: i64, value2: &Array<String>) -> Result<String,TsrFunctionError0> {}\nfn main() {}';
 assert.match(nativeEncodingAllocation(emitted,driver,'encode_typed_strings'),/input.push\(String::from\(value\)\)/);
 assert.throws(()=>nativeEncodingAllocation(emitted,driver,'encode_typed'));
 assert.match(directEncodingAllocation(driver,'encode_typed_strings'),/Vec<String>/);
});

test('object encoding checks record layout and drops preparation trees before counters',()=>{
 const emitted='struct Record0 { field0: bool, field1: Array<NumericArray<i64>>, field2: String }\nfn tsr_result_function0(value0: i64, value1: i64, value2: &Record0) -> Result<String,TsrFunctionError0> {}\nfn main() {}';
 const adapted=nativeEncodingAllocation(emitted,driver,'encode_typed_object');
 assert.ok(adapted.indexOf('drop(raw)')<adapted.indexOf('ALLOCS.store(0'));
 assert.throws(()=>nativeEncodingAllocation(emitted.replace('field0: bool','field0: i64'),driver,'encode_typed_object'));
 assert.match(directEncodingAllocation(driver,'encode_typed_object'),/serialize_struct/);
});
