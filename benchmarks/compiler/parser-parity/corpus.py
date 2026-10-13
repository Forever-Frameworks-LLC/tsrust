"""Build a deterministic parser parity corpus from tracked sources, test literals,
grammar snippets, limit-boundary probes and seeded token mutations."""
import argparse
import hashlib
import random
import re
import subprocess
from pathlib import Path

KEYWORDS = ('function', '=>', 'type ', 'const ', 'let ', 'interface', 'enum ', 'match')
POOL = '( ) { } [ ] ; , : => = < > >> ? ?. . ... ! + - * ** ?? || && | function const let if else for while do return switch case default try catch finally match new in of as satisfies break continue throw enum type interface void typeof ` ${ 1 "s" name'.split()
TOKEN = re.compile(r'[A-Za-z_][A-Za-z0-9_]*|\d+|"(?:[^"\\\n]|\\.)*"|\S')


def literals(text):
    """Rust string literals: ordinary with escapes, and raw r#"..."# forms."""
    out = []
    for match in re.finditer(r'r(#*)"(.*?)"\1', text, re.S):
        out.append(match.group(2))
    for match in re.finditer(r'(?<![r#A-Za-z0-9_])"((?:[^"\\]|\\.)*)"', text, re.S):
        body = match.group(1)
        body = re.sub(r'\\\n\s*', '', body)
        body = body.replace('\\n', '\n').replace('\\t', '\t').replace('\\"', '"').replace("\\'", "'").replace('\\\\', '\\')
        out.append(body)
    return out


SNIPPET_STATEMENTS = '''
if (a) b(); else if (c) d(); else { e(); }
if (a) { b(); } else if (c) { d(); } else if (e) { f(); } else g();
while (i < 3) i += 1;
do { i++; } while (i < 3);
do i++; while (i < 3);
for (let i = 0, j = 1; i < 3; i++, j--) { continue; }
for (i = 0, j = 2; ; ) { break; }
for (;;) { break; }
for (const [a, b = 2, ...rest] of pairs) { use(a, b, rest); }
for (const { x, y: { z = 1 }, ...others } of items) use(x, z, others);
for (let item of items) item.run();
for (const key in record) { use(key); }
outer: inner: for (const a of b) { continue outer; }
label: { break label; }
label: ;
label: x = 1;
label: for (let i = 0; i < 2; i++) { break label; }
try { risky(); } catch (error) { handle(error); } finally { done(); }
try { risky(); } catch { handle(); }
try { risky(); } finally { done(); }
try { try { a(); } finally { b(); } } catch (e) { try { c(); } catch { d(); } }
switch (value) { case 1: case 2: a(); break; case 3: { b(); break; } default: c(); }
switch (f(x), y) { case g(1), 2: return; default: }
switch (value) { default: a(); break; case 1: b(); }
switch (a) { case 1: switch (b) { case 2: c(); break; default: d(); } break; }
match (value) { Ok(x) => { use(x); } Err(_) => { fail(); }, _ => { } }
match (f(a, b)) { Some(value) => { match (value) { A(b) => { use(b); } _ => {} } } }
const [a, [b, c = d(1)], { e, f: [g] }, ...h] = value;
const { a, b: { c, d = [1, 2] }, ...rest } = record;
let [x, , y]: int[] = values, z = 3, { w } = other;
let count: int, name: string;
let declared: int;
const direct = source, [p, q] = direct;
[a, b] = [b, a];
[a, [b, c], ...d] = value;
({ x, y: z, w: [u, v] } = record);
[a, b] = pair, c = 1;
function local<T>(value: T, [a, b]: int[], { c }: Rec, other: Mutable<int[]> = [], ...rest: int[]): T { return value; }
function outer(): int { function inner(): int { function deepest(): int { return 1; } return deepest(); } return inner(); }
const named = function fact(n: int): int { return n <= 1 ? 1 : n * fact(n - 1); };
const anonymous = function (a: int, b: int = a + 1): int { return a + b; }(1);
const arrow = (a: int, [b, c]: int[] = [1, 2], { d = (x: int): int => x + 1 }: Rec = defaults): int => a + b;
const curried = (a: int) => (b: int) => (c: int): int => { return a + b + c; };
const typed = (callback: Function<int, [value: ReadOnly<int[]>, other?: Mutable<Map<string, int[]>>]>): Once<Function<void>> => callback;
const generic = identity<Map<string, Array<int>>>(new Map<string, Array<int>>());
const set = new Set<int>([1, 2, ...more]), map = new Map<string, int>(), native = new Heap<int>(compare), bare = new Deque();
const chain = a?.b?.[c]?.(d)!.e!?.<int>(f);
const template = `a${b + `c${d ? `e${f}` : g}h`}i${j}`;
const assertion = (value as int | null) satisfies int | null;
const conditional = a ? b ? c : d : e ? f : g;
const assigned = a = b ??= c ||= d &&= e += f **= g;
const mixed = (a ?? b) || c && d | e ^ f & g === h < i << j + k * l ** m;
const sequence = (a, b, c);
const keyed = "x" in record;
const unary = !-+~typeof void ++x;
const update = x++ + --y;
const record = { a, b: 1, ...spread, c: { d: [1, { e: () => { return { f: 1 }; } }] } };
const call = f(...args, g(...more), h<int>(1))(2)[3].k;
const of = Array.of<int>(1, 2), plain = Array.of(1);
const nested = function () { return function () { return (() => { if (a) { return () => 1; } return () => 2; })(); }; };
const deferred = () => { for (const x of xs) { const f = () => { while (x) { return x; } }; } };
return;
return a, b;
throw new Error("x");
throw fail(), error;
break;
continue;
{ { { a(); } } }
if (a) if (b) c(); else d();
if (a) for (;;) while (b) do c(); while (d);
const f = (function g() { return 1; })();
if ((function cond() { return true; })()) { a(); b(); }
const h = (a: int = (function inner() { return 1; })()) => a;
const p = function ([a, b]: int[], g = function ([c]: int[]) { return c; }) { return a; };
const q = ([a, b]: int[], g = ([c]: int[]) => c) => a;
x = y
const broken = ;
const [ = 1;
if (a { b(); }
for (const of x) {}
const f = (a: ) => a;
const g = function (a: int { return a; };
switch (a) { case: b(); }
switch (a) { b(); }
try { a(); }
match (a) { Ok() => {} }
label: const x = 1;
const t = (a): int[] => a;
const u = (a) ? (b) : c;
const v = (a): Map<string, int>;
a < b > (c);
a<b>(c);
f(1 + ); g(2); h(3 + );
const w = { a: , b: 2 }; const ok = 1; return ok;
'''.strip().split('\n')

