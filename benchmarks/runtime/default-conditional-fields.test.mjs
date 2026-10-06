import { expectedChecksum } from './oracle.mjs';
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { execFileSync } from 'node:child_process';
import { assertABI } from './adapters.mjs';
const require = createRequire(import.meta.url);
const ts = require('./node_modules/typescript');
const here = path.dirname(new URL(import.meta.url).pathname);
const root = path.resolve(here, '../..');

test('default-owned conditional fields retain matched branch and replacement results', () => {
  const id = 'default_conditional_callable_fields';
  const source = fs.readFileSync(path.join(here, 'workloads', id + '.tsr'), 'utf8');
  const reference = fs.readFileSync(path.join(here, 'workloads', id + '.ts'), 'utf8');
  const manifest = JSON.parse(fs.readFileSync(path.join(here, 'manifest.json'), 'utf8'));
  const workload = manifest.workloads.find(w => w.id === id);
  const compiler = path.join(root, 'target/editor/debug/tsrust');
  const emitted = execFileSync(compiler, ['emit', path.join(here, 'workloads', id + '.tsr')], {encoding: 'utf8'});
  assertABI(emitted, workload, {source});
  const js = ts.transpileModule(reference, {compilerOptions: {target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS}}).outputText;
  const evaluate = new Function('require', 'n', 'seed', js + '\nreturn workload(n, seed);');
  const temporary = fs.mkdtempSync(path.join(root, 'target/editor/owned-fields-workload-'));
  try {
    for (const [n, seed] of [[0, 7], [2, 7], [17, 0], [20000, 7]]) {
      let expected = seed;
      for (let i = 0; i < n; i++) {
        expected += i % 2 === 0 ? seed + 2 * i : seed;
        expected += i % 3 === 0 ? 2 * i : i + 1;
      }
      assert.equal(expectedChecksum(id, n, seed), expected);
      assert.equal(evaluate(require, n, seed), expected);
      const file = path.join(temporary, 'main.tsr');
      fs.writeFileSync(file, source.replace('function main() { console.log(workload(2, 7)); }', `function main(){console.log(workload(${n},${seed}));}`));
      for (const flags of [[], ['--release']]) {
        assert.equal(execFileSync(compiler, ['run', file, ...flags], {encoding: 'utf8'}), expected + '\n');
      }
    }
  } finally { fs.rmSync(temporary, {recursive: true, force: true}); }
});
