import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNativeByteCopyAppendSource} from './native-byte-copy-append-source-contract.mjs';
import {expectedNativeByteCopyAppendValues, expectedNativeByteCopyAppendObservation} from './native-byte-copy-append-oracle.mjs';
import {expectedExactResult, expectedChecksum} from './oracle.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const manifest = JSON.parse(read('./manifest.json'));
const row = manifest.workloads.find(value => value.id === 'native_byte_copy_append');
test('Native byte copy/append retains atomic append and independent copy', () => {
    assert.ok(row, 'Native byte copy/append workload registered');
    assert.equal(manifest.workloads.filter(value => value.id === row.id).length, 1);
    assert.equal(row.measured, false);
    assert.equal(reportCategory(row.id), 'bytes');
    const emitted = 'fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
    assert.equal(assertABI(emitted, row, read('./' + row.source)).fallible, false);
    for (const [kind, file] of [['native', row.source], ['typescript', row.counterpart]]) {
        const source = read('./' + file);
        assertNativeByteCopyAppendSource(source, row, kind);
        assert.throws(() => assertNativeByteCopyAppendSource(source + ' ', row, kind));
        assert.throws(() => assertNativeByteCopyAppendSource(source, {...row, operationsPerIteration: 2}, kind));
        const key = kind === 'native' ? 'sourceSha256' : 'counterpartSha256';
        for (const from of ['const add = append;', 'const duplicate = copy;', 'const independent = read(part,0);', 'const preserved = read(bytes,3);']) {
            const changed = replaceCodeSnippet(source, from, '');
            assert.throws(() => assertNativeByteCopyAppendSource(changed, {...row, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
        }
    }
    assert.deepEqual(expectedNativeByteCopyAppendValues(row.id, 3, 255, 'bytes'), [[2,1,10,1,7,2,1,255,1,65,0,5,4],[2,1,10,1,7,2,1,0,1,65,0,5,4],[2,1,10,1,7,2,1,1,1,65,0,5,4]]);
    for (const seed of [0,7,1000000]) for (const n of [0,1,37,2048]) {
        const expected = expectedNativeByteCopyAppendObservation(row.id, n, seed, 'bytes');
        assert.deepEqual(expectedExactResult(row.id, n, seed, 'bytes'), expected);
        assert.equal(expectedChecksum(row.id, n, seed, 'bytes'), expected.checksum);
    }
    assert.throws(() => expectedNativeByteCopyAppendValues(row.id, -1, 0, 'bytes'));
    assert.throws(() => expectedNativeByteCopyAppendValues(row.id, 1, 0, 'other'));
    for (const file of ['./run.mjs', './profile-native.mjs']) {
        const source = read(file);
        assert.ok(source.includes('assertNativeByteCopyAppendSource(source'));
        assert.ok(source.includes("'native-byte-copy-append-oracle.mjs'"));
    }
});
