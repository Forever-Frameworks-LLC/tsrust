import assert from 'node:assert/strict';
import path from 'node:path';
import {readFileSync, writeFileSync, copyFileSync} from 'node:fs';
import {root, compiler, verifyCompiler, temporary, run, tokens, sha} from './harness.mjs';

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
  const emitted = path.join(dir, 'control.rs'), binary = path.join(dir, 'control');
  const native = JSON.parse(readFileSync(path.join(root, 'release-native-provenance.json'), 'utf8'));
  const verifyLibraries = () => { for (const [file, digest] of Object.entries(native.libraries)) assert.equal(sha(file), digest, file); };
  verifyLibraries();
  run(compiler, ['check', source]);
  run(compiler, ['emit', source, '-o', emitted]);
  run('rustc', ['--edition=2024', '--crate-name', 'ui_release_control', emitted,
    '--extern', `rayon=${native.rayon}`, '-L', `dependency=${native.dependency_directory}`, '-o', binary]);
  assert.equal(run(binary, []).stdout, '');
  verifyLibraries();
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
  return {observer: 'native-only calibration passed', checkedResourceBoundary: 'blocked', diagnostic: releaseDiagnostic,
    synchronousB2: 'open', asyncWorkers: 'intended; transport, cancellation and cleanup completion unproved'};
}
