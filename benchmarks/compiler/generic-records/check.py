"""Validate deterministic generic-record scaling sources without timing them."""
import json
from pathlib import Path
import subprocess
import tempfile

root = Path(__file__).resolve().parents[3]
manifest = json.loads(Path(__file__).with_name('manifest.json').read_text())

def source(count, scalar):
    records = [f'type Argument{i}={{value:{scalar};}};' for i in range(count)]
    holders = [f'type Holder{i}={{item:Box<Argument{i}>;}};' for i in range(count)]
    return '\n'.join(['type Box<T>={value:T;};', *records, *holders, 'function main(){}', ''])

if __name__ == '__main__':
    with tempfile.TemporaryDirectory(prefix='generic-record-scaling-', dir=root / 'target/editor') as directory:
        for count in manifest['instanceCounts']:
            native = Path(directory) / 'main.tsr'
            reference = Path(directory) / 'main.ts'
            native.write_text(source(count, 'int'))
            reference.write_text(source(count, 'number'))
            subprocess.run([root / 'target/editor/debug/tsrust', 'check', native], check=True, capture_output=True)
            emitted = subprocess.run([root / 'target/editor/debug/tsrust', 'emit', native], check=True, capture_output=True)
            assert emitted.stdout, 'Expected emitted native source'
            subprocess.run(['node', root / 'benchmarks/runtime/node_modules/typescript/bin/tsc', '--noEmit', '--strict', '--target', 'ES2022', reference], check=True, capture_output=True)
            print(f'{count} concrete record arguments: native check/emit and strict TypeScript check passed')
