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