SNIPPET_PROGRAMS = '''
type Alias = int | null; type Other = Alias[]; function main() { const a: Other = []; }
type Box<T> = { value: T; next?: Box<T> | null; }; interface Named extends Base, Other { name: string; run: Function<int, [int]>, }
interface Base { id: int } interface Other { flag?: boolean; }
enum Color { Red, Green = 4, Blue = Green + 1, Mask = ~(1 << 2) | Color.Red } enum Text { A = "a", B = "b" + "c" }
let counter: int = 0; const table = [1, 2, 3], [first, second] = table; function main() { counter += first; }
function f<T, U>(a: T, b: U = a as U, [c, d]: int[] = [1, 2], ...rest: int[]): (value: T) => U { return (value: T): U => b; }
type A = B; type B = A; function main() {}
type Fn = (a: int, b?: string, ...rest: int[]) => (c: ReadOnly<int[]>) => void; function main() {}
type R = { f: Function<Box<Box<int>>, [x: Owned<Box<int>>, y?: Mutable<int[]>]>; g: Once<Function<void>>; h: (ReadOnly<Function<int>>) | null; };
function main() { const x: [int, string, [boolean, int[]][]] = [1, "a", []]; const y: "a" | 1 | -2 | 1.5 | true = "a"; }
test("registers", () => { expect(1).toBe(1); }); const shared = 1; function helper(): int { return shared; }
function a() { return 1; } function a() { return 2; } type T = int; type T = string;
function main() { const f = function named() {}; function named() {} }
function main( { }
type X = { a: int; b: ; };
interface I { m(): int; }
enum E { A = f() }
function main() { return 1 }
function g(a: int = 1, b: int) {} function h(...r: int[], x: int) {} function k(a?: int = 1) {}
'''.strip().split('\n')

