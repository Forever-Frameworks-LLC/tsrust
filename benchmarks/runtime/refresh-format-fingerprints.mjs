import fs from 'node:fs';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';
import {fileURLToPath, pathToFileURL} from 'node:url';
import {codeTokens} from './source-contract-tokens.mjs';

const digest = source => createHash('sha256').update(source).digest('hex');
const spelling = source => JSON.stringify(codeTokens(source, true).map(token => token.text));

// Rehash only proven code-identical sources. Changed semantics need ordinary
// feature acceptance, not this automatic formatting path.
export function refreshFingerprintText(text, readCurrent, readBaseline, ids = []) {
    const manifest = JSON.parse(text);
    for (const id of ids) {
        if (!manifest.workloads.some(row => row.id === id)) throw new Error('Unknown workload: ' + id);
    }
    let updated = text, count = 0;
    for (const row of manifest.workloads) {
        if (ids.length && !ids.includes(row.id)) continue;
        const changes = [];
        for (const [key, filename] of [['sourceSha256', row.source ?? 'workloads/' + row.id + '.tsr'], ['counterpartSha256', row.counterpart ?? 'workloads/' + row.id + '.ts']]) {
            if (!row[key]) continue;
            const source = readCurrent(filename), hash = digest(source);
            if (hash === row[key]) continue;
            const baseline = readBaseline(filename, row[key]);
            if (digest(baseline) !== row[key]) throw new Error('Committed fingerprint does not match baseline: ' + filename);
            if (spelling(source) !== spelling(baseline)) throw new Error('Code tokens changed; review required: ' + filename);
            changes.push([key, row[key], hash]);
        }
        if (!changes.length) continue;
        const marker = '"id": ' + JSON.stringify(row.id);
        const start = updated.indexOf(marker);
        const end = updated.indexOf('\n    }', start);
        if (start < 0 || end < 0) throw new Error('Manifest row layout changed: ' + row.id);
        let block = updated.slice(start, end);
        for (const [key, previous, next] of changes) {
            const before = JSON.stringify(key) + ': ' + JSON.stringify(previous);
            if (!block.includes(before)) throw new Error('Missing fingerprint field: ' + row.id + '/' + key);
            block = block.replace(before, JSON.stringify(key) + ': ' + JSON.stringify(next));
        }
        updated = updated.slice(0, start) + block + updated.slice(end);
        count += changes.length;
    }
    return {text: updated, count};
}

export function capturePinnedSources() {
    const here = path.dirname(fileURLToPath(import.meta.url));
    const rows = JSON.parse(fs.readFileSync(path.join(here, 'manifest.json'), 'utf8')).workloads;
    const sources = new Map();
    for (const row of rows) {
        for (const [key, filename] of [['sourceSha256', row.source ?? 'workloads/' + row.id + '.tsr'], ['counterpartSha256', row.counterpart ?? 'workloads/' + row.id + '.ts']]) {
            if (row[key] && !sources.has(filename)) sources.set(filename, fs.readFileSync(path.join(here, filename), 'utf8'));
        }
    }
    return sources;
}

export function refreshProjectFingerprints(ids = [], priorSources = new Map()) {
    const here = path.dirname(fileURLToPath(import.meta.url));
    const root = path.resolve(here, '../..');
    const manifestPath = path.join(here, 'manifest.json');
    const cache = new Map();
    const result = refreshFingerprintText(fs.readFileSync(manifestPath, 'utf8'),
        filename => fs.readFileSync(path.join(here, filename), 'utf8'),
        (filename, expectedHash) => {
            if (priorSources.has(filename) && digest(priorSources.get(filename)) === expectedHash) return priorSources.get(filename);
            if (!cache.has(filename)) cache.set(filename, execFileSync('git', ['show', 'HEAD:benchmarks/runtime/' + filename], {cwd: root, encoding: 'utf8', maxBuffer: 16 * 1024 * 1024}));
            return cache.get(filename);
        }, ids);
    if (result.count) fs.writeFileSync(manifestPath, result.text);
    return result.count;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
    const options = process.argv.slice(2);
    if (options.some(option => !option.startsWith('--only=')) || options.length > 1) throw new Error('Usage: refresh-format-fingerprints.mjs [--only=id,id]');
    const ids = options.length ? options[0].slice('--only='.length).split(',') : [];
    console.log('Updated ' + refreshProjectFingerprints(ids) + ' formatting fingerprints.');
}
