"""Validate or measure fresh-process compilation of deterministic call chains."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]
MANIFEST_PATH = Path(__file__).with_name('manifest.json')
MANIFEST = json.loads(MANIFEST_PATH.read_text())


def source(count, scalar, call_main):
    functions = []
    for index in range(count):
        result = 'value' if index + 1 == count else f'step{index + 1}(value)'
        functions.append(f'function step{index}(value: {scalar}): {scalar} {{ return {result}; }}')
    return '\n'.join([*functions, 'function main() { console.log(step0(7)); }',
                      *(['main();'] if call_main else []), ''])


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, default=Path(os.environ.get('TSRUST', '/private/tmp/tsrust-gate100-editor-target/debug/tsrust')))
    parser.add_argument('--measure', action='store_true')
    parser.add_argument('--only', type=int, nargs='+', choices=MANIFEST['functionCounts'])
    parser.add_argument('--phases', nargs='+', choices=list(MANIFEST['phases']))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    compiler = args.compiler.resolve(strict=True)
    tsc = ROOT / 'benchmarks/runtime/node_modules/typescript/bin/tsc'
    record = {
        'schemaVersion': 1, 'manifest': MANIFEST, 'manifestSha256': digest(MANIFEST_PATH.read_bytes()),
        'generatorSha256': digest(Path(__file__).read_bytes()),
        'compiler': str(compiler), 'compilerSha256': digest(compiler.read_bytes()),
        'gitHead': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
        'gitStatus': subprocess.check_output(['git', 'status', '--short'], cwd=ROOT, text=True),
        'compilerSourceHashes': {str(path.relative_to(ROOT)): digest(path.read_bytes())
            for path in sorted((ROOT / 'crates/tsrust-compiler').rglob('*'))
            if path.is_file() and path.suffix in {'.rs', '.json', '.toml', '.tsr'}},
        'workspaceHashes': {name: digest((ROOT / name).read_bytes()) for name in ['Cargo.toml', 'Cargo.lock']},
        'nodeVersion': subprocess.check_output(['node', '--version'], text=True).strip(),
        'typescriptVersion': subprocess.check_output(['node', str(tsc), '--version'], text=True).strip(),
        'measurement': args.measure, 'groups': [], 'complete': False,
    }

    def save():
        args.output.write_text(json.dumps(record, indent=2) + '\n')

    save()
    try:
        with tempfile.TemporaryDirectory(prefix='tsrust-function-dependencies-') as directory:
            directory = Path(directory)
            for count in args.only or MANIFEST['functionCounts']:
                native = directory / 'main.tsr'
                reference = directory / 'main.ts'
                native.write_text(source(count, 'int', False))
                reference.write_text(source(count, 'number', True))
                subprocess.run([str(compiler), 'format', str(native), '--color=never'], check=True, capture_output=True)
                formatted = native.read_bytes()
                subprocess.run([str(compiler), 'format', str(native), '--color=never'], check=True, capture_output=True)
                assert native.read_bytes() == formatted, 'Formatter must be idempotent'
                group = {'functionCount': count, 'nativeSource': native.read_text(), 'referenceSource': reference.read_text(),
                         'nativeSha256': digest(formatted), 'referenceSha256': digest(reference.read_bytes()), 'phases': {}}
                record['groups'].append(group)
                commands = {
                    'native-check': [str(compiler), 'check', str(native), '--color=never'],
                    'native-emit': [str(compiler), 'emit', str(native)],
                    'node-tsc-check': ['node', str(tsc), '--noEmit', '--strict', '--target', 'ES2022', str(reference)],
                }
                for phase in args.phases or MANIFEST['phases']:
                    command = commands[phase]
                    result = {'command': command, 'validated': False, 'warmupsCompleted': 0, 'nanoseconds': []}
                    group['phases'][phase] = result
                    save()
                    validation = subprocess.run(command, capture_output=True, timeout=120)
                    result['returncode'] = validation.returncode
                    result['stderr'] = validation.stderr.decode(errors='replace')
                    if validation.returncode:
                        save()
                        raise RuntimeError(f'{count} {phase} failed: {result["stderr"]}')
                    result['validated'] = True
                    if args.measure:
                        for _ in range(MANIFEST['warmups']):
                            subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, timeout=120)
                            result['warmupsCompleted'] += 1
                            save()
                        for _ in range(MANIFEST['samples']):
                            start = time.perf_counter_ns()
                            subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, timeout=120)
                            result['nanoseconds'].append(time.perf_counter_ns() - start)
                            save()
                    save()
                    print(f'{count} {phase}: valid; {len(result["nanoseconds"])} samples', flush=True)
        record['complete'] = True
    finally:
        save()


if __name__ == '__main__':
    main()
