import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {ownedIteratorIDs,ownedIteratorChecksum,assertOwnedIteratorSource} from './owned-iterators-contract.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url),'utf8'));
test('owned iterator rows retain matched execution and independent ordered oracle',()=>{
 for(const id of ownedIteratorIDs){
  const row=manifest.workloads.find(row=>row.id===id);assert.ok(row);
  for(const[engine,key]of [['native','source'],['typescript','counterpart']]){
   const source=fs.readFileSync(new URL(row[key],import.meta.url),'utf8');assertOwnedIteratorSource(source,row,engine);
   const changed=replaceCodeSnippet(source,'j<64','j<63');assert.notEqual(changed,source);assert.throws(()=>assertOwnedIteratorSource(changed,row,engine));
  }
  assert.equal(ownedIteratorChecksum(id,0,7),0);
  assert.ok(Number.isSafeInteger(ownedIteratorChecksum(id,3,7)));
 }
 assert.equal(ownedIteratorChecksum('iterator_flat_array',3,7),ownedIteratorChecksum('iterator_flat_iterator',3,7));
 assert.notEqual(ownedIteratorChecksum('iterator_take_array',1,7),ownedIteratorChecksum('iterator_map_filter_reduce',1,7));
});

test('owned iterator sources and Rust operations reject rehashed semantic mutations',async()=>{
 const {createHash}=await import('node:crypto');const {replaceCodeSnippet}=await import('./source-contract-tokens.mjs');const {assertOwnedIteratorReference,ownedIteratorReferenceOperations}=await import('./owned-iterators-contract.mjs');const {adaptRustDriver}=await import('./adapters.mjs');const hash=source=>createHash('sha256').update(source).digest('hex');const driver=fs.readFileSync(new URL('driver.rs',import.meta.url),'utf8');
 assert.equal(manifest.processes,3);assert.equal(manifest.samples,31);assert.equal(manifest.warmups,8);
 for(const id of ownedIteratorIDs){const row=manifest.workloads.find(row=>row.id===id);assert.equal(row.measured,false);for(const [engine,key,fingerprint]of [['native','source','sourceSha256'],['typescript','counterpart','counterpartSha256']]){const source=fs.readFileSync(new URL(row[key],import.meta.url),'utf8');assertOwnedIteratorSource(source,row,engine);assert.throws(()=>assertOwnedIteratorSource(source+' ',row,engine));const changed=replaceCodeSnippet(source,'j<64','j<63');assert.notEqual(changed,source);assert.throws(()=>assertOwnedIteratorSource(changed,{...row,[fingerprint]:hash(changed)},engine));}const source=fs.readFileSync(new URL(row.rustReference,import.meta.url),'utf8');assertOwnedIteratorReference(source,row);adaptRustDriver(driver,row,{reference:true,emitted:source});for(const operation of ownedIteratorReferenceOperations[id]){const changed=replaceCodeSnippet(source,operation,'omitted_operation');assert.notEqual(changed,source);const altered={...row,rustReferenceSha256:hash(changed)};assert.throws(()=>assertOwnedIteratorReference(changed,altered));assert.throws(()=>adaptRustDriver(driver,altered,{reference:true,emitted:changed}));}}
});
