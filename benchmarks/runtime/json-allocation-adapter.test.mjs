import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {adaptJsonAllocation, directJsonAllocation, nodeJsonAllocation, jsonAllocationInput, jsonAllocationChecksum} from './json-allocation-adapter.mjs';
const driver=fs.readFileSync(new URL('./allocation-driver.rs',import.meta.url),'utf8');
test('JSON allocation adapters preserve input/output disposal and concrete return ABI',()=>{
 for(const kind of ['dynamic','typed']){
  const result=kind==='dynamic'?'Record12':'NumericArray<i64>';
  const emitted=`fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<${result},TsrFunctionError0> { todo!() }\nfn main() {}`;
  const adapted=adaptJsonAllocation(emitted,driver,kind);
  assert.match(adapted,/let input = String::from\(arguments\[2\].clone\(\)\)/);
  assert.match(adapted,/match tsr_result_function0/);
  assert.match(adapted,/liveBytesAfterOutputDrop/);
  assert.match(adapted,/liveBytesAfterDrop/);
  assert.throws(()=>adaptJsonAllocation(emitted.replace('value2: &String','value2: String'),driver,kind));
  assert.throws(()=>adaptJsonAllocation(emitted,driver.replace('fn main()','fn changed()'),kind));
 }
});
test('numeric JSON allocation oracle is independent of decoding',()=>{
 for(const n of [0,1,17,10000]){
  const text=jsonAllocationInput(n);
  assert.equal(text,n===0?'[]':'['+Array.from({length:n},(_,i)=>String(i+7)).join(',')+']');
  assert.equal(jsonAllocationChecksum(n),n*7+n*(n-1)/2);
 }
 for(const n of [-1,1.5,Number.MAX_SAFE_INTEGER])assert.throws(()=>jsonAllocationInput(n));
});

test('direct Rust and Node adapters reject protocol drift',()=>{
 assert.match(directJsonAllocation(driver,'typed'),/serde_json::from_str::<Vec<i64>>/);
 assert.match(directJsonAllocation(driver,'dynamic'),/serde_json::from_str::<serde_json::Value>/);
 assert.throws(()=>directJsonAllocation(driver,'unknown'));
 const nodeDriver=fs.readFileSync(new URL('./allocation-driver.ts',import.meta.url),'utf8');
 const adapted=nodeJsonAllocation('function workload(n:number,seed:number,input:string):unknown{return JSON.parse(input);}',nodeDriver);
 assert.match(adapted,/workload\(0, 7, input\)/);
 assert.match(adapted,/workload\(n, 7, input\)/);
 assert.throws(()=>nodeJsonAllocation('',nodeDriver.replace('workload(n, 7)','changed(n, 7)')));
});

test('Unicode allocation input keeps escaped source and counts decoded UTF-8 bytes',()=>{
 assert.equal(jsonAllocationInput(1,'typed_strings'),'["é🦀","line\\n\\u0061"]');
 assert.equal(jsonAllocationInput(0,'dynamic_strings'),'[]');
 assert.equal(jsonAllocationChecksum(1000,'typed_strings'),12000);
 const emitted='fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<Array<String>,TsrFunctionError0> { todo!() }\nfn main() {}';
 assert.match(adaptJsonAllocation(emitted,driver,'typed_strings'),/value.iter\(\).*len\(\)/);
 assert.throws(()=>adaptJsonAllocation(emitted,driver,'typed'));
 assert.match(directJsonAllocation(driver,'typed_strings'),/Vec<std::string::String>/);
 const nodeDriver=fs.readFileSync(new URL('./allocation-driver.ts',import.meta.url),'utf8');
 assert.match(nodeJsonAllocation('',nodeDriver,'typed_strings'),/TextEncoder/);
});

test('nested and selective inputs retain nested integer values with independent sums',()=>{
 assert.equal(jsonAllocationInput(2,'typed_nested'),'[[7,8],[9,10]]');
 assert.equal(jsonAllocationChecksum(2,'typed_nested'),34);
 const selected=jsonAllocationInput(2,'typed_selective');
 assert.match(selected,/"items":\[\[7,8\],\[9,10\]\]/);
 assert.match(selected,/"ignored":/);
 assert.equal(jsonAllocationChecksum(2,'typed_selective'),34);
 const emitted='fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<Array<NumericArray<i64>>,TsrFunctionError0> { todo!() }\nfn main() {}';
 assert.match(adaptJsonAllocation(emitted,driver,'typed_nested'),/flat_map/);
 assert.throws(()=>adaptJsonAllocation(emitted,driver,'typed'));
 assert.match(directJsonAllocation(driver,'typed_nested'),/Vec<Vec<i64>>/);
});

test('selective record instrumentation fails closed when its field ABI changes',()=>{
 const emitted='struct Record0 { field0: Array<NumericArray<i64>> }\nfn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<Record0,TsrFunctionError0> { todo!() }\nfn main() {}';
 assert.match(adaptJsonAllocation(emitted,driver,'typed_selective'),/value.field0.iter/);
 assert.throws(()=>adaptJsonAllocation(emitted.replace('field0:','field1:'),driver,'typed_selective'));
 assert.match(directJsonAllocation(driver,'typed_selective'),/IgnoredAny/);
});

test('failure probes reject malformed and late type-mismatch input without retaining errors',()=>{
 assert.equal(jsonAllocationInput(0,'dynamic_invalid'),'[?]');
 assert.equal(jsonAllocationInput(1,'typed_invalid'),'["é🦀",?]');
 assert.equal(jsonAllocationInput(1,'typed_mismatch'),'["é🦀",7]');
 assert.equal(jsonAllocationChecksum(1000,'typed_mismatch'),1);
 const emitted='fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<Array<String>,TsrFunctionError0> { todo!() }\nfn main() {}';
 assert.match(adaptJsonAllocation(emitted,driver,'typed_mismatch'),/Err\(error\)=>\{drop\(error\);1i64\}/);
 assert.match(directJsonAllocation(driver,'typed_mismatch'),/is_err\(\) as i64/);
 const nodeDriver=fs.readFileSync(new URL('./allocation-driver.ts',import.meta.url),'utf8');
 assert.match(nodeJsonAllocation('function workload(n:number,seed:number,input:string):unknown{return JSON.parse(input);}',nodeDriver,'dynamic_invalid'),/catch/);
});

test('deep dynamic probe keeps 32 container levels and checks the leaf values',()=>{
 const input=jsonAllocationInput(3,'dynamic_deep');
 assert.equal(input,'['.repeat(31)+'[7,8,9]'+']'.repeat(31));
 assert.equal(jsonAllocationChecksum(3,'dynamic_deep'),24);
 assert.match(directJsonAllocation(driver,'dynamic_deep'),/0\.\.31/);
 const nodeDriver=fs.readFileSync(new URL('./allocation-driver.ts',import.meta.url),'utf8');
 assert.match(nodeJsonAllocation('',nodeDriver,'dynamic_deep'),/depth<31/);
});

test('Unicode and deep encoding oracles describe exact canonical output',()=>{
 const expected='["é🦀","line\\na"]';
 assert.equal(jsonAllocationInput(1,'encode_typed_strings'),expected);
 assert.equal(jsonAllocationChecksum(1,'encode_typed_strings'),Buffer.from(expected).reduce((sum,byte)=>sum+byte,0));
 assert.equal(jsonAllocationInput(1,'encode_dynamic_deep'),'['.repeat(32)+'7'+']'.repeat(32));
});
