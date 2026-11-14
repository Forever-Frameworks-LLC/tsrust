"""Compare fresh Tsrust builds with the exact Rust invocation they produce."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import tempfile
import time

import baseline


def build_timings(stderr):
    match = re.search(
        r'\[build\] check=([\d.]+)ms emit=([\d.]+)ms backend=([\d.]+)ms '
        r'cache=(hit|miss) \(([\d.]+)ms\) native\+link=([\d.]+)ms',
        stderr,
    )
    if not match or match[4] != 'miss':
        raise ValueError('A matched fresh build must report a native cache miss.')
    names = ['checkMs', 'emitMs', 'backendMs', 'cacheMs', 'nativeBuildMs']
    return dict(zip(names, (float(match[i]) for i in [1, 2, 3, 5, 6])))


def run(command, root, env):
    started = time.perf_counter()
    result = subprocess.run(command, cwd=root, env=env, capture_output=True, text=True, check=True)
    return (time.perf_counter() - started) * 1000, result


def shape(compiler, rustup, count, samples):
    with tempfile.TemporaryDirectory(prefix='tsrust-rust-control-') as directory:
        root = Path(directory)
        baseline.project(root, count)
        capture = root / 'capture.json'
        wrappers = root / 'wrappers'
        wrappers.mkdir()
        wrapper = wrappers / ('rustup.exe' if os.name == 'nt' else 'rustup')
        if os.name == 'nt':
            raise RuntimeError('Invocation capture needs a Windows process wrapper before Windows measurements.')
        wrapper.write_text(
            '#!/usr/bin/env python3\n'
            'import json, os, pathlib, shutil, sys\n'
            'args = sys.argv[1:]\n'
            'sources = [value for value in args if value.endswith(".rs")]\n'
            'if sources:\n'
            '    assert len(sources) == 1\n'
            '    libraries = {}\n'
            '    for argument in args:\n'
            '        if argument.startswith("tsrust_runtime="):\n'
            '            original = argument.split("=", 1)[1]\n'
            f'            copied = {str(root / "libtsrust_runtime.rlib")!r}\n'
            '            shutil.copy2(original, copied)\n'
            '            libraries[argument] = "tsrust_runtime=" + copied\n'
            f'    pathlib.Path({str(capture)!r}).write_text(json.dumps('
            '{"args": args, "source": pathlib.Path(sources[0]).read_text(), "libraries": libraries}))\n'
            f'os.execv({rustup!r}, [{rustup!r}, *args])\n'
        )
        wrapper.chmod(0o755)
        capture_env = dict(os.environ, PATH=str(wrappers) + os.pathsep + os.environ['PATH'],
                           TSRUST_CACHE_DIR=str(root / 'capture-cache'))
        subprocess.run([compiler, 'build', 'main.tsr', '-o', 'capture-program'],
                       cwd=root, env=capture_env, capture_output=True, check=True)
        captured = json.loads(capture.read_text())
        rust_args = captured['args']
        externs = [rust_args[i + 1] for i, value in enumerate(rust_args) if value == '--extern']
        if any(not name.startswith('tsrust_runtime=') for name in externs) or '-L' in rust_args:
            raise ValueError('Extend the protocol to validate external dependency archives before measuring them.')
        emitted = captured['source'].encode()
        rust_source = root / 'program.rs'
        rust_source.write_bytes(emitted)
        rust_args = [captured['libraries'].get(argument, argument) for argument in rust_args]
        source_index = next(i for i, value in enumerate(rust_args) if value.endswith('.rs'))
        rust_args[source_index] = str(rust_source)
        rust_args[rust_args.index('-o') + 1] = str(root / 'rust-program')
        commands = {'rustc': [rustup, *rust_args],
                    'tsrust-build': [compiler, 'build', 'main.tsr', '-o', 'tsrust-program']}
        durations = {engine: [] for engine in commands}
        phases = []
        expected = f'DEV_READY {baseline.expected(count, 1, 0)}\n'
        for repetition in range(samples):
            order = ['rustc', 'tsrust-build'] if repetition % 2 == 0 else ['tsrust-build', 'rustc']
            for engine in order:
                if engine == 'tsrust-build':
                    application_cache = root / 'capture-cache' / 'v2'
                    if application_cache.exists():
                        shutil.rmtree(application_cache)
                env = dict(os.environ, TSRUST_BUILD_TIMINGS='1',
                           TSRUST_CACHE_DIR=str(root / 'capture-cache'))
                env.setdefault('RUST_MIN_STACK', '1073741824')
                elapsed, result = run(commands[engine], root, env)
                binary = root / ('rust-program' if engine == 'rustc' else 'tsrust-program')
                output = subprocess.check_output([str(binary)], cwd=root, text=True)
                if output != expected:
                    raise ValueError(f'{engine} violated the independent output oracle: {output!r}')
                if engine == 'tsrust-build':
                    phase = build_timings(result.stderr)
                    runtime = re.search(r'\[build\] runtime=(\w+)', result.stderr)
                    if captured['libraries'] and (not runtime or runtime[1] != 'hit'):
                        raise ValueError('The shared support runtime must be a cache hit in this protocol.')
                    phase['runtime'] = runtime[1] if runtime else 'none'
                    phases.append(phase)
                durations[engine].append(elapsed)
        return {
            'modules': count + 2,
            'cacheState': 'warm support library; fresh application native cache',
            'sourceSha256': hashlib.sha256(emitted).hexdigest(),
            'supportSha256': {
                argument.split('=', 1)[0]: hashlib.sha256(Path(argument.split('=', 1)[1]).read_bytes()).hexdigest()
                for argument in captured['libraries'].values()
            },
            'backendVersion': subprocess.check_output(
                [rustup, *captured['args'][:3], '--version'], cwd=root, text=True).strip(),
            'commands': commands,
            'durations': durations,
            'phases': phases,
        }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--compiler', required=True)
    parser.add_argument('--samples', type=int, default=3)
    parser.add_argument('--modules', type=int, nargs='+', default=[4, 128])
    args = parser.parse_args()
    if args.samples < 1 or any(count < 1 for count in args.modules):
        parser.error('positive sample and module counts required')
    compiler = str(Path(args.compiler).resolve(strict=True))
    rustup = str(Path(shutil.which('rustup')).resolve(strict=True))
    result = {
        'protocol': 'matched-native-build-v1',
        'timestamp': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
        'compilerSha256': hashlib.sha256(Path(compiler).read_bytes()).hexdigest(),
        'harnessSha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        'host': platform.platform(),
        'manifest': {'workloads': []},
        'execution': [],
        'shapes': [],
    }
    try:
        for count in args.modules:
            measured = shape(compiler, rustup, count, args.samples)
            workload = f'matched-native-build-{count + 2}-modules'
            result['manifest']['workloads'].append({'id': workload, 'iterations': 1})
            for engine, durations in measured['durations'].items():
                result['execution'].append({'workload': workload, 'engine': engine, 'durations': durations})
            result['shapes'].append(measured)
    finally:
        print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
