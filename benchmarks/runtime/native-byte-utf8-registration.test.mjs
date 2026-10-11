import vm from 'node:vm';
import ts from './node_modules/typescript/lib/typescript.js';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNativeByteUtf8Source} from './native-byte-utf8-source-contract.mjs';
import {expectedNativeByteUtf8Values, expectedNativeByteUtf8Observation} from './native-byte-utf8-oracle.mjs';
import {expectedExactResult, expectedChecksum} from './oracle.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const manifest = JSON.parse(read('./manifest.json'));
const row = manifest.workloads.find(value => value.id === 'native_byte_utf8');
test('Native byte UTF-8 retains Unicode round trips and strict validation', () => {
    assert.ok(row, 'Native byte UTF-8 workload registered');
    assert.equal(manifest.workloads.filter(value => value.id === row.id).length, 1);
    assert.equal(row.measured, true);
    assert.equal(row.measurementStatus, 'measured-resources-bytes-family');
    assert.equal(row.measurementEvidence, 'runtime/results/s07-measurement-proof-2026-10-11.json');
    assert.equal(reportCategory(row.id), 'bytes');
    const emitted = 'fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
    assert.equal(assertABI(emitted, row, read('./' + row.source)).fallible, false);
    for (const [kind, file] of [['native', row.source], ['typescript', row.counterpart]]) {
        const source = read('./' + file);
        assertNativeByteUtf8Source(source, row, kind);
        assert.throws(() => assertNativeByteUtf8Source(source + ' ', row, kind));
        assert.throws(() => assertNativeByteUtf8Source(source, {...row, operationsPerIteration: 2}, kind));
        const key = kind === 'native' ? 'sourceSha256' : 'counterpartSha256';
        for (const from of ['const text=textFor((i+seed)%4);', 'const bad=invalid([65,195,40]);', 'const truncated=invalid([65,226,130]);', 'return output;']) {
            const changed = replaceCodeSnippet(source, from, '');
            assert.throws(() => assertNativeByteUtf8Source(changed, {...row, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
        }
    }
    assert.deepEqual(expectedNativeByteUtf8Values(row.id, 3, 255, 'bytes'), [[4,1,4,1,4,1,4],[5,1,4,1,4,1,5],[6,1,4,1,4,1,6]]);
    for (const seed of [0,7,1000000]) for (const n of [0,1,37,2048]) {
        const expected = expectedNativeByteUtf8Observation(row.id, n, seed, 'bytes');
        assert.deepEqual(expectedExactResult(row.id, n, seed, 'bytes'), expected);
        assert.equal(expectedChecksum(row.id, n, seed, 'bytes'), expected.checksum);
    }
    assert.throws(() => expectedNativeByteUtf8Values(row.id, -1, 0, 'bytes'));
    assert.throws(() => expectedNativeByteUtf8Values(row.id, 1, 0, 'other'));
    for (const file of ['./run.mjs', './profile-native.mjs']) {
        const source = read(file);
        assert.ok(source.includes('assertNativeByteUtf8Source(source'));
        assert.ok(source.includes("'native-byte-utf8-oracle.mjs'"));
    }
});

test('UTF-8 counterpart preserves BOM/NUL and independently locates malformed sequences', () => {
    const source = read('./' + row.counterpart);
    const compiled = ts.transpileModule(source, {compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.CommonJS}}).outputText;
    const context = {exports:{},TextEncoder,TextDecoder};
    vm.runInNewContext(compiled + '\nglobalThis.utf8 = {fromUtf8,toUtf8,fromArray,firstInvalidOffset};', context);
    for (const values of [[-1],[256],[3,256,-1]]) {
        const result=context.utf8.fromArray(values);
        assert.equal(result.kind,'Err');
        assert.equal(result.error.code,'byte-range');
        assert.equal(result.error.offset,values[0]===3?1:0);
    }
    for (const text of ['', 'ASCII', 'é😀', 'A\0B', '\uFEFFz']) {
        const encoded = context.utf8.fromUtf8(text);
        assert.equal(encoded.kind, 'Ok');
        assert.equal(context.utf8.firstInvalidOffset(encoded.value), undefined);
        const decoded = context.utf8.toUtf8(encoded.value);
        assert.equal(decoded.kind, 'Ok');
        assert.equal(decoded.value, text);
    }
    for (const [bytes,offset] of [[[65,195,40],1],[[65,226,130],1],[[192,175],0],[[237,160,128],0],[[244,144,128,128],0],[[128],0]]) {
        const result = context.utf8.toUtf8(Uint8Array.from(bytes));
        assert.equal(result.kind, 'Err');
        assert.equal(result.error.code, 'utf8');
        assert.equal(result.error.offset, offset);
    }
});
