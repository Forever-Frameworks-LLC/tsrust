import { execFileSync } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { join } from 'node:path';

const grammar = fileURLToPath(new URL('.', import.meta.url));
const output = fileURLToPath(new URL('../grammars/', import.meta.url));
if (!process.env.WASI_SDK_PATH) throw new Error('Set WASI_SDK_PATH to the WASI SDK directory.');
mkdirSync(output, { recursive: true });
execFileSync(join(process.env.WASI_SDK_PATH, 'bin', 'clang'), [
  '-fPIC', '-shared', '-Os',
  '-Wl,--export=tree_sitter_tsrust',
  '-Wl,--max-memory=4294967296',
  '-o', join(output, 'tsrust.wasm'),
  '-I', join(grammar, 'src'),
  join(grammar, 'src/parser.c'),
  join(grammar, 'src/scanner.c'),
], { stdio: 'inherit' });
