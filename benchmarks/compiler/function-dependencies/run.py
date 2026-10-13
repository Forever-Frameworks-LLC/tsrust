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


def source(count, scalar, call_main, recursive=False, inferred=False, callable_result=False, record_result=False, generic_records=False, generic_functions=False):
    functions = []
    if generic_functions:
        for index in range(count):
            result = "value" if index + 1 == count else f"step{index + 1}<T>(value)"
            functions.append(f"function step{index}<T>(value:T):T {{ return {result}; }}")
        functions.append(f"function main() {{ console.log(step0<{scalar}>(7)); }}")
        return "\n".join([*functions, *(["main();"] if call_main else []), ""])
    if generic_records:
        for index in range(count):
            field = 'value: T' if index + 1 == count else f'next: Layer{index + 1}<T>[]'
            functions.append(f'type Layer{index}<T> = {{ {field}; }};')
        functions.append(f'function main() {{ const root: Layer0<{scalar}> = {{ next: [] }}; console.log(root.next.length); }}')
        return '\n'.join([*functions, *(['main();'] if call_main else []), ''])
    if record_result:
        functions.append(f'type Item = {{ value: {scalar}; }};')
        for index in range(count):
            seed = 'if (remaining === 0) { return value; } ' if index + 1 == count else ''
            remaining = 'remaining - 1' if index + 1 == count else 'remaining'
            functions.append(f'function step{index}(value: Item, remaining: {scalar}): Item {{ {seed}return step{(index + 1) % count}(value, {remaining}); }}')
        return '\n'.join([*functions, 'function main() { const result = step0({ value: 7 }, 0); console.log(result.value); }', *(['main();'] if call_main else []), ''])
    if callable_result:
        for index in range(count):
            seed = f'if (remaining === 0) {{ return (): {scalar} => {{ return 7; }}; }} ' if index == 0 else ''
            remaining = 'remaining - 1' if index == 0 else 'remaining'
            functions.append(f'function step{index}(remaining: {scalar}): () => {scalar} {{ {seed}return step{(index + 1) % count}({remaining}); }}')
        return '\n'.join([*functions, 'function main() { const action = step0(0); console.log(action()); }', *(['main();'] if call_main else []), ''])
    for index in range(count):
        if recursive:
            seed = index == 0 if inferred else index + 1 == count
            terminal = 'if (remaining === 0) { return value; } ' if seed else ''
            result_annotation = '' if inferred and not call_main else f': {scalar}'
            next_count = 'remaining - 1' if index + 1 == count else 'remaining'
            functions.append(f'function step{index}(value: {scalar}, remaining: {scalar}){result_annotation} {{ {terminal}return step{(index + 1) % count}(value, {next_count}); }}')
            continue
        result = 'value' if index + 1 == count else f'step{index + 1}(value)'
        functions.append(f'function step{index}(value: {scalar}): {scalar} {{ return {result}; }}')
    entry = 'function main() { console.log(step0(7, 0)); }' if recursive else 'function main() { console.log(step0(7)); }'
    return '\n'.join([*functions, entry,
                      *(['main();'] if call_main else []), ''])


def digest(data):
    return hashlib.sha256(data).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', type=Path, default=Path(os.environ.get('TSRUST', '/private/tmp/tsrust-gate100-editor-target/debug/tsrust')))
    parser.add_argument('--measure', action='store_true')
    parser.add_argument('--generic-functions', action='store_true', help='Use explicit generic function dependency chains')
    parser.add_argument('--generic-records', action='store_true', help='Use the generic materialization workload; incompatible with recursive function variants')
    parser.add_argument('--record', action='store_true', help='Return data records through a recursive group; requires --recursive')
    parser.add_argument('--callable', action='store_true', help='Resolve recursive callable environments; requires --recursive')
    parser.add_argument('--inferred', action='store_true', help='Infer native recursive results; requires --recursive')
    parser.add_argument('--recursive', action='store_true', help='Use the annotated recursive-group manifest')
    parser.add_argument('--only', type=int, nargs='+', choices=MANIFEST['functionCounts'])
    parser.add_argument('--phases', nargs='+', choices=list(MANIFEST['phases']))
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Output already exists; preserve the original record and choose a new path')
    variants = [name for name in ('inferred', 'callable', 'record') if getattr(args, name)]
    if variants and not args.recursive:
        parser.error('Recursive variants require --recursive')
    if len(variants) > 1:
        parser.error('Select only one recursive variant')
    manifest_name = (variants[0] + '-' if variants else '') + 'recursive-manifest.json'
    manifest_path = MANIFEST_PATH.with_name(manifest_name) if args.recursive else MANIFEST_PATH
    if args.generic_records:
        if args.recursive or variants:
            parser.error('Generic materialization is separate from recursive function variants')
        manifest_path = MANIFEST_PATH.parent.parent / 'generic-materialization/manifest.json'
    if args.generic_functions:
        if args.recursive or args.generic_records or variants:
            parser.error('Generic function chains are separate from other variants')
        manifest_path = MANIFEST_PATH.parent.parent / 'generic-function-dependencies/manifest.json'
    manifest = json.loads(manifest_path.read_text())
    counts = manifest.get('declarationCounts', manifest.get('functionCounts'))
    if args.only and any(count not in counts for count in args.only):
        parser.error('Requested count is not registered for this workload')
    compiler = args.compiler.resolve(strict=True)
    tsc = ROOT / 'benchmarks/runtime/node_modules/typescript/bin/tsc'
    record = {
        'schemaVersion': 1, 'manifest': manifest, 'manifestSha256': digest(manifest_path.read_bytes()),
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
            for count in args.only or counts:
                native = directory / 'main.tsr'
                reference = directory / 'main.ts'
                native.write_text(source(count, 'int', False, args.recursive, args.inferred, args.callable, args.record, args.generic_records, args.generic_functions))
                reference.write_text(source(count, 'number', True, args.recursive, args.inferred, args.callable, args.record, args.generic_records, args.generic_functions))
                subprocess.run([str(compiler), 'format', str(native), '--color=never'], check=True, capture_output=True)
                formatted = native.read_bytes()
                subprocess.run([str(compiler), 'format', str(native), '--color=never'], check=True, capture_output=True)
                assert native.read_bytes() == formatted, 'Formatter must be idempotent'
                group = {('declarationCount' if args.generic_records else 'functionCount'): count, 'nativeSource': native.read_text(), 'referenceSource': reference.read_text(),
                         'nativeSha256': digest(formatted), 'referenceSha256': digest(reference.read_bytes()), 'phases': {}}
                record['groups'].append(group)
                commands = {
                    'native-check': [str(compiler), 'check', str(native), '--color=never'],
                    'native-emit': [str(compiler), 'emit', str(native)],
                    'node-tsc-check': ['node', str(tsc), '--noEmit', '--strict', '--target', 'ES2022', str(reference)],
                }
                for phase in args.phases or manifest['phases']:
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
                        for _ in range(manifest['warmups']):
                            subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, timeout=120)
                            result['warmupsCompleted'] += 1
                            save()
                        for _ in range(manifest['samples']):
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
