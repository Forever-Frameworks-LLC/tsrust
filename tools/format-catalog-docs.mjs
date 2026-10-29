// Formats Tsrust code fences inside editor documentation (builtins.json) with the
// project formatter, so hover text and the generated API reference show readable
// examples. Only changed documentation values are rewritten in the raw JSON text.
// Recorded regression `examples` are left unchanged.
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const compiler = process.env.TSRUST || path.join(root, 'target/editor/debug/tsrust');
const catalogPath = path.join(root, 'crates/tsrust-lsp/src/builtins.json');
const check = process.argv.includes('--check');

let raw = fs.readFileSync(catalogPath, 'utf8');
const scratch = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-catalog-format-'));
const file = path.join(scratch, 'snippet.tsr');
let changed = 0;
const unformatted = [];
try {
  // Walk documentation string tokens in file order so the rest of the raw JSON,
  // including its indentation and escape spelling, stays untouched.
  const key = '"documentation": ';
  let output = '';
  let cursor = 0;
  for (let at = raw.indexOf(key); at !== -1; at = raw.indexOf(key, cursor)) {
    const open = at + key.length;
    let close = open + 1;
    while (raw[close] !== '"') close += raw[close] === '\\' ? 2 : 1;
    const original = JSON.parse(raw.slice(open, close + 1));
    let documentation = original;
    for (const [fence, body] of original.matchAll(/```(?:tsrust|tsr)?\n([\s\S]*?)```/g)) {
      fs.writeFileSync(file, body);
      const result = spawnSync(compiler, ['format', file, '--write', '--color=never'], {encoding: 'utf8'});
      if (result.error) throw result.error;
      // Snippets the formatter cannot parse stay as written.
      if (result.status !== 0) { unformatted.push(body.split('\n')[0]); continue; }
      const formatted = fs.readFileSync(file, 'utf8');
      if (formatted !== body) { documentation = documentation.replace(fence, () => fence.replace(body, () => formatted)); changed++; }
    }
    output += raw.slice(cursor, open) + (documentation === original ? raw.slice(open, close + 1) : JSON.stringify(documentation));
    cursor = close + 1;
  }
  raw = output + raw.slice(cursor);
} finally {
  fs.rmSync(scratch, {recursive: true, force: true});
}
const before = JSON.parse(fs.readFileSync(catalogPath, 'utf8'));
const after = JSON.parse(raw);
if (Object.keys(before).join() !== Object.keys(after).join()) throw new Error('Catalog structure changed while formatting');
if (check) {
  if (changed) { console.error(`${changed} documentation snippets need formatting.`); process.exit(1); }
} else if (changed) {
  fs.writeFileSync(catalogPath, raw);
}
console.log(`Formatted ${changed} documentation snippets; ${unformatted.length} left as written.`);
