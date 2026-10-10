import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNativeBytesSource} from './native-bytes-source-contract.mjs';
import {expectedNativeBytesValues, expectedNativeBytesObservation} from './native-bytes-oracle.mjs';
import {expectedExactResult, expectedChecksum} from './oracle.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const manifest = JSON.parse(read('./manifest.json'));
const row = manifest.workloads.find(value => value.id === 'native_bytes');
test('Native bytes retain validated copies, repeated borrowing and first-error observations', () => {
    assert.ok(row, 'Native byte workload registered');
    assert.equal(manifest.workloads.filter(value => value.id === row.id).length, 1);
    assert.equal(row.measured, false);
    assert.equal(reportCategory(row.id), 'bytes');
    const emitted = 'fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
    assert.equal(assertABI(emitted, row, read('./' + row.source)).fallible, false);
    for (const [kind, file] of [['native', row.source], ['typescript', row.counterpart]]) {
        const source = read('./' + file);
        assertNativeBytesSource(source, row, kind);
        assert.throws(() => assertNativeBytesSource(source + ' ', row, kind));
        assert.throws(() => assertNativeBytesSource(source, {...row, operationsPerIteration: 2}, kind));
        const key = kind === 'native' ? 'sourceSha256' : 'counterpartSha256';
        for (const from of ['const construct = fromArray;', 'const observe = length;', 'const invalid = i % 2 === 0 ? -1 : 256;', 'output.push([first, second, values[1], empty, offset, codeLength]);']) {
            const changed = replaceCodeSnippet(source, from, '');
            assert.throws(() => assertNativeBytesSource(changed, {...row, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
        }
    }
    assert.deepEqual(expectedNativeBytesValues(row.id, 3, 255, 'bytes'), [[3,3,255,0,1,10],[3,3,0,0,1,10],[3,3,1,0,1,10]]);
    for (const seed of [0,7,1000000]) for (const n of [0,1,37,2048]) {
        const expected = expectedNativeBytesObservation(row.id, n, seed, 'bytes');
        assert.deepEqual(expectedExactResult(row.id, n, seed, 'bytes'), expected);
        assert.equal(expectedChecksum(row.id, n, seed, 'bytes'), expected.checksum);
    }
    assert.throws(() => expectedNativeBytesValues(row.id, -1, 0, 'bytes'));
    assert.throws(() => expectedNativeBytesValues(row.id, 1, 0, 'other'));
    for (const file of ['./run.mjs', './profile-native.mjs']) {
        const source = read(file);
        assert.ok(source.includes('assertNativeBytesSource(source'));
        assert.ok(source.includes("'native-bytes-oracle.mjs'"));
    }
});
