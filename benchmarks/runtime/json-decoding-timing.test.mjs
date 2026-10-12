import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {directJsonAllocation} from './json-allocation-adapter.mjs';
test('decoding driver prepares input outside timing and observes decoded values',()=>{
 const driver=fs.readFileSync(new URL('./json-decoding-timing-driver.rs',import.meta.url),'utf8');
 for(const kind of ['typed','dynamic','typed_strings','typed_nested','typed_selective','typed_invalid','typed_mismatch','dynamic_deep']){
  const text=directJsonAllocation(driver,kind);
  assert.ok(text.indexOf('let input')<text.indexOf('let cold='));
  assert.ok(text.includes('probe_checksum(&value)'));
  assert.ok(!text.includes('assert_eq!(&*value,arguments[2]'));
 }
});
