import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import crypto from 'node:crypto';
import {execFileSync, spawnSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const packager = path.join(root, 'tools/package-toolchain.mjs');
const preflight = path.join(root, 'tools/check-release.mjs');
const digest = bytes => crypto.createHash('sha256').update(bytes).digest('hex');
function temporary(body) {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-distribution-test-'));
  try { return body(directory); }
  finally { fs.rmSync(directory, {recursive: true, force: true}); }
}
function consumers(directory) {
  const cli = path.join(directory, 'compiler');
  const lsp = path.join(directory, 'server');
  fs.writeFileSync(cli, 'CLI fixture\n');
  fs.writeFileSync(lsp, 'LSP fixture\n');
  return [cli, lsp];
}
function pack(args) {
  return spawnSync(process.execPath, [packager, ...args], {cwd: root, encoding: 'utf8'});
}

test('packaging preserves bytes, source versions, notices and executable permissions', () => temporary(directory => {
  const inputs = consumers(directory);
  const vendor = path.join(directory, 'vendor');
  const crate = path.join(vendor, 'example-1.0.0');
  fs.mkdirSync(path.join(crate, 'licenses'), {recursive: true});
  fs.writeFileSync(path.join(crate, 'Cargo.toml'), '[package]\nname="example"\nversion="1.0.0"\nlicense="MIT"\n');
  fs.writeFileSync(path.join(crate, '.cargo-checksum.json'), '{}');
  fs.writeFileSync(path.join(crate, 'licenses', 'MIT.txt'), 'dependency license fixture\n');
  fs.writeFileSync(path.join(crate, 'lib.rs'), 'excluded implementation');
  const output = path.join(directory, 'package');
  const result = pack([...inputs, 'aarch64-apple-darwin', output, vendor]);
  assert.equal(result.status, 0, result.stderr);
  const manifest = JSON.parse(fs.readFileSync(path.join(output, 'manifest.json')));
  assert.equal(manifest.distribution, 'release');
  assert.match(manifest.sourceCommit, /^[a-f0-9]{40}$/);
  assert.equal(manifest.version, fs.readFileSync(path.join(root, 'crates/tsrust-compiler/Cargo.toml'), 'utf8').match(/^version = "([^"]+)"/m)[1]);
  for (const [relative, record] of Object.entries(manifest.files)) {
    const bytes = fs.readFileSync(path.join(output, relative));
    assert.equal(record.bytes, bytes.length);
    assert.equal(record.sha256, digest(bytes));
  }
  assert.deepEqual(fs.readFileSync(path.join(output, 'bin/tsrust')), fs.readFileSync(inputs[0]));
  assert.deepEqual(fs.readFileSync(path.join(output, 'bin/tsrust-lsp')), fs.readFileSync(inputs[1]));
  if (process.platform !== 'win32') assert.ok(fs.statSync(path.join(output, 'bin/tsrust')).mode & 0o111);
  assert.equal(fs.readFileSync(path.join(output, 'LICENSE'), 'utf8'), fs.readFileSync(path.join(root, 'LICENSE'), 'utf8'));
  assert.equal(fs.readFileSync(path.join(output, 'attributions/dependencies/example-1.0.0/licenses/MIT.txt'), 'utf8'), 'dependency license fixture\n');
  assert.ok(!fs.existsSync(path.join(output, 'attributions/dependencies/example-1.0.0/lib.rs')));
}));

test('packaging never replaces existing destinations and removes incomplete output', () => temporary(directory => {
  const inputs = consumers(directory);
  const output = path.join(directory, 'package');
  fs.mkdirSync(output);
  fs.writeFileSync(path.join(output, 'owner'), 'keep');
  assert.notEqual(pack([...inputs, 'aarch64-apple-darwin', output]).status, 0);
  assert.equal(fs.readFileSync(path.join(output, 'owner'), 'utf8'), 'keep');
  const broken = path.join(directory, 'broken');
  assert.notEqual(pack([inputs[0], path.join(directory, 'missing'), 'aarch64-apple-darwin', broken]).status, 0);
  assert.ok(!fs.existsSync(broken));
  assert.notEqual(pack([...inputs, '../invalid', path.join(directory, 'bad-target')]).status, 0);
  assert.ok(!fs.existsSync(path.join(directory, 'bad-target')));
  const vendor = path.join(directory, 'empty-vendor');
  fs.mkdirSync(vendor);
  assert.notEqual(pack([...inputs, 'aarch64-apple-darwin', broken, vendor]).status, 0);
  assert.ok(!fs.existsSync(broken));
}));

test('development packages are marked as incomplete for redistribution', () => temporary(directory => {
  const output = path.join(directory, 'package');
  const result = pack([...consumers(directory), 'x86_64-unknown-linux-gnu', output]);
  assert.equal(result.status, 0, result.stderr);
  assert.equal(JSON.parse(fs.readFileSync(path.join(output, 'manifest.json'))).distribution, 'development');
  assert.match(fs.readFileSync(path.join(output, 'README.md'), 'utf8'), /not a redistribution artifact/);
}));

test('Windows packages keep executable suffixes and exact manifest identities', () => temporary(directory => {
  const inputs = consumers(directory);
  const output = path.join(directory, 'Windows package');
  const result = pack([...inputs, 'x86_64-pc-windows-msvc', output]);
  assert.equal(result.status, 0, result.stderr);
  const manifest = JSON.parse(fs.readFileSync(path.join(output, 'manifest.json')));
  for (const [index, name] of ['tsrust.exe', 'tsrust-lsp.exe'].entries()) {
    const bytes = fs.readFileSync(path.join(output, 'bin', name));
    assert.deepEqual(bytes, fs.readFileSync(inputs[index]));
    assert.deepEqual(manifest.files[`bin/${name}`], {sha256: digest(bytes), bytes: bytes.length});
    assert.ok(!fs.existsSync(path.join(output, 'bin', name.slice(0, -4))));
  }
}));

test('release preflight accepts exact versions and rejects mismatches and dirty sources', () => temporary(directory => {
  const compiler = path.join(directory, 'crates/tsrust-compiler');
  fs.mkdirSync(path.join(compiler, 'native-dependencies'), {recursive: true});
  fs.writeFileSync(path.join(compiler, 'Cargo.toml'), '[package]\nversion = "0.1.0"\n');
  fs.writeFileSync(path.join(compiler, 'native-dependencies/Cargo.lock'), '# fixture\n');
  fs.writeFileSync(path.join(directory, 'Cargo.lock'), '# fixture\n');
  fs.writeFileSync(path.join(directory, 'LICENSE'), 'MIT License\n');
  const git = (...args) => execFileSync('git', ['-C', directory, ...args], {stdio: 'pipe'});
  git('init', '--quiet');
  git('add', '.');
  git('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '--quiet', '-m', 'fixture');
  const check = tag => spawnSync(process.execPath, [preflight, tag], {cwd: directory, encoding: 'utf8'});
  assert.equal(check('v0.1.0').status, 0);
  for (const tag of ['v0.2.0', 'v0.1.0-rc.1', 'v01.1.0', 'v0.1.0-alpha.01', 'v0.1.0;echo bad', '0.1.0']) assert.notEqual(check(tag).status, 0, tag);
  fs.writeFileSync(path.join(directory, 'untracked'), 'dirty');
  assert.notEqual(check('v0.1.0').status, 0);
  git('add', 'untracked');
  assert.notEqual(check('v0.1.0').status, 0);
  git('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '--quiet', '-m', 'fixture');
  fs.writeFileSync(path.join(compiler, 'Cargo.toml'), '[package]\nversion = "0.1.0-rc.1"\n');
  git('add', '.');
  git('-c', 'user.name=Test', '-c', 'user.email=test@example.invalid', 'commit', '--quiet', '-m', 'prerelease');
  assert.equal(check('v0.1.0-rc.1').status, 0);
}));

test('public entry-point documentation links resolve within the checkout', () => {
  for (const relative of ['README.md', 'CONTRIBUTING.md', 'SECURITY.md', 'examples/README.md', 'docs/README.md', 'docs/content/getting-started/installation.md', 'docs/guides/releases.md', 'editors/vscode/README.md']) {
    const filename = path.join(root, relative);
    const text = fs.readFileSync(filename, 'utf8').replace(/```[\s\S]*?```/g, '');
    for (const [, link] of text.matchAll(/\[[^\]]*\]\(([^)]+)\)/g)) {
      if (/^(?:https?:|mailto:|#)/.test(link)) continue;
      const target = path.resolve(path.dirname(filename), decodeURIComponent(link.split('#')[0]));
      assert.ok(fs.existsSync(target), `${relative}: broken link ${link}`);
    }
  }
});
