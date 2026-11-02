import test from 'node:test';
import assert from 'node:assert/strict';
import {manifestMetadataHash,withCurrentSourceFingerprints} from './manifest-metadata-contract.mjs';
import fs from 'node:fs';

test('manifest metadata ignores only independently checked formatting fingerprints', () => {
    const rows = [{id: 'first', sourceSha256: 'old', counterpartSha256: 'old-ts', iterations: 7, samples: 11}, {id: 'second', result: 'int[]'}];
    const original = manifestMetadataHash(rows);
    assert.equal(manifestMetadataHash([{...rows[0], sourceSha256: 'new', counterpartSha256: 'new-ts'}, rows[1]]), original);
    for (const change of [{iterations: 8}, {samples: 12}, {source: 'other.tsr'}, {result: 'number[]'}, {operationsPerIteration: 2}]) {
        assert.notEqual(manifestMetadataHash([{...rows[0], ...change}, rows[1]]), original);
    }
    assert.notEqual(manifestMetadataHash(rows.toReversed()), original);
    assert.notEqual(manifestMetadataHash(rows.slice(1)), original);
});

test('registration fragments use live source identities without changing operation contracts',()=>{
    const row=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url),'utf8')).workloads.find(row=>row.sourceSha256&&row.counterpartSha256);
    const fragment={...row,sourceSha256:'historical',counterpartSha256:'historical-ts',notes:'preserve this contract'};
    const expected={...fragment,sourceSha256:row.sourceSha256,counterpartSha256:row.counterpartSha256};
    assert.deepEqual(withCurrentSourceFingerprints(fragment),expected);
    assert.deepEqual(withCurrentSourceFingerprints([fragment]),[expected]);
    assert.equal(fragment.sourceSha256,'historical');
    assert.throws(()=>withCurrentSourceFingerprints({...fragment,id:'missing-workload'}),/Unknown/);
    assert.throws(()=>withCurrentSourceFingerprints({...fragment,source:'other.tsr'}),/review/);
});
