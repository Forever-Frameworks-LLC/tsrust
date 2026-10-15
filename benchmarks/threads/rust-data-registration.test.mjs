import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {containsCodeSnippet} from '../runtime/source-contract-tokens.mjs';
const hash=value=>createHash('sha256').update(value).digest('hex');
test('Rust data workload retains matched recurrence, oracle and timing protocol',()=>{
 const manifest=JSON.parse(fs.readFileSync(new URL('./rust-data-manifest.json',import.meta.url)));
 for(const [key,fingerprint] of [['source','sourceSha256'],['counterpart','counterpartSha256'],['rust','rustSha256']]){
  const source=fs.readFileSync(new URL(manifest[key],import.meta.url),'utf8');assert.equal(hash(source),manifest[fingerprint]);
 }
 const native=fs.readFileSync(new URL('./rust-data-throughput.tsr',import.meta.url),'utf8');
 const node=fs.readFileSync(new URL('./rust-data-throughput.ts',import.meta.url),'utf8');
 for(const source of [native,node])for(const snippet of ['warmup<8','sample<31'])assert.ok(containsCodeSnippet(source,snippet));
 const rust=fs.readFileSync(new URL('./rust-data-compute.rs',import.meta.url),'utf8');assert.ok(rust.includes('0..250000'));assert.ok(rust.includes('value*48271%2147483647'));
 const harness=fs.readFileSync(new URL('./run.mjs',import.meta.url),'utf8');assert.ok(harness.includes('48271n%2147483647n'));assert.ok(harness.includes('rustSha256'));
});
