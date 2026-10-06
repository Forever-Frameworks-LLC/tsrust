import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execFileSync } from 'node:child_process';
const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../../..');
const manifest = JSON.parse(fs.readFileSync(path.join(here, 'manifest.json'), 'utf8'));
const compiler = path.join(root, 'target/editor/debug/tsrust');
const temporary = fs.mkdtempSync(path.join(root, 'target/editor/stdlib-selection-'));
const environment = {...process.env, TSRUST_CACHE_DIR: path.join(temporary, 'cache')};
try {
  for (const workload of manifest.workloads) {
    const directory = path.join(here, workload.directory);
    const source = path.join(directory, 'main.tsr');
    execFileSync(compiler, ['check', source], {env: environment});
    const emitted = execFileSync(compiler, ['emit', source], {env: environment, encoding: 'utf8'});
    assert.equal(emitted.includes('mod tsrust_native_path'), workload.nativeFeatures.includes('path'));
    const output = path.join(temporary, workload.directory);
    const configuration = path.join(temporary, workload.directory + '.json');
    fs.writeFileSync(configuration, JSON.stringify({
      extends: path.join(root, 'benchmarks/runtime/tsconfig.json'),
      files: [path.join(directory, 'main.ts')],
      compilerOptions: {outDir: output},
    }));
    execFileSync(process.execPath, [path.join(root, 'benchmarks/runtime/node_modules/typescript/bin/tsc'), '--project', configuration]);
    assert.equal(execFileSync(process.execPath, [path.join(output, 'main.js')], {encoding: 'utf8'}), workload.stdout);
    for (const state of ['cold', 'warm']) {
      assert.equal(execFileSync(compiler, ['run', source], {env: environment, encoding: 'utf8'}), workload.stdout, `${workload.id}: ${state}`);
    }
    console.log(`${workload.id}: frontend, emission selection, native cold/warm output and compiled TypeScript passed`);
  }
} finally { fs.rmSync(temporary, {recursive: true, force: true}); }
