import fs from 'node:fs';
import crypto from 'node:crypto';
import assert from 'node:assert/strict';
import test from 'node:test';
test('request readers retain matched source identities and independent byte oracle', () => {
  const manifest = JSON.parse(fs.readFileSync(new URL('./readers-manifest.json', import.meta.url)));
  assert.equal(manifest.expected, 128 * (40 + Buffer.byteLength('é')));
  assert.deepEqual(manifest.protocol, {processes: 3, warmups: 8, samples: 31, iterations: 128, requestsPerIteration: 2});
  for (const extension of ['tsr', 'ts']) {
    const source = fs.readFileSync(new URL(`./request-readers.${extension}`, import.meta.url));
    assert.equal(crypto.createHash('sha256').update(source).digest('hex'), manifest.sources[extension]);
    for (const witness of ['index < 128', 'warmup < 8', 'sample < 31', 'JSON mismatch', 'Stream mismatch', 'JSON response mismatch', 'Stream response mismatch']) assert.ok(source.toString().includes(witness), witness);
  }
});
