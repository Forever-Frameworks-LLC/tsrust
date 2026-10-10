import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertResultMatchSource} from './result-match-source-contract.mjs';
import {expectedResultMatchValues, expectedResultMatchObservation} from './result-match-oracle.mjs';
import {expectedExactResult, expectedChecksum} from './oracle.mjs';
import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';

const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const manifest = JSON.parse(read('./manifest.json'));
const row = manifest.workloads.find(value => value.id === 'result_match');

test('Result match workloads retain independent observations, source guards and execution hooks', () => {
    assert.ok(row, 'Result match workload registered');
    assert.equal(manifest.workloads.filter(value => value.id === row.id).length, 1);
    assert.equal(row.measured, false);
    assert.equal(row.measurementStatus, 'pending-result-match-family');
    assert.equal(reportCategory(row.id), 'callbacks');
    const emitted = 'fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
    assert.equal(assertABI(emitted, row, read('./' + row.source)).fallible, false);
    for (const [kind, file] of [['native', row.source], ['typescript', row.counterpart]]) {
        const source = read('./' + file);
        assertResultMatchSource(source, row, kind);
        assert.throws(() => assertResultMatchSource(source + ' ', row, kind));
        assert.throws(() => assertResultMatchSource(source, {...row, operationsPerIteration: 2}, kind));
        const key = kind === 'native' ? 'sourceSha256' : 'counterpartSha256';
        for (const from of ['const first = inspect(result);', 'const second = inspect(result);', 'update(editable);', 'output.push([first, second, owned, callA, callB, mutated]);']) {
            const changed = replaceCodeSnippet(source, from, '');
            assert.throws(() => assertResultMatchSource(changed, {...row, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
        }
    }
    assert.deepEqual(expectedResultMatchValues(row.id, 3, 0, 'results'), [[2, 2, 8, 2, 3, 10], [4, 4, 9, 3, 4, 6], [24, 24, 19, 4, 5, 10]]);
    for (const seed of [0, 7, 1000000]) for (const n of [0, 1, 37, 2048]) {
        const expected = expectedResultMatchObservation(row.id, n, seed, 'results');
        assert.deepEqual(expectedExactResult(row.id, n, seed, 'results'), expected);
        assert.equal(expectedChecksum(row.id, n, seed, 'results'), expected.checksum);
    }
    for (const file of ['./run.mjs', './profile-native.mjs']) {
        const source = read(file);
        assert.ok(source.includes('assertResultMatchSource(source'));
        assert.ok(source.includes("'result-match-oracle.mjs'"));
    }
});
