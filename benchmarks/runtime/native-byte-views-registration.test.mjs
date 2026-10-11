import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNativeByteViewsSource} from './native-byte-views-source-contract.mjs';
import {expectedNativeByteViewsValues, expectedNativeByteViewsObservation} from './native-byte-views-oracle.mjs';
import {expectedExactResult, expectedChecksum} from './oracle.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const manifest = JSON.parse(read('./manifest.json'));
const row = manifest.workloads.find(value => value.id === 'native_byte_views');
test('Native byte views retain zero-copy reads, write-through and range errors', () => {
    assert.ok(row, 'Native byte access workload registered');
    assert.equal(manifest.workloads.filter(value => value.id === row.id).length, 1);
    assert.equal(row.measured, false);
    assert.equal(reportCategory(row.id), 'bytes');
    const emitted = 'fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
    assert.equal(assertABI(emitted, row, read('./' + row.source)).fallible, false);
    for (const [kind, file] of [['native', row.source], ['typescript', row.counterpart]]) {
        const source = read('./' + file);
        assertNativeByteViewsSource(source, row, kind);
        assert.throws(() => assertNativeByteViewsSource(source + ' ', row, kind));
        assert.throws(() => assertNativeByteViewsSource(source, {...row, operationsPerIteration: 2}, kind));
        const key = kind === 'native' ? 'sourceSha256' : 'counterpartSha256';
        for (const from of ['size = length(part);', 'head = first(part);', 'const written = (byte + 2) % 256;']) {
            const changed = replaceCodeSnippet(source, from, '');
            assert.throws(() => assertNativeByteViewsSource(changed, {...row, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
        }
    }
    assert.deepEqual(expectedNativeByteViewsValues(row.id, 3, 255, 'bytes'), [[3,255,255,0,2,1,11,2,1,5,9,1],[3,0,255,1,2,1,11,2,2,5,9,1],[3,1,255,2,2,1,11,2,3,5,9,1]]);
    for (const seed of [0,7,1000000]) for (const n of [0,1,37,2048]) {
        const expected = expectedNativeByteViewsObservation(row.id, n, seed, 'bytes');
        assert.deepEqual(expectedExactResult(row.id, n, seed, 'bytes'), expected);
        assert.equal(expectedChecksum(row.id, n, seed, 'bytes'), expected.checksum);
    }
    assert.throws(() => expectedNativeByteViewsValues(row.id, -1, 0, 'bytes'));
    assert.throws(() => expectedNativeByteViewsValues(row.id, 1, 0, 'other'));
    for (const file of ['./run.mjs', './profile-native.mjs']) {
        const source = read(file);
        assert.ok(source.includes('assertNativeByteViewsSource(source'));
        assert.ok(source.includes("'native-byte-views-oracle.mjs'"));
    }
});

for (const id of ['native_byte_views_recursive', 'native_byte_views_linked']) test(`${id} retains traversal and shared owner observations`, () => {
    const recursive = manifest.workloads.find(value => value.id === id);
    assert.ok(recursive, 'Recursive view workload registered');
    assert.equal(recursive.measured, false);
    assert.equal(reportCategory(recursive.id), 'bytes');
    const emitted = 'fn tsr_result_function2(value0: i64, value1: i64, value2: &String) -> Result<Array<NumericArray<i64>>, TsrFunctionError2> { todo!() }\nfn main() {}';
    assert.equal(assertABI(emitted, recursive, read('./' + recursive.source)).fallible, true);
    for (const [kind, file] of [['native', recursive.source], ['typescript', recursive.counterpart]]) {
        const source = read('./' + file);
        assertNativeByteViewsSource(source, recursive, kind);
        const changed = replaceCodeSnippet(source, 'sum += first(root);', 'sum += 1;');
        const key = kind === 'native' ? 'sourceSha256' : 'counterpartSha256';
        assert.throws(() => assertNativeByteViewsSource(changed, {...recursive, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
    }
    assert.deepEqual(expectedNativeByteViewsValues(recursive.id, 3, 255, 'bytes'), [[17,4335,2],[17,0,2],[17,17,2]]);
    for (const n of [0,1,37,2048]) {
        const expected = expectedNativeByteViewsObservation(recursive.id,n,7,'bytes');
        assert.deepEqual(expectedExactResult(recursive.id,n,7,'bytes'), expected);
        assert.equal(expectedChecksum(recursive.id,n,7,'bytes'), expected.checksum);
    }
});

test('Capturing callbacks retain argument views and oracle dispatch stays selective', () => {
    const callback = manifest.workloads.find(value => value.id === 'native_byte_views_callbacks');
    assert.ok(callback);
    assert.equal(reportCategory(callback.id), 'bytes');
    for (const [kind,file] of [['native',callback.source],['typescript',callback.counterpart]]) {
        const source=read('./'+file);
        assertNativeByteViewsSource(source,callback,kind);
        for (const from of ['return other;','const selected = picker.pick(window(owner, 1, 3));']) {
            const changed=replaceCodeSnippet(source,from,'');
            const key=kind==='native'?'sourceSha256':'counterpartSha256';
            assert.throws(()=>assertNativeByteViewsSource(changed,{...callback,[key]:createHash('sha256').update(changed).digest('hex')},kind));
        }
    }
    assert.deepEqual(expectedNativeByteViewsValues(callback.id,3,255,'bytes'),[[2,0],[2,1],[2,2]]);
    for (const n of [0,1,37,2048]) {
        const expected=expectedNativeByteViewsObservation(callback.id,n,7,'bytes');
        assert.deepEqual(expectedExactResult(callback.id,n,7,'bytes'),expected);
        assert.equal(expectedChecksum(callback.id,n,7,'bytes'),expected.checksum);
    }
    // An unrelated workload must not enter the native-view oracle.
    assert.doesNotThrow(()=>expectedChecksum('native_bytes',1,7,'bytes'));
});
