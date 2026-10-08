import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertTruthinessSource} from './truthiness-source-contract.mjs';
import {expectedTruthinessValues, expectedTruthinessObservation} from './truthiness-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
const read = path => fs.readFileSync(new URL(path, import.meta.url), 'utf8');
const manifest = JSON.parse(read('./manifest.json'));
const row = manifest.workloads.find(row => row.id === 'truthiness');
test('Truthiness registration preserves conditions, stores and independent observations', () => {
    assert.equal(manifest.workloads.filter(row => row.id === 'truthiness').length, 1);
    for (const [kind, path, key] of [['native', row.source, 'sourceSha256'], ['typescript', row.counterpart, 'counterpartSha256']]) {
        const source = read('./' + path);
        assertTruthinessSource(source, row, kind);
        assert.throws(() => assertTruthinessSource(source + ' ', row, kind));
        for (const [from, to] of [['selected ||= value', 'selected = value'], ['label &&= "member"', 'label = "member"'], ['if (optional)', 'if (true)'], ['while (remaining)', 'while (remaining !== 0)']]) {
            assert.ok(source.includes(from));
            const changed = source.replace(from, to);
            assert.throws(() => assertTruthinessSource(changed, {...row, [key]: createHash('sha256').update(changed).digest('hex')}, kind));
        }
    }
    assert.deepEqual(expectedTruthinessValues('truthiness', 2, 0, 'truthiness'), [[2, 6, 7, 1, 0, 1], [3, 6, 7, 2, 0, 1]]);
    for (const n of [0, 1, 17, 2048]) for (const seed of [0, 7, 1000000]) {
        assert.deepEqual(expectedExactResult('truthiness', n, seed, 'truthiness'), expectedTruthinessObservation('truthiness', n, seed, 'truthiness'));
    }
    for (const path of ['./run.mjs', './profile-native.mjs']) assert.ok(read(path).includes('assertTruthinessSource(source'));
});
