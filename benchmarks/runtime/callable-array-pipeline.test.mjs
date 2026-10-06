import { expectedChecksum } from './oracle.mjs';
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { createRequire } from 'node:module';
import { execFileSync } from 'node:child_process';
import { assertABI } from './adapters.mjs';
const require = createRequire(import.meta.url);
const here = path.dirname(new URL(import.meta.url).pathname);
const root = path.resolve(here, '../..');

for (const id of ['callable_array_mutable_pipeline', 'callable_array_transfer_pipeline', 'callable_array_once_transfer', 'callable_array_reduction_pipeline']) {
test(`${id} matches independent retained observations`, () => {
  const source = fs.readFileSync(path.join(here, 'workloads', id + '.tsr'), 'utf8');
  const manifest = JSON.parse(fs.readFileSync(path.join(here, 'manifest.json'), 'utf8'));
  const workload = manifest.workloads.find(w => w.id === id);
  const compiler = path.join(root, 'target/editor/debug/tsrust');
  const emitted = execFileSync(compiler, ['emit', path.join(here, 'workloads', id + '.tsr')], {encoding: 'utf8'});
  assertABI(emitted, workload, {source});
  const checked = path.join(root, 'target/editor/callable-array-pipeline-tsc');
  execFileSync(process.execPath, [path.join(here, 'node_modules/typescript/bin/tsc'), path.join(here, 'workloads', id + '.ts'), '--strict', '--target', 'ES2022', '--module', 'commonjs', '--outDir', checked]);
  const js = fs.readFileSync(path.join(checked, id + '.js'), 'utf8');
  const evaluate = new Function('require', 'n', 'seed', js + '\nreturn workload(n, seed);');
  const temporary = fs.mkdtempSync(path.join(root, 'target/editor/callable-array-workload-'));
  try {
    for (const [n, seed] of [[0, 7], [2, 7], [17, 0], [20000, 7]]) {
      const expected = id === 'callable_array_reduction_pipeline'
        ? seed + 15 * n * seed + 15 * n * (n - 1) / 2 + 54 * n
        : id === 'callable_array_mutable_pipeline'
        ? seed + 6 * n * seed + 3 * n * (n - 1) + 19 * n
        : seed + 4 * n * seed + 2 * n * (n - 1) + (id === 'callable_array_once_transfer' ? 10 : 12) * n;
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
}
