import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {capturePinnedSources, refreshProjectFingerprints} from '../benchmarks/runtime/refresh-format-fingerprints.mjs';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const compiler = process.env.TSRUST || path.join(root, 'target/editor/debug/tsrust');
const priorSources = capturePinnedSources();
const result = spawnSync(compiler, ['format', root, '--color=never'], {cwd: root, stdio: 'inherit'});
if (result.error) throw result.error;
if (result.status !== 0) process.exit(result.status ?? 1);
console.log('Updated ' + refreshProjectFingerprints([], priorSources) + ' formatting fingerprints.');
