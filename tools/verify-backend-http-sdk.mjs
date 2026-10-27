// Exercise the actual CLI-created backend; no frontend toolchain or editor control.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import net from 'node:net';
import { spawn, spawnSync } from 'node:child_process';
import { pathToFileURL } from 'node:url';

const root = path.resolve(process.argv[2] || 'generated-projects/backend-http');
const compiler = path.resolve(process.env.TSRUST || 'target/debug/tsrust.exe');
const config = JSON.parse(fs.readFileSync(path.join(root, 'tsrust.config.json')));
const executable = path.resolve(root, config.build.output);
const base = 'http://127.0.0.1:4318';
// Never connect our test to an owner's already-running service.
await new Promise((resolve, reject) => {
  const probe = net.createServer();
  probe.once('error', reject);
  probe.listen(4318, '127.0.0.1', () => probe.close(resolve));
});
const child = spawn(executable, [], { cwd: root, windowsHide: true, stdio: ['ignore', 'pipe', 'pipe'] });
let output = '', errors = '';
child.stdout.on('data', value => { output += value; });
child.stderr.on('data', value => { errors += value; });
const exited = new Promise(resolve => { child.once('error', error => resolve({ error })); child.once('exit', (code, signal) => resolve({ code, signal })); });
const temporary = fs.mkdtempSync(path.resolve('target/backend-sdk-http-'));
try {
  const deadline = Date.now() + 15000;
  while (!output.includes('API ready')) {
    assert.equal(child.exitCode, null, `Backend exited: ${errors}`);
    assert.ok(Date.now() < deadline, `Backend readiness timeout: ${errors}`);
    await new Promise(resolve => setTimeout(resolve, 50));
  }
  const response = await fetch(`${base}/api/description`, { signal: AbortSignal.timeout(5000) });
  assert.equal(response.status, 200);
  assert.equal(response.headers.get('cache-control'), 'no-store');
  const description = await response.text();
  assert.deepEqual(JSON.parse(description), JSON.parse(fs.readFileSync(path.join(root, 'generated/client/http-api.json'))));
  const specification = await fetch(`${base}/api/openapi.json`, { signal: AbortSignal.timeout(5000) });
  assert.equal(specification.status, 200);
  assert.deepEqual(await specification.json(), JSON.parse(fs.readFileSync(path.join(root, 'generated/client/openapi.json'))));
  fs.writeFileSync(path.join(temporary, 'api.json'), description);
  const generated = spawnSync(compiler, ['api', 'generate', '--description', 'api.json', '--out', 'sdk'], { cwd: temporary, encoding: 'utf8', windowsHide: true, timeout: 30000 });
  assert.ifError(generated.error);
  assert.equal(generated.status, 0, generated.stderr);
  const { createClient } = await import(pathToFileURL(path.join(temporary, 'sdk/index.js')));
  const { createHttpTransport } = await import(pathToFileURL(path.join(temporary, 'sdk/http.js')));
  const api = createClient(createHttpTransport({ url: `${base}/api` }));
  assert.equal(await api.greet('  backend  '), 'Hello, backend!');
  assert.equal(await api.greet(''), 'Hello, world!');
  await assert.rejects(api.greet(42), error => error.code === 'api-validation');
  console.log('Fresh backend: live authorized metadata/OpenAPI, detached description generation, named HTTP calls and input validation passed.');
} finally {
  if (child.exitCode === null) child.kill();
  let timer;
  try { await Promise.race([exited, new Promise((_, reject) => { timer = setTimeout(() => reject(Error('Owned backend did not exit')), 5000); })]); }
  finally { clearTimeout(timer); }
  assert.equal(path.dirname(fs.realpathSync(temporary)), fs.realpathSync('target'));
  assert.ok(path.basename(temporary).startsWith('backend-sdk-http-'));
  fs.rmSync(temporary, { recursive: true, force: true });
}
