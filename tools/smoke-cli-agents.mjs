// Exercise generated project/component guidance through the real CLI transaction.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {execFileSync} from 'node:child_process';

if (process.argv.length !== 3) throw Error('Usage: node tools/smoke-cli-agents.mjs CLI');
const compiler = path.resolve(process.argv[2]);
const temporary = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-agents-smoke-'));
const application = path.join(temporary, 'app');
const run = (cwd, ...args) => execFileSync(compiler, args, {
  cwd, encoding: 'utf8', timeout: 120000,
  env: {...process.env, CARGO_INCREMENTAL: '0', CARGO_BUILD_JOBS: '1', TSRUST_CACHE_DIR: path.join(temporary, 'cache')},
});
const fingerprint = directory => {
  const files = {};
  const walk = current => {
    for (const entry of fs.readdirSync(current, {withFileTypes: true})) {
      const file = path.join(current, entry.name);
      if (entry.isDirectory()) walk(file);
      else files[path.relative(directory, file)] = createHash('sha256').update(fs.readFileSync(file)).digest('hex');
    }
  };
  walk(directory);
  return files;
};
try {
  run(temporary, 'new', 'app', '--ui', '--target', 'native');
  const rootGuide = path.join(application, 'AGENTS.md');
  assert.ok(fs.readFileSync(rootGuide, 'utf8').includes('ui.components'));
  const components = path.join(application, 'src/components/ui');
  const index = path.join(components, 'AGENTS.md');
  const buttonGuide = path.join(components, 'button-3.0.0-default.AGENTS.md');
  const guide = fs.readFileSync(buttonGuide, 'utf8');
  for (const witness of ['function Button', 'onPress', 'variant', 'secondary']) assert.ok(guide.includes(witness), witness);
  assert.ok(fs.readFileSync(index, 'utf8').includes('button-3.0.0-default.AGENTS.md'));
  assert.ok(!guide.includes('const variant ='), 'guidance must not snapshot component bodies');

  fs.appendFileSync(rootGuide, '\nOwner project advice.\n');
  fs.writeFileSync(buttonGuide, 'Owner component advice.\n');
  const source = path.join(components, 'button.tsrx');
  fs.writeFileSync(source, '// Owner customization\n' + fs.readFileSync(source, 'utf8'));
  const before = fingerprint(application);
  run(application, 'ui', 'add', 'button', '--offline');
  assert.deepEqual(fingerprint(application), before, 're-adding must preserve owner files and lock bytes');

  run(application, 'ui', 'add', 'badge', '--offline', '--dry-run', '--json');
  assert.deepEqual(fingerprint(application), before, 'dry-run must not publish guides or components');
  const indexBefore = fs.readFileSync(index, 'utf8');
  run(application, 'ui', 'add', 'badge', '--offline');
  assert.ok(fs.readFileSync(index, 'utf8').startsWith(indexBefore));
  assert.ok(fs.existsSync(path.join(components, 'badge-1.0.0-default.AGENTS.md')));
  assert.equal(fs.readFileSync(buttonGuide, 'utf8'), 'Owner component advice.\n');
  assert.ok(fs.readFileSync(rootGuide, 'utf8').endsWith('Owner project advice.\n'));

  run(application, 'ui', 'add', 'aspect-ratio', '--offline', '--path', 'local/controls');
  assert.ok(fs.readFileSync(path.join(application, 'local/controls/AGENTS.md'), 'utf8').includes('aspect-ratio-1.0.0-default.AGENTS.md'));
  run(application, 'format');
  run(application, 'format', '--check');
  run(application, 'check');
  console.log('CLI agent guidance: UI scaffold, checked props, owner preservation, idempotence, dry-run, custom paths, format and check passed.');
} finally {
  const resolved = fs.realpathSync(temporary);
  assert.equal(path.dirname(resolved), fs.realpathSync(os.tmpdir()));
  assert.ok(path.basename(resolved).startsWith('tsrust-agents-smoke-'));
  fs.rmSync(resolved, {recursive: true, force: true});
}
