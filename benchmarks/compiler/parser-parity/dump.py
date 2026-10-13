"""Run the opt-in parity dump to completion. A process death (for example a
stack overflow) is recorded as ABORT for the observation that was running,
then the dump resumes after it. Aborts are evidence; they are never dropped."""
import argparse
import os
import subprocess
from pathlib import Path


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--probe', type=Path, required=True, help='Compiler unit-test executable')
    parser.add_argument('--corpus', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--full', type=Path)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Preserve existing evidence; choose a new output path')
    skip = args.output.with_suffix('.aborts')
    marker = args.output.with_suffix('.running')
    skip.write_text('')
    env = os.environ.copy()
    env.pop('RUST_MIN_STACK', None)
    env.update(TSRUST_PARITY_CORPUS=str(args.corpus), TSRUST_PARITY_OUT=str(args.output), TSRUST_PARITY_SKIP=str(skip))
    if args.full:
        args.full.mkdir(parents=True, exist_ok=True)
        env['TSRUST_PARITY_FULL'] = str(args.full)
    command = [str(args.probe), 'syntax::parity_dump::dump_parser_parity', '--exact', '--ignored', '--test-threads=1']
    while True:
        result = subprocess.run(command, env=env, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
        if result.returncode == 0 and not marker.exists():
            break
        if not marker.exists():
            raise SystemExit(f'Dump failed without a marker:\n{result.stdout[-2000:]}')
        key = marker.read_text()
        marker.unlink()
        reason = 'stack overflow' if 'overflowed its stack' in result.stdout else f'exit {result.returncode}'
        with skip.open('a') as out:
            out.write(key + '\n')
        print(f'ABORT {key!r}: {reason}', flush=True)
    print(f'complete: {sum(1 for _ in args.output.open())} files, {sum(1 for _ in skip.open())} aborts')


if __name__ == '__main__':
    main()
