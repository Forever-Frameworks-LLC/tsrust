"""Reproducible dev-loop latency samples; writes one JSON record to stdout."""
import argparse
import hashlib
import json
import os
import pathlib
import platform
import queue
import signal
import subprocess
import tempfile
import threading
import time


def leaf(index, extra=0):
    return f"import {{shared}} from './shared.tsr'; export function part{index}():int{{return shared()+{index + extra};}}\n"


def project(root, count):
    (root / 'tsrust.config.json').write_text(json.dumps({'schemaVersion': 1, 'name': 'dev-baseline', 'build': {'entry': 'main.tsr', 'output': 'target/app'}}))
    (root / 'shared.tsr').write_text('export function shared():int{return 1;}\n')
    for index in range(count):
        (root / f'module{index}.tsr').write_text(leaf(index))
    imports = ''.join(f"import {{part{i}}} from './module{i}.tsr';\n" for i in range(count))
    calls = ''.join(f'total += part{i}();\n' for i in range(count))
    (root / 'main.tsr').write_text(imports + 'function main(){let total=0;\n' + calls + 'console.log("DEV_READY",total); }\n')


def expected(count, shared, extra):
    return count * shared + count * (count - 1) // 2 + extra


def cpu_seconds(text):
    total = 0.0
    for component in text.strip().split(':'):
        total = total * 60 + float(component)
    return total


def cache_bytes(root):
    return sum(path.stat().st_size for path in (root / 'target').rglob('*') if path.is_file())


class Dev:
    def __init__(self, compiler, root):
        self.events = queue.Queue()
        self.lines = []
        self.started = time.monotonic()
        self.process = subprocess.Popen([compiler, 'dev', '--verbose'], cwd=root, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, start_new_session=True)
        self.reader = threading.Thread(target=self.read, daemon=True)
        self.reader.start()

    def read(self):
        for line in self.process.stdout:
            event = (time.monotonic(), line.rstrip())
            self.lines.append(event[1])
            self.events.put(event)

    def ready(self, value, started):
        deadline = time.monotonic() + 60
        wanted = f'DEV_READY {value}'
        while True:
            try:
                observed, line = self.events.get(timeout=max(0.01, deadline - time.monotonic()))
            except queue.Empty:
                raise RuntimeError(f'Missing {wanted}: {self.lines}')
            if line == wanted:
                return (observed - started) * 1000
            if '[dev] Build failed' in line or time.monotonic() >= deadline:
                raise RuntimeError(f'Failed waiting for {wanted}: {self.lines}')

    def usage(self):
        # Supervisor only, after the one-shot program exits. CPU time has ps resolution.
        result = subprocess.check_output(['ps', '-o', 'time=', '-o', 'rss=', '-p', str(self.process.pid)], text=True).split()
        return {'cpuSeconds': cpu_seconds(result[0]), 'rssKiB': int(result[1])}

    def close(self):
        if self.process.poll() is None:
            self.process.send_signal(signal.SIGINT)
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(self.process.pid, signal.SIGKILL)
                self.process.wait(timeout=5)
                raise RuntimeError('dev did not stop within five seconds')
        self.reader.join(timeout=2)
        if self.process.returncode != 0:
            raise RuntimeError(f'dev exit {self.process.returncode}: {self.lines}')


def sample(compiler, count, idle):
    with tempfile.TemporaryDirectory(prefix='tsrust-dev-baseline-') as directory:
        root = pathlib.Path(directory)
        project(root, count)
        sources = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(root.iterdir())}
        rows = {}
        cold = Dev(compiler, root)
        try:
            rows['coldStartMs'] = cold.ready(expected(count, 1, 0), cold.started)
        finally:
            cold.close()
        warm = Dev(compiler, root)
        try:
            rows['warmStartMs'] = warm.ready(expected(count, 1, 0), warm.started)
            start = time.monotonic()
            (root / 'module0.tsr').write_text(leaf(0, 10))
            rows['leafEditMs'] = warm.ready(expected(count, 1, 10), start)
            start = time.monotonic()
            (root / 'shared.tsr').write_text('export function shared():int{return 2;}\n')
            rows['sharedEditMs'] = warm.ready(expected(count, 2, 10), start)
            config = json.loads((root / 'tsrust.config.json').read_text())
            config['build']['output'] = 'target/changed'
            start = time.monotonic()
            (root / 'tsrust.config.json').write_text(json.dumps(config))
            rows['configEditMs'] = warm.ready(expected(count, 2, 10), start)
            # Let exit bookkeeping settle before measuring an idle supervisor.
            time.sleep(0.25)
            before = warm.usage()
            start = time.monotonic()
            time.sleep(idle)
            after = warm.usage()
            rows['idle'] = {'elapsedSeconds': time.monotonic() - start, 'cpuSeconds': after['cpuSeconds'] - before['cpuSeconds'], 'rssKiBBefore': before['rssKiB'], 'rssKiBAfter': after['rssKiB']}
            rows['cacheBytes'] = cache_bytes(root)
        finally:
            warm.close()
        return {'modules': count + 2, 'sources': sources, 'observations': rows, 'coldLog': cold.lines, 'warmLog': warm.lines}


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--compiler', required=True)
    parser.add_argument('--samples', type=int, default=3)
    parser.add_argument('--modules', type=int, nargs='+', default=[4, 128])
    parser.add_argument('--idle-seconds', type=float, default=3)
    args = parser.parse_args()
    if args.samples < 1 or any(n < 1 for n in args.modules) or args.idle_seconds <= 0:
        parser.error('positive sample, module and idle values required')
    compiler = str(pathlib.Path(args.compiler).resolve(strict=True))
    result = {'protocol': 'dev-loop-v1', 'commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(), 'compilerSha256': hashlib.sha256(pathlib.Path(compiler).read_bytes()).hexdigest(), 'harnessSha256': hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(), 'host': platform.platform(), 'samplesPerShape': args.samples, 'idleSecondsPerSample': args.idle_seconds, 'rows': []}
    try:
        for count in args.modules:
            for repetition in range(args.samples):
                result['rows'].append({'repetition': repetition, **sample(compiler, count, args.idle_seconds)})
    finally:
        # Preserve completed samples even when a later sample fails.
        print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
