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

test('std path benchmark retains matched filename results and native ABI', () => {
  const id = 'std_path_basename_ascii';
  const source = fs.readFileSync(path.join(here, 'workloads', id + '.tsr'), 'utf8');
  const reference = fs.readFileSync(path.join(here, 'workloads', id + '.ts'), 'utf8');
  const manifest = JSON.parse(fs.readFileSync(path.join(here, 'manifest.json'), 'utf8'));
  const workload = manifest.workloads.find(w => w.id === id);
  const compiler = path.join(root, 'target/editor/debug/tsrust');
  const emitted = execFileSync(compiler, ['emit', path.join(here, 'workloads', id + '.tsr')], {encoding: 'utf8'});
  assertABI(emitted, workload, {source});
  const js = ts.transpileModule(reference, {compilerOptions: {target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS}}).outputText;
  const evaluate = new Function('require', 'n', 'seed', js + '\nreturn workload(n, seed);');
  const temporary = fs.mkdtempSync(path.join(root, 'target/editor/std-path-workload-'));
  try {
    for (const [n, seed] of [[0, 7], [2, 7], [17, 0]]) {
      const expected = Array.from({length: n}, (_, i) => (i + seed) % 2 === 0 ? 'archive' : 'report');
      assert.deepEqual(evaluate(require, n, seed), expected);
      const file = path.join(temporary, 'main.tsr');
      fs.writeFileSync(file, source.slice(0, source.indexOf('function main()')) + `function main(){const values=workload(${n},${seed});for(const value of values){console.log(value);}}`);
      assert.equal(execFileSync(compiler, ['run', file], {encoding: 'utf8'}), expected.map(v => v + '\n').join(''));
    }
  } finally { fs.rmSync(temporary, {recursive: true, force: true}); }
});
