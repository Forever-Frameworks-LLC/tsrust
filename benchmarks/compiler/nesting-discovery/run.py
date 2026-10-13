"""Discover pipeline behavior geometrically; operational budgets are not language limits."""
import argparse
from collections import deque
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import time

FAMILIES = ('local-functions', 'mixed-local-scopes', 'assertions', 'satisfies', 'rest-arguments', 'array-of', 'mixed-wrappers', 'parentheses', 'blocks', 'binary', 'type-groups', 'arrays', 'generic-explicit', 'generic-inferred', 'closure-bodies', 'declared-arguments', 'bound-arguments', 'inline-arguments', 'field-arguments', 'mixed-literals', 'conditionals', 'mixed-branches', 'coalescing', 'mixed-coalescing', 'logical-values', 'logical-booleans', 'mixed-logical', 'assignment-values', 'compound-values', 'mixed-assignments', 'sequences', 'owned-store-sequences', 'if-statements', 'initializer-bodies', 'mixed-loops')


def repeated(seed, size, opening, closing):
    return opening * size + seed + closing * size


def source(family, size):
    if family == 'local-functions':
        return 'function probe():int{' + 'function nested():int{' * size + 'return 1;' + '}return nested();' * size + '}function main(){console.log(probe());}'
    if family == 'mixed-local-scopes':
        return 'function probe():int{' + '{const thunk=():int=>{function nested():int{' * size + 'return 1;' + '}return nested();};return thunk();}' * size + '}function main(){console.log(probe());}'
    if family in ('assertions', 'satisfies'):
        operator = 'as' if family == 'assertions' else 'satisfies'
        return 'function main(){console.log(7' + (' ' + operator + ' int') * size + ');}'
    if family in ('rest-arguments', 'array-of', 'mixed-wrappers'):
        opening, closing = {
            'rest-arguments': ('head(', ')'),
            'array-of': ('Array.of<int>(', ')[0]'),
            'mixed-wrappers': ('head(Array.of<int>((', ' as int))[0])'),
        }[family]
        return 'function head(...values:int[]):int{return values[0];}function main(){console.log(' + repeated('7', size, opening, closing) + ');}'
    if family == 'parentheses':
        body = 'console.log(' + '(' * size + '7' + ')' * size + ');'
    elif family == 'blocks':
        body = '{' * size + 'console.log(7);' + '}' * size
    elif family == 'binary':
        body = 'console.log(' + '+'.join(['1'] * size) + ');'
    elif family == 'type-groups':
        body = 'const value:' + '(' * size + 'int' + ')' * size + '=7;console.log(value);'
    elif family == 'closure-bodies':
        value = repeated('7', size, '(():int=>{return ', ';})()')
        body = 'console.log(' + value + ');'
    elif family == 'mixed-loops':
        layers = [('while(true){', 'break;}') if level % 3 == 0 else
                  ('for(const item' + str(level) + ' of [1]){', '}') if level % 3 == 1 else
                  ('do{', '}while(false);') for level in range(size)]
        body = ''.join(opening for opening, _ in reversed(layers)) + 'console.log(7);' + ''.join(closing for _, closing in layers)
    elif family == 'if-statements':
        body = repeated('console.log(7);', size, 'if(true){', '}else{}')
    elif family == 'initializer-bodies':
        value = repeated('7', size, '(():int=>{const value=', ';if(true){return value;}else{return 0;}})()')
        body = 'const result=' + value + ';console.log(result);'
    elif family in ('sequences', 'owned-store-sequences'):
        value = repeated('7', size, '(text="next",' if family == 'owned-store-sequences' else '(0,', ')')
        return 'function main(){let text="old";console.log(' + value + ');}'
    elif family in ('assignment-values', 'compound-values', 'mixed-assignments'):
        opening, closing = ('(target+=' if family == 'compound-values' else '(target='), ')'
        if family == 'mixed-assignments':
            opening, closing = '(values[0]+=(true?(null??(0||' + opening, closing + ')):0))'
        value = repeated('7', size, opening, closing)
        return 'function main(){let target=0;const values=[0];console.log(' + value + ');}'
    elif family in ('logical-values', 'logical-booleans', 'mixed-logical'):
        opening, closing = ('(true&&' if family == 'logical-booleans' else '(0||'), ')'
        if family == 'mixed-logical':
            opening, closing = 'identity(({value:[true?(null??' + opening, closing + '):0]}).value[0])'
        value = repeated('true' if family == 'logical-booleans' else '7', size, opening, closing)
        return 'function identity(value:int):int{return value;}function main(){console.log(' + value + ');}'
    elif family in ('coalescing', 'mixed-coalescing'):
        opening, closing = '(null??', ')'
        if family == 'mixed-coalescing':
            opening, closing = 'identity(({value:[true?' + opening, closing + ':0]}).value[0])'
        value = repeated('7', size, opening, closing)
        return 'function identity(value:int):int{return value;}function main(){console.log(' + value + ');}'
    elif family in ('conditionals', 'mixed-branches'):
        opening, closing = '(true?', ':0)'
        if family == 'mixed-branches':
            opening, closing = 'identity(({value:[' + opening, closing + ']}).value[0])'
        value = repeated('7', size, opening, closing)
        return 'function identity(value:int):int{return value;}function main(){console.log(' + value + ');}'
    elif family == 'mixed-literals':
        value = repeated('7', size, 'identity(({value:[', ']}).value[0])')
        return 'function identity(value:int):int{return value;}function main(){console.log(' + value + ');}'
    elif family in ('declared-arguments', 'bound-arguments', 'inline-arguments', 'field-arguments'):
        callee = {
            'declared-arguments': 'identity',
            'bound-arguments': 'action',
            'inline-arguments': '((value:int):int=>{return value;})',
            'field-arguments': 'holder.run',
        }[family]
        value = repeated('7', size, callee + '(', ')')
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
    parser.add_argument('--unbounded-parser', action='store_true', help='Bypass parser admission only; retain normal lowering, checking, emission and disposal')
    parser.add_argument('--breadth-first', action='store_true', help='Observe every family at one size before expanding successful families')
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
              'breadth_first': args.breadth_first,
              'parser_policy': 'unbounded' if args.unbounded_parser else 'public',
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
        pending = deque((family, args.start) for family in args.families)
        while pending and time.monotonic() < deadline:
            family, size = pending.popleft()
            text = source(family, size)
            path = directory / 'source.tsr'
            path.write_text(text)
            env = os.environ.copy()
            env.pop('RUST_MIN_STACK', None)
            env['TSRUST_NESTING_PROBE_SOURCE'] = str(path)
            env['TSRUST_NESTING_PROBE_POLICY'] = 'unbounded' if args.unbounded_parser else 'public'
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
            if complete:
                if args.breadth_first:
                    pending.append((family, size * args.factor))
                else:
                    pending.appendleft((family, size * args.factor))
        record['pending_cases'] = [{'family': family, 'size': size} for family, size in pending]
    record['finished'] = True
    record['total_budget_exhausted'] = time.monotonic() >= deadline
    save()


if __name__ == '__main__':
    main()