SNIPPET_TYPES = '''
int
int[][]
(int | null)[]
Map<string, Array<Set<int>>>
Function<int, [a: int, b?: ReadOnly<string[]>, ...rest: Mutable<int[]>[]]>
Once<Function<void>> | null
ReadOnly<Function<int>>
Mutable<int[]>
Owned<Mutable<int>>
ReadOnly<ReadOnly<Function<int>>>
Once<int>
(a: int, b?: string, ...c: int[]) => (d: Owned<Box<int>>) => void
() => () => () => int | null
[int, [string, boolean], () => void]
"a" | 1 | -2 | 1.5 | true | false
Box<Box<Box<int>>>
Box<Box<int>>=value
Function<int, int>
Function<int, [int], int>
(int
[int, string
read int
Box<
(a: int, b: int = 1) => int
(a?: int, b: int) => int
(...a: int[], b: int) => int
Mutable<Function<int, [x: Mutable<Function<void>>]>>[]
(Mutable<Function<int>>) | undefined
'''.strip().split('\n')

SNIPPET_EXPRESSIONS = '''
a + b * c ** d ** e
(a, b) => a + b
(a: int, b: int = a): int => { return a + b; }
function (a: int) { return a; }(1)[0].x
function named(a: int) { return a; }
new Map<string, int>([["a", 1]]).get("a")!
a ? (b): c
a ? (b): int => c : d
x => y => z
[1, [2, [3, ...[4, 5]]], , ]
{ a: 1, ...b, c }
`x${`y${`z${1}`}`}`
a?.[b]?.(c)?.d
tag`x`
a ?? b || c
-2 ** 3
f<int>(1) < g<int, string>(2)
a < b
a < b >> (c)
((((1))))
'''.strip().split('\n')


