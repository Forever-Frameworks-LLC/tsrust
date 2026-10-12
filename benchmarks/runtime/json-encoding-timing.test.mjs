import test from 'node:test';
import assert from 'node:assert/strict';
import {validateEncodingTiming} from './json-encoding-timing-contract.mjs';
test('timing evidence requires full protocol and stable exact-output checksum',()=>{
 const row={checksum:42,coldMs:1,durations:Array(31).fill(1),iterations:100,warmups:8};
 assert.doesNotThrow(()=>validateEncodingTiming(row,42));
 for(const patch of [{checksum:0},{coldMs:-1},{durations:[1]},{durations:Array(31).fill(NaN)},{iterations:1},{warmups:0}])assert.throws(()=>validateEncodingTiming({...row,...patch},42));
});
