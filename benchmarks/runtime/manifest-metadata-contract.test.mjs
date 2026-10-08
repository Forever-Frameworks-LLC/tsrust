import test from 'node:test';
import assert from 'node:assert/strict';
import {manifestMetadataHash} from './manifest-metadata-contract.mjs';

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
