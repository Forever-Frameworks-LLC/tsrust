import assert from 'node:assert/strict';
import test from 'node:test';
import {rustReferenceSource} from './rust-reference-source.mjs';

test('size variants share a Rust source without losing workload identity',()=>{
 assert.equal(rustReferenceSource({id:'native_interfaces'}),'parity/native_interfaces.rs');
 for(const size of ['token','4k','1m'])assert.equal(rustReferenceSource({id:`bytes_hex_${size}`,rustReference:'parity/bytes_to_hex.rs'}),'parity/bytes_to_hex.rs');
 for(const rustReference of ['../outside.rs','parity/../outside.rs','/tmp/reference.rs','parity/a.rs/extra',42,null])assert.throws(()=>rustReferenceSource({id:'invalid',rustReference}),/Invalid hand-written Rust source/);
});
