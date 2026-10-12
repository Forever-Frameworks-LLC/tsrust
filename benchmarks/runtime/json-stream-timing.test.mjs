import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {nodeStreamAllocation} from './json-stream-allocation.mjs';
test('stream first operation and warm batches call the same full workload',()=>{
 const driver=fs.readFileSync(new URL('./json-stream-timing-driver.ts',import.meta.url),'utf8');
 for(const writer of [false,true]){
  const text=nodeStreamAllocation('',driver,writer);
  assert.ok(text.includes('const call=()=>n<0?workload(0, 7, input):workload('+(writer?'n':'1')+', 7, input)'));
  assert.ok(text.includes('result=call()'));
 }
});
