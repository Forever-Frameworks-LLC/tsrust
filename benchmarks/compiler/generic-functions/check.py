"""Validate deterministic generic-function scaling sources without timing them."""
import hashlib
import json
import re
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[3]
manifest = json.loads(Path(__file__).with_name('manifest.json').read_text())


def source(count, scalar, mode):
    assert count in manifest['instanceCounts']
    assert mode in manifest['modes']
    records = [f'type Argument{i}={{value:{scalar};}};' for i in range(count)]
    calls = []
    for i in range(count):
        calls.append(f'const value{i}:Argument{i}={{value:{i}}};')
        arguments = f'<Argument{i}>' if mode == 'mixed' and i % 2 == 0 else ''
        calls.append(f'identity{arguments}(value{i});')
    # Both paths reuse the same nominal instance, even at the declaration limit.
    calls += ['const repeated:Argument0={value:7};identity(repeated);',
              'identity<(Argument0)>({value:8});']
    return '\n'.join([*records,
                      'function identity<T>(value:T):T{return value;}',
                      'function main(){', *calls, '}', ''])


if __name__ == '__main__':
    with tempfile.TemporaryDirectory(prefix='generic-function-scaling-', dir=root / 'target/editor') as directory:
        for count in manifest['instanceCounts']:
            for mode in manifest['modes']:
                native = Path(directory) / 'main.tsr'
                reference = Path(directory) / 'main.ts'
                native.write_text(source(count, 'int', mode))
                reference.write_text(source(count, 'number', mode))
                subprocess.run([root / 'target/editor/debug/tsrust', 'check', native], check=True, capture_output=True)
                emitted = subprocess.run([root / 'target/editor/debug/tsrust', 'emit', native], check=True, capture_output=True)
                assert emitted.stdout, 'Expected emitted native source'
                instances = len(re.findall(rb'^fn function\d+\(', emitted.stdout, re.MULTILINE))
                assert instances == count, (count, mode, instances)
                subprocess.run(['node', root / 'benchmarks/runtime/node_modules/typescript/bin/tsc', '--noEmit', '--strict', '--target', 'ES2022', reference], check=True, capture_output=True)
                print(json.dumps({'instances': count, 'mode': mode,
                                  'emittedInstances': instances,
                                  'nativeSha256': hashlib.sha256(native.read_bytes()).hexdigest(),
                                  'referenceSha256': hashlib.sha256(reference.read_bytes()).hexdigest(),
                                  'verification': 'native check/emit and strict TypeScript check passed'}))
