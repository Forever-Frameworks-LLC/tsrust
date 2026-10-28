// Real LSP diagnostics for a CLI-generated project; never controls the owner's editor.
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawn } from 'node:child_process';
import { pathToFileURL } from 'node:url';

const target = path.resolve(process.argv[2] || 'target/application-preview');
const standalone = fs.statSync(target).isFile();
const root = standalone ? path.dirname(target) : target;
const executable = path.resolve(process.env.TSRUST_LSP_SERVER || 'target/debug/tsrust-lsp.exe');
const diagnosticTimeoutMs = Number(process.env.TSRUST_EDITOR_DIAGNOSTIC_TIMEOUT_MS || 15000);
assert.ok(Number.isInteger(diagnosticTimeoutMs) && diagnosticTimeoutMs >= 1000 && diagnosticTimeoutMs <= 300000);
const projectConfig = standalone ? {build: {entry: path.basename(target)}} : JSON.parse(fs.readFileSync(path.join(root, 'tsrust.config.json'), 'utf8'));
const standaloneApi = path.join(root, 'tsrust.api.json');
const apiConfig = standalone ? null : projectConfig.api || (fs.existsSync(standaloneApi) ? JSON.parse(fs.readFileSync(standaloneApi, 'utf8')) : null);
const child = spawn(executable, [], { windowsHide: true, stdio: ['pipe', 'pipe', 'pipe'] });
let pending = Buffer.alloc(0), sequence = 0, errors = '';
const requests = new Map(), diagnostics = new Map();
const exited = new Promise((resolve, reject) => { child.once('error', reject); child.once('close', resolve); });
const timeout = setTimeout(() => child.kill(), Math.max(60000, diagnosticTimeoutMs * 4));
child.stderr.on('data', chunk => { errors += chunk; if (process.env.TSRUST_LSP_TRACE_TIMINGS) process.stderr.write(chunk); });
child.stdout.on('data', chunk => {
  pending = Buffer.concat([pending, chunk]);
  while (true) {
    const boundary = pending.indexOf('\r\n\r\n'); if (boundary < 0) return;
    const length = Number(/Content-Length: (\d+)/i.exec(pending.subarray(0, boundary).toString())?.[1]);
    assert.ok(length > 0 && length < 4 * 1024 * 1024);
    if (pending.length < boundary + 4 + length) return;
    const value = JSON.parse(pending.subarray(boundary + 4, boundary + 4 + length)); pending = pending.subarray(boundary + 4 + length);
    if (value.method === 'textDocument/publishDiagnostics') diagnostics.set(value.params.uri, value.params);
    if (value.id !== undefined) { requests.get(value.id)?.(value); requests.delete(value.id); }
  }
});
function send(method, params, id) {
  const body = Buffer.from(JSON.stringify({ jsonrpc: '2.0', method, params, ...(id === undefined ? {} : { id }) }));
  child.stdin.write(`Content-Length: ${body.length}\r\n\r\n`); child.stdin.write(body);
}
async function request(method, params) {
  const id = ++sequence;
  const response = new Promise((resolve, reject) => {
    const timer = setTimeout(() => { requests.delete(id); reject(new Error(`LSP request timed out: ${method}`)); }, 15000);
    requests.set(id, value => { clearTimeout(timer); value.error ? reject(new Error(JSON.stringify(value.error))) : resolve(value.result); });
  });
  send(method, params, id); return response;
}
async function result(uri, version) {
  const deadline = Date.now() + diagnosticTimeoutMs;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`LSP exited: ${errors}`);
    const value = diagnostics.get(uri); if (value && value.version === version) return value.diagnostics;
    await new Promise(resolve => setTimeout(resolve, 100));
  }
  const observed = [...diagnostics].map(([uri, value]) => ({ uri, version: value.version, count: value.diagnostics.length }));
  const counters = await request('tsrust/counters', {});
  throw new Error(`No LSP diagnostics for version ${version}: ${JSON.stringify({ observed, counters, stderr: errors })}`);
}
try {
  await request('initialize', { rootUri: pathToFileURL(root).href, capabilities: {}, initializationOptions: { debounceMs: 0 } });
  send('initialized', {});
  for (const relative of [...new Set([projectConfig.build.entry, ...(apiConfig ? [apiConfig.entry] : [])])]) {
    const file = path.join(root, relative), uri = pathToFileURL(file).href, source = fs.readFileSync(file, 'utf8');
    send('textDocument/didOpen', { textDocument: { uri, languageId: 'tsrust', version: 1, text: source } });
    assert.deepEqual(await result(uri, 1), [], `${relative} has editor diagnostics`);
    if (relative === (apiConfig?.entry || projectConfig.build.entry)) {
      const facade = standalone && source.includes("from 'std:ui-runtime'");
      const invalid = source + '\nfunction _tsrustEditorDiagnosticProbe(): int { return "invalid"; }\n';
      send('textDocument/didChange', { textDocument: { uri, version: 2 }, contentChanges: [{ text: invalid }] });
      const rejected = await result(uri, 2);
      assert.ok(rejected.some(value => value.severity === 1), 'LSP must report the changed source');
      if (facade) assert.ok(rejected.some(value => value.code === 'module-native'), 'Modified facades must retain the private-runtime guard');
      send('textDocument/didChange', { textDocument: { uri, version: 3 }, contentChanges: [{ text: source }] });
      assert.deepEqual(await result(uri, 3), [], 'Editor diagnostics must clear after repair');
      assert.equal(fs.readFileSync(file, 'utf8'), source, 'Protocol edits must not change owner files');
      if (!standalone && relative === projectConfig.build.entry && source.includes("'api:server'")) {
        const offset = source.indexOf('dispatch');
        assert.ok(offset >= 0, 'The backend scaffold imports its checked dispatcher');
        const prefix = source.slice(0, offset);
        const position = {line: prefix.split('\n').length - 1, character: prefix.split('\n').at(-1).length};
        const hover = await request('textDocument/hover', {textDocument: {uri}, position});
        assert.ok(hover?.contents, 'In-memory API handlers must retain checked cursor assistance');
      }
      if (facade) {
        const offset = source.indexOf('export type Node') + 'export type '.length;
        const prefix = source.slice(0, offset);
        const position = {line: prefix.split('\n').length - 1, character: prefix.split('\n').at(-1).length};
        const hover = await request('textDocument/hover', {textDocument: {uri}, position});
        assert.ok(hover?.contents, 'The physical facade document must retain checked cursor assistance');
      }
    }
    send('textDocument/didClose', { textDocument: { uri } });
  }
  await request('shutdown', null); send('exit');
  assert.equal(await exited, 0, errors);
  console.log('Editor: real LSP diagnostics clean; changed source rejected and cleared after repair; disk source preserved.');
} finally {
  clearTimeout(timeout);
  if (child.exitCode === null) { child.kill(); await exited; }
}
