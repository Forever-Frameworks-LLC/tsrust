import assert from 'node:assert/strict';
import fs from 'node:fs';
import crypto from 'node:crypto';
const read = name => fs.readFileSync(new URL(name, import.meta.url), 'utf8');
const manifest = JSON.parse(read('static-manifest.json'));
assert.equal(manifest.expected, 128 * 42);
assert.equal(manifest.fixture, '42');
assert.deepEqual(manifest.protocol, { processes: 3, warmups: 8, samples: 31, iterations: 128 });
for (const extension of ['tsr', 'ts']) {
  const source = read(`static-file.${extension}`);
  assert.equal(crypto.createHash('sha256').update(source).digest('hex'), manifest.sources[extension]);
  for (const pattern of [/index < 128/, /warmup < 8/, /sample < 31/, /sum \+= 42/, /TSRUST_STATIC_BENCH_ROOT/]) assert.match(source, pattern);
}
assert.match(read('static-file.tsr'), /serveStatic\('\/assets', root\)/);
assert.match(read('static-file.ts'), /createReadStream/);
assert.match(read('run.mjs'), /static-file-v1/);
console.log('Static-file paired sources, protocol, oracle and harness registration passed.');
