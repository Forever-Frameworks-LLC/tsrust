"""Isolate MSVC linking across ordinary/verbatim paths and short/deep workspaces."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import tomllib


def probe(root, verbatim, llvm):
    root.mkdir(parents=True)
    (root / 'src').mkdir()
    (root / 'Cargo.toml').write_text(
        '[package]\nname="tsrust-linker-probe"\nversion="0.1.0"\nedition="2024"\n[workspace]\n'
    )
    (root / 'src/lib.rs').write_text('pub fn answer() -> u32 { 42 }\n')
    (root / 'build.rs').write_text('fn main() { println!("cargo:rerun-if-changed=build.rs"); }\n')
    target = str(root / 'native-target')
    if verbatim:
        target = '\\\\?\\' + target
    command = ['rustup', 'run', VERSION, 'cargo', 'build', '--offline', '-vv',
               '--manifest-path', str(root / 'Cargo.toml'), '--target-dir', target]
    env = {key: value for key, value in os.environ.items()
           if not key.startswith('CARGO_PROFILE_')}
    if llvm:
        env['CARGO_ENCODED_RUSTFLAGS'] = '-Clinker-flavor=lld-link'
    env['CARGO_INCREMENTAL'] = '0'
    env['CARGO_BUILD_JOBS'] = '1'
    result = subprocess.run(command, cwd=root, env=env, text=True,
                            capture_output=True, timeout=90)
    print(json.dumps({'root': str(root), 'targetCharacters': len(target),
                      'verbatim': verbatim, 'llvm': llvm, 'exitCode': result.returncode}), flush=True)
    if result.returncode:
        print((result.stdout + result.stderr)[-24000:], flush=True)
    return result.returncode == 0


if __name__ == '__main__':
    if os.name != 'nt':
        raise SystemExit('This probe requires Windows/MSVC.')
    VERSION = tomllib.loads((Path(__file__).resolve().parents[1] / 'rust-toolchain.toml').read_text())['toolchain']['channel']
    results = []
    with tempfile.TemporaryDirectory(prefix='tsrust-linker-') as scratch:
        for deep in (False, True):
            for verbatim in (False, True):
                for llvm in (False, True):
                    root = Path(scratch) / f'{deep}-{verbatim}-{llvm}'
                    if deep:
                        while len(str(root)) < 180:
                            root /= 'nested-workspace'
                    passed = probe(root, verbatim, llvm)
                    if llvm or not deep:
                        results.append(passed)
    raise SystemExit(0 if all(results) else 1)
