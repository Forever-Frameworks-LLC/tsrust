import assert from 'node:assert/strict';
import path from 'node:path';
import {readFileSync, writeFileSync, copyFileSync} from 'node:fs';
import {root, compiler, verifyCompiler, temporary, run, tokens, formattedWitnesses, execute} from './harness.mjs';

export const releaseDiagnostic = 'Boundary data cannot implement Drop; keep resource owners private to the Rust adapter.';
export function prepareRelease(dir) {
  verifyCompiler();
  for (const name of ['tracked.rs', 'plain.rs', 'native-calibration.rs']) {
    copyFileSync(path.join(root, 'release', name), path.join(dir, name));
  }
  for (const name of ['blocked.ts', 'control.tsr']) {
    const source = readFileSync(path.join(root, 'release', name), 'utf8');
    const file = path.join(dir, name.replace(/\.ts$/, '.tsr'));
    writeFileSync(file, source);
    const formatted = run(compiler, ['format', file]).stdout;
    assert.deepEqual(tokens(formatted), tokens(source));
    writeFileSync(file, formatted);
    assert.equal(run(compiler, ['format', file]).stdout, formatted);
  }
}
export function boundaryEvidence(dir) {
  for (const command of ['check', 'emit']) {
    const result = run(compiler, [command, path.join(dir, 'blocked.tsr')], 1);
    assert.ok(result.stderr.includes(releaseDiagnostic), result.stderr);
    assert.match(result.stderr, /rust-bridge/);
    assert.doesNotMatch(result.stderr, /panicked|stack overflow/);
  }
  const source = path.join(dir, 'control.tsr');
  const emitted = path.join(dir, 'control.rs');
  run(compiler, ['check', source]);
  run(compiler, ['emit', source, '-o', emitted]);
  // Rebuild through the supported project path; historical dependency binaries are disposable.
  assert.equal(run(compiler, ['run', source]).stdout, '');
  verifyCompiler();
}
export function nativeCalibration(dir) {
  const binary = path.join(dir, 'calibration');
  run('rustc', ['--edition=2024', '--forbid=unsafe_code', path.join(dir, 'native-calibration.rs'), '-o', binary]);
  assert.match(run(binary, []).stdout, /native observer calibration passed/);
  for (const mode of ['count', 'id', 'missing', 'premature', 'order']) {
    const result = run(binary, [mode], 101);
    assert.match(result.stderr, /assertion.*failed/s);
  }
  // Strict Rust transfer control calibrates the native candidate, not Tsrust resource support.
  const illegal = path.join(dir, 'consumed.rs');
  writeFileSync(illegal, '#[path="tracked.rs"] mod tracked; fn main(){let value=tracked::tracked();tracked::close(value);println!("{}",value.id);}');
  const rejection = run('rustc', ['--edition=2024', '--forbid=unsafe_code', illegal, '-o', path.join(dir, 'consumed')], 1);
  assert.match(rejection.stderr, /E0382/);
  verifyCompiler();
}
export function releaseDemo() {
  temporary(dir => { prepareRelease(dir); nativeCalibration(dir); boundaryEvidence(dir); });
  temporary(dir => {
    formattedWitnesses(dir, assert);
    assert.equal(execute(path.join(dir, 'tracked-release.tsr'), dir), 'UI retained resources released\n');
  });
  return {observer: 'checked std:testing resource witness passed', checkedResourceBoundary: 'compiler-owned opaque resource', externalProjectedDrop: releaseDiagnostic,
    synchronousB2: 'checked synchronous witness passed', asyncWorkers: 'unproved; transport, cancellation and cleanup completion remain required'};
}