def boundary_probes():
    out = {}
    wrap = lambda body: 'function main(){' + body + '}'
    for size in (31, 32, 33, 34, 100, 119, 120, 121, 126, 127, 128, 129, 130, 200):
        s = size
        out[f'parens-{s}'] = wrap('console.log(' + '(' * s + '7' + ')' * s + ');')
        out[f'blocks-{s}'] = wrap('{' * s + 'a();' + '}' * s)
        out[f'arrays-{s}'] = wrap('const v=' + '[' * s + '7' + ']' * s + ';')
        out[f'records-{s}'] = wrap('const v=' + '{a:' * s + '7' + '}' * s + ';')
        out[f'calls-{s}'] = wrap('f(' * s + '1' + ')' * s + ';')
        out[f'index-{s}'] = wrap('a[' * s + '0' + ']' * s + ';')
        out[f'fields-{s}'] = wrap('a' + '.b' * s + ';')
        out[f'postfix-calls-{s}'] = wrap('a' + '()' * s + ';')
        out[f'binary-{s}'] = wrap('x=' + '+'.join(['1'] * s) + ';')
        out[f'right-assoc-{s}'] = wrap('x=' + '**'.join(['2'] * s) + ';')
        out[f'assign-chain-{s}'] = wrap('a=' * s + '1;')
        out[f'conditional-{s}'] = wrap('x=' + 'a?b:' * s + 'c;')
        out[f'conditional-left-{s}'] = wrap('x=' + 'a?' * s + 'b' + ':c' * s + ';')
        out[f'unary-{s}'] = wrap('x=' + '!' * s + 'a;')
        out[f'unary-mixed-{s}'] = wrap('x=' + '-+~' * s + 'a;')
        out[f'update-{s}'] = wrap('++' * s + 'n;')
        out[f'templates-{s}'] = wrap('x=' + '`a${' * s + '1' + '}b`' * s + ';')
        out[f'closures-{s}'] = wrap('x=' + '(()=>' * s + '1' + ')' * s + ';')
        out[f'closure-blocks-{s}'] = wrap('const f=():void=>{' * s + '};' * s)
        out[f'closure-calls-{s}'] = wrap('x=' + '(():int=>{return ' * s + '7' + ';})()' * s + ';')
        out[f'function-expressions-{s}'] = wrap('x=' + 'function(){return ' * s + '1' + ';}' * s + ';')
        out[f'named-functions-{s}'] = wrap('function next():int{' * s + 'return 1;' + '}' * s)
        out[f'defaults-{s}'] = 'function use(value:int=' + 'id(' * s + '1' + ')' * s + '):int{return value;}'
        out[f'closure-defaults-{s}'] = wrap('x=' + '(a=' * s + '1' + ')=>a' * s + ';')
        out[f'if-{s}'] = wrap('if(a){' * s + '}' * s)
        out[f'if-braceless-{s}'] = wrap('if(a)' * s + 'b();')
        out[f'else-if-{s}'] = wrap('if(a){}' + 'else if(a){}' * s)
        out[f'else-nested-{s}'] = wrap('if(a){}else{' * s + '}' * s)
        out[f'while-{s}'] = wrap('while(a){' * s + '}' * s)
        out[f'do-{s}'] = wrap('do{' * s + '}while(a);' * s)
        out[f'for-{s}'] = wrap('for(;;){' * s + '}' * s)
        out[f'for-of-{s}'] = wrap('for(const n of [1]){' * s + '}' * s)
        out[f'try-{s}'] = wrap('try{' * s + 'throw 1;' + '}finally{}' * s)
        out[f'catch-{s}'] = wrap('try{}catch{' * s + '}' * s)
        out[f'switch-nested-{s}'] = wrap('switch(a){case 1:' * s + 'break;' + '}' * s)
        out[f'switch-clauses-{s}'] = wrap('switch(a){' + ''.join(f'case {i}:b();break;' for i in range(s)) + '}')
        out[f'match-nested-{s}'] = wrap('match(v){Ok(x)=>{' * s + '}}' * s)
        out[f'match-arms-{s}'] = wrap('match(v){' + ''.join(f'V{i}(x)=>{{}}' for i in range(s)) + '}')
        out[f'labels-{s}'] = wrap(''.join(f'l{i}:' for i in range(s)) + 'for(;;){}')
        out[f'pattern-array-{s}'] = wrap('const ' + '[' * s + 'a' + ']' * s + '=v;')
        out[f'pattern-record-{s}'] = wrap('const ' + '{a:' * s + 'b' + '}' * s + '=v;')
        out[f'pattern-rest-{s}'] = wrap('const ' + '[...' * s + 'a' + ']' * s + '=v;')
        out[f'pattern-param-{s}'] = wrap('const f=(' + '[' * s + 'a' + ']' * s + ':T)=>a;')
        out[f'pattern-assign-{s}'] = wrap('[' * s + 'a' + ']' * s + '=v;')
        out[f'pattern-assign-record-{s}'] = wrap('(' + '{a:' * s + 'b' + '}' * s + '=v);')
        out[f'type-params-{s}'] = 'function f<' + ','.join(f'T{i}' for i in range(s)) + '>(){}'
        out[f'type-args-{s}'] = wrap('f<' + ','.join(['int'] * s) + '>();')
        for name, text in (
            ('group', '(' * s + 'int' + ')' * s), ('applied', 'Box<' * s + 'int' + '>' * s),
            ('array', 'int' + '[]' * s), ('tuple', '[' * s + 'int' + ']' * s),
            ('arrow', '()=>' * s + 'int'), ('arrow-param', '(a:' * s + 'int' + ')=>int' * s),
            ('function', 'Function<' * s + 'int' + '>' * s), ('function-args', 'Function<int,[' * s + 'int' + ']>' * s),
            ('wrapper', 'ReadOnly<' * s + 'Function<int>' + '>' * s), ('union', '|'.join(['int'] * s)),
            ('group-array', '(' * s + 'int' + ')[]' * s), ('mixed', 'Box<[(' * s + 'int' + ')]>' * s),
        ):
            out[f'type-{name}-{s}.type'] = text
            out[f'type-{name}-decl-{s}'] = wrap('const v:' + text + '=x;')
            out[f'type-{name}-param-{s}'] = wrap('const f=(a:' + text + ')=>a;')
            out[f'type-{name}-assert-{s}'] = wrap('x=(v as ' + text + ');')
            out[f'type-{name}-alias-{s}'] = 'type A=' + text + ';function main(){const v:A=x;}'
            out[f'type-{name}-call-{s}'] = wrap('f<' + text + '>(1);')
    return out


