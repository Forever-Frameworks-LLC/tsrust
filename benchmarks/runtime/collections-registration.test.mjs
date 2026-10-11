import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertCollectionSource, expectedCollectionValues, expectedCollectionObservation} from './collections-contract.mjs';
import {expectedExactResult, expectedChecksum} from './oracle.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {reportCategory} from './report.mjs';
import {assertABI} from './adapters.mjs';
const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const row = JSON.parse(read('./manifest.json')).workloads.find(row => row.id === 'native_deque_clone');
test('Deep deque clone keeps independent nested storage and matched observations', () => {
  assert.ok(row);
  assert.equal(reportCategory(row.id),'collections');
  assert.equal(row.measured, false);
  assert.equal(row.measurementStatus, 'pending-collections-family');
  for (const [kind, file] of [['native',row.source],['typescript',row.counterpart]]) {
    const source = read('./'+file);
    assertCollectionSource(source,row,kind);
    for (const required of ['changed.values.push(value+1);', kind==='native' ? 'const copied=clone(original);' : 'const copied=original.map(item=>({values:item.values.slice()}));']) {
      const changed=replaceCodeSnippet(source,required,'');
      const key=kind==='native'?'sourceSha256':'counterpartSha256';
      assert.throws(()=>assertCollectionSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind),/Collection scenario/);
    }
  }
  assert.deepEqual(expectedCollectionValues(row.id,3,255,'collections'),[[2,255,3,256,0,0],[2,0,3,1,0,0],[2,1,3,2,0,0]]);
  for(const n of [0,1,17,2048]) {
    const expected=expectedCollectionObservation(row.id,n,7,'collections');
    assert.deepEqual(expectedExactResult(row.id,n,7,'collections'),expected);
    assert.equal(expectedChecksum(row.id,n,7,'collections'),expected.checksum);
  }
  const emitted='fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
  assert.equal(assertABI(emitted,row,{source:read('./'+row.source)}).fallible,false);
});
test('Timing and allocation runners guard and fingerprint collection contracts', () => {
  for(const file of ['./run.mjs','./profile-native.mjs']) {
    const source=read(file);
    assert.ok(source.includes('assertCollectionSource(source'));
    assert.ok(source.includes("'collections-contract.mjs'"));
  }
});

test('Heap reference preserves ordering, cloning and independent arithmetic', () => {
  const heap=JSON.parse(read('./manifest.json')).workloads.find(row=>row.id==='native_heap');
  assert.ok(heap);assert.equal(heap.measured,false);assert.equal(reportCategory(heap.id),'collections');
  for(const [kind,file,guard] of [['native',heap.source,'const copied=clone(values);'],['typescript',heap.counterpart,'values.sort((a,b)=>a-b);']]) {
    const source=read('./'+file);assertCollectionSource(source,heap,kind);
    const changed=replaceCodeSnippet(source,guard,'');const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertCollectionSource(changed,{...heap,[key]:createHash('sha256').update(changed).digest('hex')},kind),/Collection scenario/);
  }
  assert.deepEqual(expectedCollectionValues(heap.id,3,255,'collections'),[[9,255,283,0],[9,8,36,0],[9,9,45,0]]);
  for(const seed of [0,7,248,255,1000000])for(const n of [0,1,17,2048]){
    const expected=expectedCollectionObservation(heap.id,n,seed,'collections');
    assert.deepEqual(expectedExactResult(heap.id,n,seed,'collections'),expected);
    assert.equal(expectedChecksum(heap.id,n,seed,'collections'),expected.checksum);
  }
});
