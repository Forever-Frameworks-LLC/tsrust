"""Compare Worklist rebuilds after changing an unused caller environment value."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import tomllib
import uuid

import rust_control

ROOT = Path(__file__).resolve().parents[2]
PROJECT = ROOT / 'apps/tasks/desktop'
CACHE = PROJECT / 'target/tsrust-native'


def digest(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def source_snapshot():
    result = {}
    for directory, children, files in os.walk(ROOT / 'apps/tasks'):
        children[:] = [name for name in children if name not in {'.git', 'target', 'node_modules'}]
        for name in files:
            path = Path(directory) / name
            if path.suffix in {'.tsr', '.tsrx'} or name == 'tsrust.config.json' or name.endswith('.bindings.json'):
                result[str(path.relative_to(ROOT))] = digest(path)
    home = Path(os.environ.get('CARGO_HOME', Path.home() / '.cargo'))
    if not home.is_absolute():
        home = PROJECT / home
    pending = [(directory / '.cargo' / name, False) for directory in [CACHE, *CACHE.parents]
               for name in ['config', 'config.toml']]
    pending.extend((home / name, False) for name in ['config', 'config.toml'])
    visited = set()
    while pending:
        path, required = pending.pop()
        try:
            content = path.read_bytes()
        except FileNotFoundError:
            if required:
                raise
            result[str(path)] = None
            continue
        result[str(path)] = hashlib.sha256(content).hexdigest()
        identity = (path.resolve(), path.parent.resolve())
        if identity in visited:
            continue
        visited.add(identity)
        for included in tomllib.loads(content.decode()).get('include', []):
            name = included if isinstance(included, str) else included['path']
            optional = False if isinstance(included, str) else included.get('optional', False)
            pending.append((path.parent / name, not optional))
    return result


def support_identity(data):
    fields = []
    position = 0
    while position < len(data):
        if len(data) - position < 8:
            raise ValueError('Truncated support identity length')
        size = int.from_bytes(data[position:position + 8], 'little')
        position += 8
        if size > len(data) - position:
            raise ValueError('Truncated support identity field')
        fields.append(data[position:position + size])
        position += size
    if len(fields) != 11 or not fields[0].startswith(b'runtime-dependencies-abi-'):
        raise ValueError('Unsupported GPUI support identity schema')
    return [hashlib.sha256(value).hexdigest() for value in fields[:10]]


def context_build_timings(stderr):
    phase = rust_control.build_timings(stderr)
    if '[build] runtime=miss' not in stderr:
        raise ValueError('Expected a support cache miss')
    return phase


def keys(namespace):
    root = CACHE / namespace
    return {path.name for path in root.iterdir() if (path / 'identity').is_file()} if root.is_dir() else set()


def compare(before, after, samples):
    sources = source_snapshot()
    result = {'protocol': 'worklist-runtime-context-v1', 'timestamp': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
              'harnessSha256': digest(Path(__file__)), 'sources': sources,
              'compilerSha256': {name: digest(path) for name, path in [('before', before), ('after', after)]},
              'manifest': {'workloads': [{'id': name, 'iterations': 1} for name in
                  ['worklist-context-build', 'worklist-context-native-build']]},
              'execution': [], 'setup': [], 'observations': [], 'createdCacheKeys': {'v2': [], 'runtime/v2': []}}
    expected_support = None
    context = uuid.uuid4().hex
    try:
        with tempfile.TemporaryDirectory(prefix='tsrust-worklist-context-') as directory:
            binary = Path(directory) / 'worklist'
            for engine, compiler in [('before', before), ('after', after)]:
                for index in range(samples + 1):
                    assert source_snapshot() == sources, 'Worklist sources changed during comparison'
                    previous = {name: keys(name) for name in result['createdCacheKeys']}
                    env = dict(os.environ, CARGO_TARGET_DIR=str(ROOT / 'target'), CARGO_INCREMENTAL='0', CARGO_BUILD_JOBS='1',
                               TSRUST_CACHE_DIR=str(CACHE), TSRUST_BUILD_TIMINGS='1', TSRUST_RUNTIME_CONTEXT=f'{context}-{engine}-{index}')
                    print(f'{engine} {"setup" if index == 0 else f"sample {index}"}: building', file=sys.stderr, flush=True)
                    elapsed, output, quiet = rust_control.run([compiler, 'build', str(PROJECT / 'src/main.tsrx'), '-o', str(binary)], PROJECT, env)
                    created = {name: sorted(keys(name) - previous[name]) for name in previous}
                    for name, added in created.items():
                        result['createdCacheKeys'][name].extend(added)
                    assert len(created['runtime/v2']) == 1, 'Expected one new support cache entry'
                    identity = support_identity((CACHE / 'runtime/v2' / created['runtime/v2'][0] / 'identity').read_bytes())
                    if expected_support is None:
                        expected_support = identity
                    assert identity == expected_support, 'Compiler runtime, dependency graph or profile changed'
                    assert source_snapshot() == sources, 'Worklist sources changed during build'
                    phase = context_build_timings(output.stderr)
                    native = phase['nativeBuildMs']
                    observed = {'engine': engine, 'index': index, 'elapsedMs': elapsed, 'nativeBuildMs': native, 'phases': phase, 'quietMachine': quiet}
                    result['setup' if index == 0 else 'observations'].append(observed)
                    if index:
                        for workload, duration in [('worklist-context-build', elapsed), ('worklist-context-native-build', native)]:
                            result['execution'].append({'workload': workload, 'engine': engine, 'durations': [duration]})
                    print(f'{engine} {index}: total {elapsed:.1f}ms; native {native:.1f}ms', file=sys.stderr, flush=True)
    finally:
        print(json.dumps(result, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before-compiler', required=True)
    parser.add_argument('--after-compiler', required=True)
    parser.add_argument('--samples', type=int, default=3)
    args = parser.parse_args()
    if args.samples < 1:
        parser.error('positive sample count required')
    compare(str(Path(args.before_compiler).resolve(strict=True)), str(Path(args.after_compiler).resolve(strict=True)), args.samples)


if __name__ == '__main__':
    main()
