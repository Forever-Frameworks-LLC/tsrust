import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNativeByteAccessSource} from './native-byte-access-source-contract.mjs';
import {expectedNativeByteAccessValues, expectedNativeByteAccessObservation} from './native-byte-access-oracle.mjs';
import {expectedExactResult, expectedChecksum} from './oracle.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const manifest = JSON.parse(read('./manifest.json'));
const row = manifest.workloads.find(value => value.id === 'native_byte_access');
test('Native byte access retains mutation atomicity and error precedence', () => {
    assert.ok(row, 'Native byte access workload registered');
    assert.equal(manifest.workloads.filter(value => value.id === row.id).length, 1);
    assert.equal(row.measured, true);
    assert.equal(row.measurementStatus, 'measured-resources-bytes-family');
    assert.equal(row.measurementEvidence, 'runtime/results/s07-measurement-proof-2026-10-11.json');
    assert.equal(reportCategory(row.id), 'bytes');
    const emitted = 'fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
    assert.equal(assertABI(emitted, row, read('./' + row.source)).fallible, false);
    for (const [kind, file] of [['native', row.source], ['typescript', row.counterpart]]) {
        const source = read('./' + file);
        assertNativeByteAccessSource(source, row, kind);
        assert.throws(() => assertNativeByteAccessSource(source + ' ', row, kind));
        assert.throws(() => assertNativeByteAccessSource(source, {...row, operationsPerIteration: 2}, kind));
        const key = kind === 'native' ? 'sourceSha256' : 'counterpartSha256';
        for (const from of ['const write = set;', 'const updated = read(bytes);', 'const afterIndex = read(bytes);', 'const afterByte = read(bytes);']) {
            const changed = replaceCodeSnippet(source, from, '');
            assert.throws(() => assertNativeByteAccessSource(changed, {...row, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
        }
    }
    assert.deepEqual(expectedNativeByteAccessValues(row.id, 3, 255, 'bytes'), [[255,0,1,0,0,11,0,10,1,11,3,3],[0,1,1,1,1,11,0,10,1,11,3,3],[1,2,1,2,2,11,0,10,1,11,3,3]]);
    for (const seed of [0,7,1000000]) for (const n of [0,1,37,2048]) {
        const expected = expectedNativeByteAccessObservation(row.id, n, seed, 'bytes');
        assert.deepEqual(expectedExactResult(row.id, n, seed, 'bytes'), expected);
        assert.equal(expectedChecksum(row.id, n, seed, 'bytes'), expected.checksum);
    }
    assert.throws(() => expectedNativeByteAccessValues(row.id, -1, 0, 'bytes'));
    assert.throws(() => expectedNativeByteAccessValues(row.id, 1, 0, 'other'));
    for (const file of ['./run.mjs', './profile-native.mjs']) {
        const source = read(file);
        assert.ok(source.includes('assertNativeByteAccessSource(source'));
        assert.ok(source.includes("'native-byte-access-oracle.mjs'"));
    }
});
