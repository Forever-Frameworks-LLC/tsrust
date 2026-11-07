import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertRustOwnedAdapterSource} from './rust-owned-adapters-source-contract.mjs';
import {rustOwnedAdapterIds,expectedRustOwnedAdapterChecksum} from './rust-owned-adapters-oracle.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url)));
test('all four actual bridge workload/control identities and observers are guarded',()=>{
 const rows=manifest.workloads.filter(row=>row.family==='rust-owned-adapters');assert.deepEqual(rows.map(row=>row.id),rustOwnedAdapterIds);
 for(const row of rows)for(const [engine,key]of [['native','source'],['typescript','counterpart']]){
  const source=fs.readFileSync(new URL(row[key],import.meta.url),'utf8');assert.doesNotThrow(()=>assertRustOwnedAdapterSource(source,row,engine));assert.throws(()=>assertRustOwnedAdapterSource(source.replace('checksum * 131','checksum * 127'),row,engine));
 }
});
test('independent checksums observe all tags and empty work',()=>{
 for(const id of rustOwnedAdapterIds)assert.equal(expectedRustOwnedAdapterChecksum(id,0,7),0);
 assert.equal(expectedRustOwnedAdapterChecksum('rust_owned_scalar',1,7),22);
 assert.equal(expectedRustOwnedAdapterChecksum('rust_owned_strings',1,7),21);
 assert.equal(expectedRustOwnedAdapterChecksum('rust_owned_records',1,7),24);
 for(const [seed,observed]of [[0,1],[1,2],[2,12]])assert.equal(expectedRustOwnedAdapterChecksum('rust_owned_nested_option',1,seed),observed);
 assert.throws(()=>expectedRustOwnedAdapterChecksum('missing',1,7));
});
