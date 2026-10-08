import {replaceCodeSnippet} from './source-contract-tokens.mjs';
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

test('generic function pipeline retains matched checksums and native ABI', () => {
  const id = 'generic_function_record_pipeline';
  const source = fs.readFileSync(path.join(here, 'workloads', id + '.tsr'), 'utf8');
  const reference = fs.readFileSync(path.join(here, 'workloads', id + '.ts'), 'utf8');
  const manifest = JSON.parse(fs.readFileSync(path.join(here, 'manifest.json'), 'utf8'));
  const workload = manifest.workloads.find(w => w.id === id);
  const compiler = path.join(root, 'target/editor/debug/tsrust');
  const emitted = execFileSync(compiler, ['emit', path.join(here, 'workloads', id + '.tsr')], {encoding: 'utf8'});
  assertABI(emitted, workload, {source});
  const js = ts.transpileModule(reference, {compilerOptions: {target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS}}).outputText;
  const evaluate = new Function('require', 'n', 'seed', js + '\nreturn workload(n, seed);');
  const temporary = fs.mkdtempSync(path.join(root, 'target/editor/generic-function-workload-'));
  try {
    for (const [n, seed] of [[0, 7], [2, 7], [17, 0], [20000, 7]]) {
      let expected = seed;
      for (let i = 0; i < n; i++) expected += (i + seed) % 1009;
      assert.equal(evaluate(require, n, seed), expected);
      const file = path.join(temporary, 'main.tsr');
      fs.writeFileSync(file, replaceCodeSnippet(source, 'function main() { console.log(workload(2, 7)); }', `function main(){console.log(workload(${n},${seed}));}`));
      assert.equal(execFileSync(compiler, ['run', file], {encoding: 'utf8'}), expected + '\n');
    }
  } finally { fs.rmSync(temporary, {recursive: true, force: true}); }
});
