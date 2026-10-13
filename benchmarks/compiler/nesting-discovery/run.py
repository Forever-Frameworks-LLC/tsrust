"""Discover pipeline behavior geometrically; operational budgets are not language limits."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import time

FAMILIES = ('parentheses', 'blocks', 'binary', 'type-groups', 'arrays', 'generic-explicit', 'generic-inferred', 'closure-bodies', 'declared-arguments', 'bound-arguments', 'inline-arguments', 'field-arguments')


def source(family, size):
    if family == 'parentheses':
        body = 'console.log(' + '(' * size + '7' + ')' * size + ');'
    elif family == 'blocks':
        body = '{' * size + 'console.log(7);' + '}' * size
    elif family == 'binary':
        body = 'console.log(' + '+'.join(['1'] * size) + ');'
    elif family == 'type-groups':
        body = 'const value:' + '(' * size + 'int' + ')' * size + '=7;console.log(value);'
    elif family == 'closure-bodies':
        value = '7'
        for _ in range(size):
            value = '(():int=>{return ' + value + ';})()'
        body = 'console.log(' + value + ');'
    elif family in ('declared-arguments', 'bound-arguments', 'inline-arguments', 'field-arguments'):
        callee = {
            'declared-arguments': 'identity',
            'bound-arguments': 'action',
            'inline-arguments': '((value:int):int=>{return value;})',
            'field-arguments': 'holder.run',
        }[family]
        value = '7'
        for _ in range(size):
            value = callee + '(' + value + ')'
        declarations = 'function identity(value:int):int{return value;}\n'
        if family == 'field-arguments':
            declarations += 'type Holder={run:ReadOnly<Function<int,[value:int]>>;};\n'
        setup = ''
        if family == 'bound-arguments':
            setup = 'const action=(value:int):int=>{return value;};'
        elif family == 'field-arguments':
            setup = 'const holder:Holder={run:(value:int):int=>{return value;}};'
        return declarations + 'function main(){' + setup + 'console.log(' + value + ');}'
    elif family == 'arrays':
        body = 'const value=' + '[' * size + '7' + ']' * size + ';console.log(value.length);'
    else:
        explicit = family == 'generic-explicit'
        functions = []
        for index in range(size):
            value = 'value' if index + 1 == size else f'step{index+1}' + ('<T>' if explicit else '') + '(value)'
            functions.append(f'function step{index}<T>(value:T):T{{return {value};}}')
        return '\n'.join(functions + ['function main(){console.log(step0' + ('<int>' if explicit else '') + '(7));}'])
    return 'function main(){' + body + '}'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--probe', type=Path, required=True, help='Built compiler unit-test executable')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--families', nargs='+', choices=FAMILIES, default=list(FAMILIES))
    parser.add_argument('--start', type=int, default=8)
    parser.add_argument('--factor', type=int, default=2)
    parser.add_argument('--seconds-per-case', type=float, required=True)
    parser.add_argument('--seconds-total', type=float, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Preserve existing evidence; choose a new output path')
    if args.start < 1 or args.factor < 2 or args.seconds_per_case <= 0 or args.seconds_total <= 0:
        parser.error('Require positive sizes/budgets and factor >= 2')
    probe = args.probe.resolve(strict=True)
    record = {'scope': 'diagnostic discovery, not formal timing or a support ceiling',
              'machine': platform.uname()._asdict(), 'cpu_count': os.cpu_count(),
              'source_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True).strip(),
              'source_diff': subprocess.check_output(['git', 'diff', '--', 'crates/tsrust-compiler'], text=True),
              'probe_sha256': hashlib.sha256(probe.read_bytes()).hexdigest(),
              'driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'budgets': {'case_seconds': args.seconds_per_case, 'total_seconds': args.seconds_total},
              'cases': [], 'finished': False}
    try:
        record['physical_memory_bytes'] = (int(subprocess.check_output(['sysctl', '-n', 'hw.memsize'], text=True, stderr=subprocess.PIPE)) if platform.system() == 'Darwin' else os.sysconf('SC_PHYS_PAGES') * os.sysconf('SC_PAGE_SIZE'))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        record['physical_memory_bytes'] = None
        record['memory_observation_error'] = str(error)
    record['probe_source'] = Path('crates/tsrust-compiler/src/nesting_probe.rs').read_text()
    def save():
        args.output.write_text(json.dumps(record, indent=2) + '\n')
    save()
    deadline = time.monotonic() + args.seconds_total
    with tempfile.TemporaryDirectory(prefix='tsrust-nesting-discovery-') as temporary:
        directory = Path(temporary)
        for family in args.families:
            size = args.start
            while time.monotonic() < deadline:
                text = source(family, size)
                path = directory / 'source.tsr'
                path.write_text(text)
                env = os.environ.copy()
                env.pop('RUST_MIN_STACK', None)
                env['TSRUST_NESTING_PROBE_SOURCE'] = str(path)
                command = [str(probe), 'nesting_probe::observe_pipeline_stages', '--exact', '--ignored', '--nocapture', '--test-threads=1']
                started = time.monotonic()
                case_deadline = min(deadline, started + args.seconds_per_case)
                timed_out = False
                with (directory / 'output.log').open('w') as output:
                    child = subprocess.Popen(command, env=env, stdout=output, stderr=subprocess.STDOUT)
                    while True:
                        pid, status, usage = os.wait4(child.pid, os.WNOHANG)
                        if pid:
                            break
                        if time.monotonic() >= case_deadline:
                            timed_out = True
                            child.kill()
                            _, status, usage = os.wait4(child.pid, 0)
                            break
                        time.sleep(0.02)
                    child.returncode = os.waitstatus_to_exitcode(status)
                output = (directory / 'output.log').read_text()
                events = [json.loads(line.split('TSRUST_NESTING_EVENT ', 1)[1]) for line in output.splitlines() if 'TSRUST_NESTING_EVENT ' in line]
                complete = child.returncode == 0 and any(e['phase'] == 'dispose' and e['status'] == 'finish' for e in events)
                result = {'family': family, 'size': size, 'source': text, 'source_sha256': hashlib.sha256(text.encode()).hexdigest(),
                          'events': events, 'output': output, 'returncode': child.returncode,
                          'elapsed_seconds': time.monotonic() - started, 'operational_timeout': timed_out,
                          'peak_rss_bytes': usage.ru_maxrss * (1 if platform.system() == 'Darwin' else 1024),
                          'pipeline_complete': complete}
                record['cases'].append(result)
                save()
                print(f'{family} {size}: complete={complete} timeout={timed_out} rss={result["peak_rss_bytes"]}', flush=True)
                if not complete:
                    break
                size *= args.factor
    record['finished'] = True
    record['total_budget_exhausted'] = time.monotonic() >= deadline
    save()


if __name__ == '__main__':
    main()
