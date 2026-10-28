// Exercise the distributed CLI outside the source tree and remove every output.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
const [input] = process.argv.slice(2);
if (!input || process.argv.length !== 3) throw Error('Usage: node tools/smoke-toolchain.mjs CLI');
const compiler = path.resolve(input);
const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-release-smoke-'));
const run = (...args) => execFileSync(compiler, args, {
  cwd: directory, encoding: 'utf8', timeout: 180000,
  env: {...process.env, TSRUST_CACHE_DIR: path.join(directory, 'cache')},
});
try {
  assert.match(run('--version'), /^tsrust \d+\.\d+\.\d+/);
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
  console.log('Distributed CLI: scaffold, check, format, native run, tests and shared-cache JSON passed.');
} finally {
  assert.equal(path.dirname(path.resolve(directory)), path.resolve(os.tmpdir()));
  assert.ok(path.basename(directory).startsWith('tsrust-release-smoke-'));
  fs.rmSync(directory, {recursive: true, force: true});
}
