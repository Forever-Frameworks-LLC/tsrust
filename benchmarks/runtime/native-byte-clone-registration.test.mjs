import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNativeByteCloneSource} from './native-byte-clone-source-contract.mjs';
import {expectedNativeByteCloneValues, expectedNativeByteCloneObservation} from './native-byte-clone-oracle.mjs';
import {expectedExactResult, expectedChecksum} from './oracle.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const manifest = JSON.parse(read('./manifest.json'));
const row = manifest.workloads.find(value => value.id === 'native_byte_clone');
test('Native byte clone retains direct and aggregate independence', () => {
    assert.ok(row, 'Native byte clone workload registered');
    assert.equal(manifest.workloads.filter(value => value.id === row.id).length, 1);
    assert.equal(row.measured, false);
    assert.equal(reportCategory(row.id), 'bytes');
    const emitted = 'fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
    assert.equal(assertABI(emitted, row, read('./' + row.source)).fallible, false);
    const fallible = 'fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<Array<NumericArray<i64>>, TsrFunctionError0> { todo!() }\nfn main() {}';
    assert.equal(assertABI(fallible, row, read('./' + row.source)).fallible, true);
    assert.throws(() => assertABI(fallible.replace('TsrFunctionError0', 'UnknownError'), row, read('./' + row.source)));
    for (const [kind, file] of [['native', row.source], ['typescript', row.counterpart]]) {
        const source = read('./' + file);
        assertNativeByteCloneSource(source, row, kind);
        assert.throws(() => assertNativeByteCloneSource(source + ' ', row, kind));
        assert.throws(() => assertNativeByteCloneSource(source, {...row, operationsPerIteration: 2}, kind));
        const key = kind === 'native' ? 'sourceSha256' : 'counterpartSha256';
        for (const from of ['write(duplicate,1,7);', 'write(rows[0].bytes,0,(byte + 1) % 256);', 'const byte = (i + seed) % 256;', 'return output;']) {
            const changed = replaceCodeSnippet(source, from, '');
            assert.throws(() => assertNativeByteCloneSource(changed, {...row, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
        }
    }
    assert.deepEqual(expectedNativeByteCloneValues(row.id, 3, 255, 'bytes'), [[255,7,255,255,0,3,3],[0,7,0,0,1,3,3],[1,7,1,1,2,3,3]]);
    for (const seed of [0,7,1000000]) for (const n of [0,1,37,2048]) {
        const expected = expectedNativeByteCloneObservation(row.id, n, seed, 'bytes');
        assert.deepEqual(expectedExactResult(row.id, n, seed, 'bytes'), expected);
        assert.equal(expectedChecksum(row.id, n, seed, 'bytes'), expected.checksum);
    }
    assert.throws(() => expectedNativeByteCloneValues(row.id, -1, 0, 'bytes'));
    assert.throws(() => expectedNativeByteCloneValues(row.id, 1, 0, 'other'));
    for (const file of ['./run.mjs', './profile-native.mjs']) {
        const source = read(file);
        assert.ok(source.includes('assertNativeByteCloneSource(source'));
        assert.ok(source.includes("'native-byte-clone-oracle.mjs'"));
    }
});
