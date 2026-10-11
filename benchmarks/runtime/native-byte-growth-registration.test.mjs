import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNativeByteGrowthSource} from './native-byte-growth-source-contract.mjs';
import {expectedNativeByteGrowthValues, expectedNativeByteGrowthObservation} from './native-byte-growth-oracle.mjs';
import {expectedExactResult, expectedChecksum} from './oracle.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const manifest = JSON.parse(read('./manifest.json'));
const row = manifest.workloads.find(value => value.id === 'native_byte_growth');
test('Native byte growth retains default fill, atomic failures and retained prefixes', () => {
    assert.ok(row, 'Native byte growth workload registered');
    assert.equal(manifest.workloads.filter(value => value.id === row.id).length, 1);
    assert.equal(row.measured, true);
    assert.equal(row.measurementStatus, 'measured-resources-bytes-family');
    assert.equal(row.measurementEvidence, 'runtime/results/s07-measurement-proof-2026-10-11.json');
    assert.equal(reportCategory(row.id), 'bytes');
    const emitted = 'fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
    assert.equal(assertABI(emitted, row, read('./' + row.source)).fallible, false);
    for (const [kind, file] of [['native', row.source], ['typescript', row.counterpart]]) {
        const source = read('./' + file);
        assertNativeByteGrowthSource(source, row, kind);
        assert.throws(() => assertNativeByteGrowthSource(source + ' ', row, kind));
        assert.throws(() => assertNativeByteGrowthSource(source, {...row, operationsPerIteration: 2}, kind));
        const key = kind === 'native' ? 'sourceSha256' : 'counterpartSha256';
        for (const from of ['const change = resize;', 'const initial = read(bytes, 0);', 'const zero = read(bytes, 3);', 'const preserved = length(bytes);']) {
            const changed = replaceCodeSnippet(source, from, '');
            assert.throws(() => assertNativeByteGrowthSource(changed, {...row, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
        }
    }
    assert.deepEqual(expectedNativeByteGrowthValues(row.id, 3, 255, 'bytes'), [[255,1,0,10,4,1,1,255,255,255,3],[0,1,0,10,4,1,1,0,255,255,3],[1,1,0,10,4,1,1,1,255,255,3]]);
    for (const seed of [0,7,1000000]) for (const n of [0,1,37,2048]) {
        const expected = expectedNativeByteGrowthObservation(row.id, n, seed, 'bytes');
        assert.deepEqual(expectedExactResult(row.id, n, seed, 'bytes'), expected);
        assert.equal(expectedChecksum(row.id, n, seed, 'bytes'), expected.checksum);
    }
    assert.throws(() => expectedNativeByteGrowthValues(row.id, -1, 0, 'bytes'));
    assert.throws(() => expectedNativeByteGrowthValues(row.id, 1, 0, 'other'));
    for (const file of ['./run.mjs', './profile-native.mjs']) {
        const source = read(file);
        assert.ok(source.includes('assertNativeByteGrowthSource(source'));
        assert.ok(source.includes("'native-byte-growth-oracle.mjs'"));
    }
});
