import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import {spawnSync} from 'node:child_process';

test('allocation probes distinguish output disposal from final external-input disposal', () => {
 const directory = fs.mkdtempSync(path.join(os.tmpdir(), 'tsrust-allocation-owners-'));
 const driver = fs.readFileSync(new URL('./allocation-driver.rs', import.meta.url), 'utf8');
 const pins = JSON.parse(fs.readFileSync(new URL('./versions.json', import.meta.url)));
 try {
  for (const externalInput of [false, true]) {
   const input = externalInput ? 'let input = std::borrow::Cow::<str>::Owned("x".repeat(1024));' : '';
   const source = driver
    .replace('let n: i64 = arguments[1].parse().unwrap();', `${input}\nlet n: i64 = arguments[1].parse().unwrap();`)
    .replace('function0(std::hint::black_box(n), std::hint::black_box(7))', externalInput
     ? 'function0(std::hint::black_box(n), std::hint::black_box(7), &input)'
     : 'function0(std::hint::black_box(n), std::hint::black_box(7))')
    + (externalInput
     ? '\nfn function0(n: i64, _: i64, input: &std::borrow::Cow<str>) -> Vec<u8> { std::hint::black_box(input.as_ref()); vec![7; n as usize] }'
     : '\nfn function0(n: i64, _: i64) -> Vec<u8> { vec![7; n as usize] }')
    + '\nfn probe_checksum(value: &[u8]) -> usize { value.iter().map(|byte| *byte as usize).sum() }';
   const file = path.join(directory, `${externalInput}.rs`), binary = path.join(directory, `${externalInput}`);
   fs.writeFileSync(file, source);
   const compiled = spawnSync('rustup', ['run', pins.rust, 'rustc', '--edition=2024', '--crate-name', 'allocation_owners', '-C', 'opt-level=3', file, '-o', binary], {encoding: 'utf8', timeout: 30000});
   assert.equal(compiled.status, 0, compiled.error?.message ?? compiled.stderr);
   const executed = spawnSync(binary, ['64'], {encoding: 'utf8', timeout: 30000});
   assert.equal(executed.status, 0, executed.error?.message ?? executed.stderr);
   const result = JSON.parse(executed.stdout);
   assert.equal(result.checksum, 448);
   assert.equal(result.liveBytesAfterDrop, 0);
   assert.equal(result.liveBytesAfterOutputDrop, 0);
   assert.equal(result.inputRequestedBytes, externalInput ? 1024 : 0);
   assert.equal(result.retainedRequestedBytes, 64);
  }
 } finally { fs.rmSync(directory, {recursive: true, force: true}); }
});
