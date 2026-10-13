"""Discover parser behavior geometrically on ordinary stacks.

Each case runs in a fresh compiler unit-test process without RUST_MIN_STACK.
Budgets bound this diagnostic run; they are not language limits. A timeout is
an unfinished observation. Results are not formal timing measurements."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import tempfile
import time


def nest(seed, layer, size):
    """Apply `layer` around `seed` `size` times in linear time."""
    before, after = layer('\0').split('\0')
    return before * size + seed + after * size


def main_body(body):
    return 'function identity(value:int):int{return value;}function main(){' + body + '}'


def valid(family, size):
    """Return (kind, source). Sources are syntactically valid."""
    n = size
    program = {
        'parentheses': lambda: main_body('console.log(' + '(' * n + '7' + ')' * n + ');'),
        'blocks': lambda: main_body('{' * n + 'console.log(7);' + '}' * n),
        'arrays': lambda: main_body('const v=' + '[' * n + '7' + ']' * n + ';'),
        'records': lambda: main_body('const v=' + '{a:' * n + '7' + '}' * n + ';'),
        'calls': lambda: main_body('console.log(' + 'identity(' * n + '7' + ')' * n + ');'),
        'indexes': lambda: main_body('const a=[0];console.log(' + 'a[' * n + '0' + ']' * n + ');'),
        'templates': lambda: main_body('const v=' + '`a${' * n + '7' + '}b`' * n + ';'),
        'binary-chain': lambda: main_body('const v=' + '+'.join(['1'] * n) + ';'),
        'exponent-chain': lambda: main_body('const v=' + '**'.join(['2'] * n) + ';'),
        'assignment-chain': lambda: main_body('let a=0;' + 'a=' * n + '1;'),
        'conditional-chain': lambda: main_body('const v=' + 'a?b:' * n + 'c;'),
        'conditional-nested': lambda: main_body('const v=' + 'a?' * n + 'b' + ':c' * n + ';'),
        'unary-chain': lambda: main_body('const v=' + '!' * n + 'a;'),
        'field-chain': lambda: main_body('const v=a' + '.b' * n + ';'),
        'call-chain': lambda: main_body('const v=a' + '()' * n + ';'),
        'index-chain': lambda: main_body('const v=a' + '[0]' * n + ';'),
        'optional-chain': lambda: main_body('const v=a' + '?.b' * n + ';'),
        'comma-chain': lambda: main_body('const v=(' + ','.join(['a'] * n) + ');'),
        'arrow-chain': lambda: main_body('const v=' + 'a=>' * n + '1;'),
        'closure-bodies': lambda: main_body('console.log(' + nest('7', lambda v: '(():int=>{return ' + v + ';})()', n) + ');'),
        'function-expressions': lambda: main_body('const v=' + nest('1', lambda v: 'function(){return ' + v + ';}', n) + ';'),
        'named-function-expressions': lambda: main_body('const v=' + nest('1', lambda v: '(function f(){return ' + v + ';})()', n) + ';'),
        'closure-defaults': lambda: main_body('const v=' + nest('1', lambda v: '((a:int=' + v + '):int=>a)()', n) + ';'),
        'named-functions': lambda: main_body('function next():int{' * n + 'return 1;' + '}' * n),
        'named-defaults': lambda: 'function use(value:int=' + 'identity(' * n + '1' + ')' * n + '):int{return value;}function identity(v:int):int{return v;}function main(){}',
        'if-nested': lambda: main_body('if(a){' * n + '}' * n),
        'if-braceless': lambda: main_body('if(a)' * n + 'b();'),
        'else-if-chain': lambda: main_body('if(a){}' + 'else if(a){}' * n),
        'loops-nested': lambda: main_body('for(const x of xs){while(a){do{for(;;){' * n + '}}while(a);}}' * n),
        'try-nested': lambda: main_body('try{' * n + 'throw 1;' + '}catch(e){}finally{}' * n),
        'switch-nested': lambda: main_body('switch(a){case 1:' * n + 'break;' + '}' * n),
        'switch-clauses': lambda: main_body('switch(a){' + ''.join(f'case {i}:b();break;' for i in range(n)) + '}'),
        'match-nested': lambda: main_body('match(v){Ok(x)=>{' * n + '}}' * n),
        'match-arms': lambda: main_body('match(v){' + ''.join(f'V{i}(x)=>{{}}' for i in range(n)) + '}'),
        'labels-nested': lambda: main_body(''.join(f'l{i}:{{' for i in range(n)) + '}' * n),
        'pattern-arrays': lambda: main_body('const ' + '[' * n + 'a' + ']' * n + '=v;'),
        'pattern-records': lambda: main_body('const ' + '{a:' * n + 'b' + '}' * n + '=v;'),
        'pattern-defaults': lambda: main_body('const ' + nest('a', lambda v: '[' + v + '=[1],b=2]', n) + '=v;'),
        'pattern-parameters': lambda: main_body('const f=(' + '[' * n + 'a' + ']' * n + ':T)=>a;'),
        'pattern-assignment': lambda: main_body('[' * n + 'a' + ']' * n + '=v;'),
        'statements-many': lambda: main_body('a=b+1;' * n),
        'alias-chain': lambda: 'type A0=int;' + ''.join(f'type A{i + 1}=A{i}[];' for i in range(n)) + f'function main(){{const v:A{n}=x;}}',
        'interface-chain': lambda: 'interface I0{f0:int}' + ''.join(f'interface I{i + 1} extends I{i}{{f{i + 1}:int}}' for i in range(n)) + 'function main(){}',
        'enum-initializer': lambda: 'enum E{A=' + '(' * n + '1' + ')' * n + ',B=' + '+'.join(['1'] * n) + ',C=' + '-' * 1 + '~' * n + '1}function main(){}',
        'generic-closers-spaced': lambda: main_body(''.join('const v' + str(i) + ': Map<string, Array<Box<int>>> = f((a: Box<Box<int>>): Box<Box<int>> => a);' for i in range(n))),
        'generic-closers': lambda: main_body(''.join('const v' + str(i) + ':Map<string,Array<Box<int>>>=f((a:Box<Box<int>>):Box<Box<int>>=>a);' for i in range(n))),
        'declarators-many': lambda: main_body('const ' + ','.join(f'v{i}=[{i}]' for i in range(n)) + ';'),
        'declarators-uninitialized': lambda: main_body('let ' + ','.join(f'v{i}:int' for i in range(n)) + ';'),
        'parameters-required': lambda: 'function many(' + ','.join(f'v{i}:int' for i in range(n)) + '):int{return 1;}',
        'parameters-closure': lambda: main_body('const f=(' + ','.join(f'v{i}:int' for i in range(n)) + '):int=>1;'),
        # Expressions inside defaults inside closures inside control-flow bodies.
        'defaults-closures-control': lambda: main_body('console.log(' + nest('7', lambda v: '((p:int=' + v + '):int=>{if(p>0){for(const q of [p]){try{return q;}finally{}}}return p;})()', n) + ');'),
        # Annotations interleaved with generic arguments and defaults.
        'annotations-generics-defaults': lambda: main_body('console.log(' + nest('7', lambda v: 'apply<Box<Array<int>>,Function<int,[v:ReadOnly<int[]>]>>(((x:Function<int,[v:ReadOnly<int[]>]>=(v:ReadOnly<int[]>):int=>' + v + '):int=>x([1]))() as int)', n) + ');'),
        # Templates, records, arrays, calls and indexes together.
        'literals-mixed': lambda: main_body('console.log(' + nest('7', lambda v: 'identity({k:[`a${' + v + '}b`,1]}.k[0].length)', n) + ');'),
        'statements-mixed': lambda: main_body(nest('console.log(7);', lambda v: 'if(a){for(let i=0;i<1;i++){while(a){do{switch(a){case 1:{try{outer:{' + v + '}}catch(e){}finally{}break;}default:}}while(a);}}}else if(b){}else{}', n)),
        'everything-mixed': lambda: main_body(nest('return 7;', lambda v: 'const f=function([a,{b=((c:Map<string,Array<int>>=new Map<string,Array<int>>()):int=>{' + v + '})()}]:T,...r:int[]):int{function g<U>(u:U=`x${[r[0]?.k!,a as int]}`):U{return u;}return g(1);};', n)),
    }
    types = {
        'type-groups': lambda: '(' * n + 'int' + ')' * n,
        'type-applied': lambda: 'Box<' * n + 'int' + '>' * n,
        'type-arrays': lambda: 'int' + '[]' * n,
        'type-tuples': lambda: '[' * n + 'int' + ']' * n,
        'type-arrows': lambda: '()=>' * n + 'int',
        'type-arrow-parameters': lambda: '(a:' * n + 'int' + ')=>int' * n,
        'type-functions': lambda: 'Function<' * n + 'int' + '>' * n,
        'type-function-arguments': lambda: 'Function<int,[' * n + 'int' + ']>' * n,
        'type-union-width': lambda: '|'.join(['int'] * n),
        'type-mixed': lambda: nest('int', lambda v: 'Function<Box<' + v + '[]>|null,[a:ReadOnly<Map<string,int>[]>,b?:(c:int)=>[int,string]|null]>', n),
    }
    if family in program:
        return 'program', program[family]()
    return 'type', types[family]()


def malformed(family, size):
    n = size
    cases = {
        # One error at the deepest point of each suspended scope kind.
        'deep-missing-operand': lambda: main_body('console.log(' + nest('1 + ', lambda v: 'identity(({value:[' + v + ']}).value[0])', n) + ');'),
        'deep-closure-error': lambda: main_body('console.log(' + nest('return +;', lambda v: '(():int=>{' + v + ' return 1;})()', n) + ');'),
        'deep-default-error': lambda: main_body('const v=' + nest('1 +', lambda v: '((a:int=' + v + '):int=>a)()', n) + ';'),
        'deep-pattern-error': lambda: main_body('const ' + '[' * n + 'a b' + ']' * n + '=v;'),
        'deep-type-error': lambda: main_body('const v:' + 'Box<(' * n + 'int int' + ')>' * n + '=x;'),
        # Every nested block holds one malformed statement before the next block.
        'errors-every-block': lambda: main_body(nest('ok();', lambda v: '{bad(;' + v + 'worse +;}', n)),
        'errors-every-closure': lambda: main_body(nest('ok();', lambda v: 'const f=()=>{bad(;' + v + '};', n)),
        'unclosed-blocks': lambda: main_body('{' * n + 'ok();'),
        'unclosed-parentheses': lambda: main_body('console.log(' + '(' * n + '7'),
        'unclosed-mixed': lambda: main_body(nest('ok(', lambda v: 'if(a){const f=(x:int=[{k:`t${' + v, n)),
        'extra-closers': lambda: main_body('{' * n + 'ok();' + '}' * (2 * n)),
    }
    return 'program', cases[family]()


VALID = ['parentheses', 'blocks', 'arrays', 'records', 'calls', 'indexes', 'templates', 'binary-chain', 'exponent-chain',
         'assignment-chain', 'conditional-chain', 'conditional-nested', 'unary-chain', 'field-chain', 'call-chain',
         'index-chain', 'optional-chain', 'comma-chain', 'arrow-chain', 'closure-bodies', 'function-expressions',
         'named-function-expressions', 'closure-defaults', 'named-functions', 'named-defaults', 'if-nested', 'if-braceless',
         'else-if-chain', 'loops-nested', 'try-nested', 'switch-nested', 'switch-clauses', 'match-nested', 'match-arms',
         'labels-nested', 'pattern-arrays', 'pattern-records', 'pattern-defaults', 'pattern-parameters', 'pattern-assignment',
         'statements-many', 'declarators-many', 'declarators-uninitialized', 'parameters-required', 'parameters-closure', 'alias-chain', 'interface-chain', 'enum-initializer', 'generic-closers', 'generic-closers-spaced', 'defaults-closures-control', 'annotations-generics-defaults', 'literals-mixed',
         'statements-mixed', 'everything-mixed', 'type-groups', 'type-applied', 'type-arrays', 'type-tuples', 'type-arrows',
         'type-arrow-parameters', 'type-functions', 'type-function-arguments', 'type-union-width', 'type-mixed']
MALFORMED = ['deep-missing-operand', 'deep-closure-error', 'deep-default-error', 'deep-pattern-error', 'deep-type-error',
             'errors-every-block', 'errors-every-closure', 'unclosed-blocks', 'unclosed-parentheses', 'unclosed-mixed', 'extra-closers']


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--probe', type=Path, required=True, help='Built compiler unit-test executable')
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--families', nargs='+', choices=VALID + MALFORMED, default=VALID + MALFORMED)
    parser.add_argument('--start', type=int, default=64)
    parser.add_argument('--factor', type=int, default=4)
    parser.add_argument('--max-size', type=int, default=1 << 20)
    parser.add_argument('--stages', default='core,recovery,format,bounded', help='Probe stages after lexing: core, lowering, recovery, format, bounded')
    parser.add_argument('--seconds-per-case', type=float, required=True)
    parser.add_argument('--seconds-total', type=float, required=True)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Preserve existing evidence; choose a new output path')
    probe = args.probe.resolve(strict=True)
    root = Path(__file__).resolve().parents[3]
    record = {'scope': 'parser diagnostic discovery, not formal timing or a support ceiling',
              'machine': platform.uname()._asdict(), 'cpu_count': os.cpu_count(),
              'source_commit': subprocess.check_output(['git', 'rev-parse', 'HEAD'], text=True, cwd=root).strip(),
              'source_diff_sha256': hashlib.sha256(subprocess.check_output(['git', 'diff', 'HEAD', '--', 'crates/tsrust-compiler'], cwd=root)).hexdigest(),
              'probe_sha256': hashlib.sha256(probe.read_bytes()).hexdigest(),
              'driver_sha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              'rust_min_stack': 'unset', 'stages': args.stages,
              'budgets': {'case_seconds': args.seconds_per_case, 'total_seconds': args.seconds_total, 'max_size': args.max_size},
              'cases': [], 'finished': False}
    try:
        record['physical_memory_bytes'] = int(subprocess.check_output(['sysctl', '-n', 'hw.memsize'], text=True, stderr=subprocess.PIPE))
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        record['physical_memory_bytes'] = None
        record['memory_observation_error'] = str(error)

    def save():
        args.output.write_text(json.dumps(record, indent=1) + '\n')
    save()
    deadline = time.monotonic() + args.seconds_total
    with tempfile.TemporaryDirectory(prefix='tsrust-parser-discovery-') as temporary:
        directory = Path(temporary)
        for family in args.families:
            size = args.start
            while time.monotonic() < deadline and size <= args.max_size:
                kind, text = (malformed if family in MALFORMED else valid)(family, size)
                path = directory / 'source.tsr'
                path.write_text(text)
                env = os.environ.copy()
                env.pop('RUST_MIN_STACK', None)
                env['TSRUST_PARSER_PROBE_SOURCE'] = str(path)
                env['TSRUST_PARSER_PROBE_KIND'] = kind
                env['TSRUST_PARSER_PROBE_STAGES'] = args.stages
                command = [str(probe), 'syntax::parser_probe::observe_parser_stages', '--exact', '--ignored', '--nocapture', '--test-threads=1']
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
                        time.sleep(0.01)
                    returncode = os.waitstatus_to_exitcode(status)
                output = (directory / 'output.log').read_text()
                events = [json.loads(line.split('TSRUST_PARSER_EVENT ', 1)[1]) for line in output.splitlines() if 'TSRUST_PARSER_EVENT ' in line]
                phases = {}
                for item in events:
                    phases.setdefault(item['phase'], {})[item['status']] = item
                durations = {name: (stages['finish']['elapsed_ns'] - stages['start']['elapsed_ns']) / 1e9
                             for name, stages in phases.items() if 'start' in stages and 'finish' in stages}
                outcome = {name: next(status for status in ('finish', 'diagnostic', 'start') if status in stages) for name, stages in phases.items()}
                unfinished = [name for name, status in outcome.items() if status == 'start']
                if timed_out:
                    classification = 'operational-timeout'
                elif returncode != 0:
                    classification = 'stack-overflow' if 'overflowed its stack' in output else 'crash'
                else:
                    classification = 'complete'
                result = {'family': family, 'size': size, 'kind': kind, 'source_bytes': len(text.encode()),
                          'source_sha256': hashlib.sha256(text.encode()).hexdigest(),
                          'classification': classification, 'unfinished_phase': unfinished[:1],
                          'outcome': outcome, 'durations_seconds': durations, 'events': events,
                          'output_tail': output[-600:] if classification != 'complete' else '',
                          'returncode': returncode, 'elapsed_seconds': time.monotonic() - started,
                          'peak_rss_bytes': usage.ru_maxrss * (1 if platform.system() == 'Darwin' else 1024)}
                record['cases'].append(result)
                save()
                key = 'type-parse' if kind == 'type' else 'parse'
                print(f'{family} {size}: {classification} {key}={outcome.get(key)} {durations.get(key, float("nan")):.3f}s '
                      f'bytes={result["source_bytes"]} rss={result["peak_rss_bytes"]}', flush=True)
                if classification != 'complete':
                    break
                size *= args.factor
    record['finished'] = True
    record['total_budget_exhausted'] = time.monotonic() >= deadline
    save()


if __name__ == '__main__':
    main()
