// Independent admission oracle. Not a registered or measured workload.
// Pin Node v24.11.1 when executing; Rust intentionally retains Rust OS semantics.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const script = fileURLToPath(import.meta.url);
const rows = readFileSync(new URL('./corpus.tsv', import.meta.url), 'utf8')
  .split('\n')
  .filter((line) => line && !line.startsWith('#'))
  .map((line) => line.split('\t'));

if (process.argv[2] === '--resolve-child') {
  const row = JSON.parse(process.argv[3]);
  // Isolated child only: injected context makes both lexical styles reproducible
  // on either host. No mutation of the harness process's cwd or environment.
  process.cwd = () => row[2];
  process.env = {
    ...process.env,
    '=C:': row[3],
    '=D:': row[3],
  };
  process.stdout.write(path[row[1]].resolve(...row.slice(5)));
  process.exit(0);
}

assert.equal(process.version, 'v24.11.1', 'use the pinned Node reference');
for (const row of rows) {
  const api = path[row[1]];
  if (row[0] === 'parse') {
    const actual = api.parse(row[2]);
    assert.deepEqual(
      ['root', 'dir', 'base', 'ext', 'name'].map((key) => actual[key]),
      row.slice(3),
      JSON.stringify(row),
    );
  } else if (row[0] === 'format') {
    const keys = ['root', 'dir', 'base', 'ext', 'name'];
    const explicit = Object.fromEntries(keys.map((key, i) => [key, row[i + 3]]));
    const omitted = Object.fromEntries(Object.entries(explicit).filter(([, value]) => value));
    assert.equal(api.format(omitted), row[2], JSON.stringify(row));
    assert.equal(api.format(explicit), row[2], 'empty fields use the same precedence');
  } else if (row[0] === 'resolve') {
    const child = spawnSync(process.execPath, [script, '--resolve-child', JSON.stringify(row)], {
      encoding: 'utf8',
      timeout: 10_000,
    });
    assert.equal(child.error, undefined);
    assert.equal(child.status, 0, child.stderr);
    assert.equal(child.stdout, row[4], JSON.stringify(row));
  } else {
    assert.fail(`unknown operation ${row[0]}`);
  }
}

// The Rust std::path baseline deliberately covers only these common POSIX
// cases. The entire corpus above remains the independent correctness oracle.
const common = ['/srv/releases/file.tar.gz', 'reports/é😊.txt', 'alpha/.gitignore', 'alpha/file.'];
const formatCases = rows.filter((row) => row[0] === 'format').map((row) => ({
  api: path[row[1]],
  parts: Object.fromEntries(['root', 'dir', 'base', 'ext', 'name'].map((key, i) => [key, row[i + 3]]).filter(([, value]) => value)),
}));
const resolveCases = rows.filter((row) => row[0] === 'resolve' && row[1] === 'posix');
if (process.argv[2] === '--profile') {
  const n = Number(process.argv[3] ?? 256);
  assert.ok(Number.isSafeInteger(n) && n > 0, 'iterations must be a positive safe integer');
  let retained;
  function profile(name, operation) {
    for (let warmup = 0; warmup < 8; warmup++) retained = operation();
    for (let repetition = 0; repetition < 3; repetition++) {
      const samples = [];
      for (let sample = 0; sample < 31; sample++) {
        const start = process.hrtime.bigint();
        retained = operation();
        const elapsed = process.hrtime.bigint() - start;
        assert.equal(retained.length, n);
        samples.push(elapsed);
      }
      samples.sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
      console.log(`${name}\t${n}\t${repetition}\t${samples[15]}`);
    }
  }
  profile('node-owned-path-parts', () => Array.from({ length: n }, (_, i) => path.posix.parse(common[i % common.length])));
  profile('node-format-parts', () => Array.from({ length: n }, (_, i) => {
    const { api, parts } = formatCases[i % formatCases.length];
    return api.format(parts);
  }));
  profile('node-posix-resolve-fixed-context', () => Array.from({ length: n }, (_, i) => {
    const row = resolveCases[i % resolveCases.length];
    return path.posix.resolve(row[2], ...row.slice(5));
  }));
  // Keep outputs observable and alive through each sample. The CLI watchdog,
  // rather than a JS heap-only figure, must bound the complete process tree.
  assert.ok(retained.every((value) => typeof value === 'string'));
}
console.log(`B03 independent Node oracle checked ${rows.length} cases`);
