"""Compare Worklist and Studio frontend time while requiring identical emitted Rust."""
import argparse
import datetime
import json
import os
from pathlib import Path
import sys
import tempfile

import rust_control
import worklist_runtime_context as context

PROJECTS = [('worklist', context.ROOT / 'apps/tasks/desktop'), ('studio', context.ROOT / 'apps/studio')]


def compare(before, after, samples):
    roots = [context.ROOT / 'apps/tasks', context.ROOT / 'apps/studio']
    sources = context.source_snapshot(roots)
    compilers = {'before': Path(before), 'after': Path(after)}
    hashes = {name: context.digest(path) for name, path in compilers.items()}
    report = {'timestamp': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'protocol': 'application-frontend-emission-v1', 'sources': sources, 'compilerSha256': hashes,
              'harnessSha256': context.digest(Path(__file__)),
              'manifest': {'workloads': [{'id': name + '-frontend-emit', 'iterations': 1} for name, _ in PROJECTS]},
              'setup': [], 'observations': [], 'execution': []}
    expected = {}
    try:
        with tempfile.TemporaryDirectory(prefix='tsrust-frontend-') as directory:
            output = Path(directory) / 'program.rs'
            for name, project in PROJECTS:
                for engine, compiler in compilers.items():
                    for index in range(samples + 1):
                        assert context.source_snapshot(roots) == sources, 'Application source/config changed'
                        assert {key: context.digest(path) for key, path in compilers.items()} == hashes, 'Compiler changed'
                        elapsed, _, quiet = rust_control.run([str(compiler), 'emit', str(project / 'src/main.tsrx'), '-o', str(output)], project, dict(os.environ))
                        emitted = context.digest(output)
                        assert emitted == expected.setdefault(name, emitted), 'Emitted Rust changed'
                        assert context.source_snapshot(roots) == sources, 'Application source/config changed during emission'
                        assert {key: context.digest(path) for key, path in compilers.items()} == hashes, 'Compiler changed during emission'
                        row = {'project': name, 'engine': engine, 'index': index, 'elapsedMs': elapsed,
                               'quietMachine': quiet, 'emittedRustSha256': emitted}
                        report['setup' if index == 0 else 'observations'].append(row)
                        if index:
                            report['execution'].append({'workload': name + '-frontend-emit', 'engine': engine, 'durations': [elapsed]})
                        print(f'{name} {engine} {index}: {elapsed:.2f}ms', file=sys.stderr, flush=True)
    finally:
        print(json.dumps(report, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--before-compiler', required=True, type=Path)
    parser.add_argument('--after-compiler', required=True, type=Path)
    parser.add_argument('--samples', type=int, default=3)
    args = parser.parse_args()
    if args.samples < 1:
        parser.error('positive sample count required')
    compare(args.before_compiler.resolve(strict=True), args.after_compiler.resolve(strict=True), args.samples)


if __name__ == '__main__':
    main()
