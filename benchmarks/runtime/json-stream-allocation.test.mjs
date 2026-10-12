import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {streamInput,nativeStreamAllocation,directStreamAllocation,nodeStreamAllocation} from './json-stream-allocation.mjs';
const driver=fs.readFileSync(new URL('./allocation-driver.rs',import.meta.url),'utf8');
test('stream probes read one file once regardless of record count',()=>{
 assert.equal(streamInput(0),'');
 assert.equal(streamInput(2).split('\n').length,3);
 const emitted='fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<i64,TsrFunctionError0> {}\nfn main() {}';
 const native=nativeStreamAllocation(emitted,driver);
 assert.match(native,/tsr_result_function0\(1,7,&input\)/);
 assert.throws(()=>nativeStreamAllocation(emitted.replace('Result<i64','Result<String'),driver));
 assert.match(directStreamAllocation(driver),/BufReader::with_capacity\(8192/);
 const nodeDriver=fs.readFileSync(new URL('./allocation-driver.ts',import.meta.url),'utf8');
 assert.match(nodeStreamAllocation('',nodeDriver),/workload\(1, 7, input\)/);
});
