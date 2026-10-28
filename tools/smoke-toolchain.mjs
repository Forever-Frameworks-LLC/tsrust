// Exercise the distributed CLI outside the source tree and remove every output.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {execFileSync, spawnSync} from 'node:child_process';
import {pathToFileURL, fileURLToPath} from 'node:url';
const [input, editor] = process.argv.slice(2);
if (!input || process.argv.length > 4) throw Error('Usage: node tools/smoke-toolchain.mjs CLI [LSP]');
const compiler = path.resolve(input);
const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-release-smoke-'));
const run = (...args) => execFileSync(compiler, args, {
  cwd: directory, encoding: 'utf8', timeout: 180000,
  env: {...process.env, TSRUST_CACHE_DIR: path.join(directory, 'cache')},
});
try {
  if (editor) {
    const messages = [
      {jsonrpc: '2.0', id: 1, method: 'initialize', params: {processId: null, rootUri: pathToFileURL(directory).href, capabilities: {}}},
      {jsonrpc: '2.0', method: 'initialized', params: {}},
      {jsonrpc: '2.0', id: 2, method: 'shutdown', params: null},
      {jsonrpc: '2.0', method: 'exit', params: null},
    ].map(message => {
      const body = JSON.stringify(message);
      return `Content-Length: ${Buffer.byteLength(body)}\r\n\r\n${body}`;
    }).join('');
    const output = execFileSync(path.resolve(editor), [], {cwd: directory, input: messages, encoding: 'utf8', timeout: 30000});
    const responses = output.split(/Content-Length: \d+\r\n\r\n/).slice(1).map(body => JSON.parse(body));
    assert.ok(responses.find(message => message.id === 1)?.result?.capabilities?.hoverProvider, output);
    assert.deepEqual(responses.find(message => message.id === 2), {jsonrpc: '2.0', id: 2, result: null});
    console.log('Distributed LSP: initialize, capabilities and clean shutdown passed.');
  }
  assert.match(run('--version'), /^tsrust \d+\.\d+\.\d+/);
  run('setup', '--dependencies', '--offline');
  const readiness = JSON.parse(run('doctor', '--native', '--json'));
  assert.equal(readiness.toolchainReady, true, JSON.stringify(readiness));
  const created = run('new', 'hello');
  const agentGuide = fs.readFileSync(path.join(directory, 'hello/AGENTS.md'), 'utf8');
  for (const command of ['tsrust check', 'tsrust test', 'tsrust format', 'tsrust dev']) assert.ok(agentGuide.includes(command));
  assert.ok(agentGuide.includes('ui.components') && agentGuide.includes('AGENTS.md'));
  const startCommand = 'tsrust dev';
  assert.ok(created.includes(`Run \`${startCommand}\``), created);
  assert.ok(fs.readFileSync(path.join(directory, 'hello/README.md'), 'utf8').includes(`Run \`${startCommand}\``));
  const config = JSON.parse(fs.readFileSync(path.join(directory, 'hello/tsrust.config.json')));
  assert.equal(config.build.output, `target/hello${process.platform === 'win32' ? '.exe' : ''}`);
  run('check', 'hello');
  run('format', 'hello', '--check');
  assert.equal(run('run', 'hello/src/main.tsr'), 'Hello from hello!\n');
  run('test', 'hello/tests');
  fs.writeFileSync(path.join(directory, 'json.tsr'), "import {parse,kind} from 'std:json';function main(){match(parse('null')){Ok(value)=>{console.log(kind(value));},Err(error)=>{throw error.message;}}}\n");
  run('format', 'json.tsr');
  assert.equal(run('run', 'json.tsr'), 'null\n');
  const docs = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../website/src/content/docs');
  const quickstart = fs.readFileSync(path.join(docs, 'start/quickstart.md'), 'utf8').match(/```typescript\n([\s\S]*?)```/)?.[1];
  assert.ok(quickstart, 'Quickstart program must exist');
  fs.writeFileSync(path.join(directory, 'quickstart.tsr'), quickstart);
  run('check', 'quickstart.tsr');
  run('format', 'quickstart.tsr', '--write');
  run('format', 'quickstart.tsr', '--check');
  assert.equal(run('run', 'quickstart.tsr'), 'TypeScript + Rust\n');

  const testing = fs.readFileSync(path.join(docs, 'tools/testing.md'), 'utf8');
  const greeting = testing.match(/```tsrust\n([\s\S]*?)```/)?.[1];
  assert.ok(greeting, 'Testing guide program must exist');
  const testDirectory = path.join(directory, 'documented-tests');
  fs.mkdirSync(testDirectory);
  const testFile = path.join(testDirectory, 'greeting.test.tsr');
  fs.writeFileSync(testFile, greeting);
  run('format', testFile, '--write');
  run('format', testFile, '--check');
  const windows = process.platform === 'win32';
  const shellCode = testing.match(windows ? /```powershell\n([\s\S]*?)```/ : /```sh\n(# macOS\/Linux shell[\s\S]*?)```/)?.[1];
  assert.ok(shellCode, 'Platform reporter command must exist');
  const options = {cwd: testDirectory, encoding: 'utf8', timeout: 180000,
    env: {...process.env, PATH: path.dirname(compiler) + path.delimiter + (process.env.PATH || ''),
      TSRUST_CACHE_DIR: path.join(directory, 'cache')}};
  const report = execFileSync(windows ? 'powershell.exe' : '/bin/sh',
    windows ? ['-NoProfile', '-NonInteractive', '-Command', shellCode] : ['-c', shellCode], options);
  assert.match(report, /joins a greeting/);
  assert.match(report, /1 passed/);
  fs.writeFileSync(testFile, greeting.replace("'Hello Ada'", "'Hello Bob'"));
  const rejected = spawnSync(compiler, ['test', testFile, '--color', 'never'], options);
  assert.equal(rejected.status, 1, rejected.stderr);
  assert.match(rejected.stdout + rejected.stderr, /Hello Ada/);
  assert.match(rejected.stdout + rejected.stderr, /Hello Bob/);
  console.log('Distributed CLI: scaffold, native run, shared-cache JSON, exact quickstart/testing examples and platform reporter commands passed.');
} finally {
  assert.equal(path.dirname(path.resolve(directory)), path.resolve(os.tmpdir()));
  assert.ok(path.basename(directory).startsWith('tsrust-release-smoke-'));
  fs.rmSync(directory, {recursive: true, force: true});
}