def mutants(text, seed, count):
    tokens = [(m.start(), m.end()) for m in TOKEN.finditer(text)]
    if len(tokens) < 3:
        return []
    rng = random.Random(seed)
    out = []
    for _ in range(count):
        start, end = tokens[rng.randrange(len(tokens))]
        kind = rng.randrange(5)
        if kind == 0:
            out.append(text[:start] + text[end:])
        elif kind == 1:
            out.append(text[:end] + ' ' + text[start:end] + text[end:])
        elif kind == 2:
            out.append(text[:start] + rng.choice(POOL) + text[end:])
        elif kind == 3:
            out.append(text[:start] + rng.choice(POOL) + ' ' + text[start:])
        else:
            out.append(text[:start])
    return out


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--mutants', type=int, default=4)
    parser.add_argument('--max-bytes', type=int, default=40000)
    args = parser.parse_args()
    if args.output.exists():
        parser.error('Choose a new corpus directory')
    root = Path(subprocess.check_output(['git', 'rev-parse', '--show-toplevel'], text=True).strip())
    tracked = subprocess.check_output(['git', 'ls-files'], text=True, cwd=root).splitlines()
    bases = {}

    def add(group, name, text):
        if not text.strip() or len(text.encode()) > args.max_bytes:
            return
        digest = hashlib.sha256(text.encode()).hexdigest()[:16]
        bases.setdefault(digest, (group, name, text))

    for path in tracked:
        if path.endswith('.tsr'):
            try:
                add('tracked', path.replace('/', '__'), (root / path).read_text())
            except UnicodeDecodeError:
                pass
    for path in tracked:
        if path.endswith('.rs') and path.startswith('crates/') and '/syntax/parity_dump.rs' not in path:
            try:
                text = (root / path).read_text()
            except UnicodeDecodeError:
                continue
            for index, value in enumerate(literals(text)):
                if len(value) < 12 or not any(key in value for key in KEYWORDS):
                    continue
                add('literal', f'{path.replace("/", "__")}-{index}', value)
                if '{{' in value or '}}' in value:
                    add('literal', f'{path.replace("/", "__")}-{index}-format', value.replace('{{', '{').replace('}}', '}'))
    for index, line in enumerate(SNIPPET_STATEMENTS):
        add('snippet', f'statement-{index}', 'function main(){' + line + '}')
        add('snippet', f'statement-closure-{index}', 'function main(){const run=()=>{' + line + '};}')
        add('snippet', f'statement-nested-{index}', 'function main(){if(a){for(const q of r){try{' + line + '}finally{}}}}')
    for index, line in enumerate(SNIPPET_PROGRAMS):
        add('snippet', f'program-{index}', line)
    for index, line in enumerate(SNIPPET_TYPES):
        add('snippet', f'type-{index}', line)
        add('snippet', f'type-declared-{index}', 'function main(){const v:' + line + '=x;const f=(a:' + line + '):' + line + '=>a;}')
        add('snippet', f'type-record-{index}', 'type R={a:' + line + ';b?:' + line + ';};function f(a:' + line + '):' + line + '{}')
    for index, line in enumerate(SNIPPET_EXPRESSIONS):
        add('snippet', f'expression-{index}', line)
        add('snippet', f'expression-statement-{index}', 'function main(){const v=' + line + ';}')
    for name, text in boundary_probes().items():
        add('boundary', name, text)
    args.output.mkdir(parents=True)
    count = 0
    for digest, (group, name, text) in sorted(bases.items()):
        directory = args.output / group
        directory.mkdir(exist_ok=True)
        (directory / f'{digest}-{name[-80:]}').write_text(text)
        count += 1
        if group == 'boundary':
            continue
        for index, mutant in enumerate(mutants(text, digest, args.mutants)):
            directory = args.output / 'mutant'
            directory.mkdir(exist_ok=True)
            (directory / f'{digest}-{index}').write_text(mutant)
            count += 1
    print(count)


if __name__ == '__main__':
    main()
