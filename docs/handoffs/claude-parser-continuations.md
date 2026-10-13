# Parser and type-syntax continuations: handoff to integration

Scope: `crates/tsrust-compiler/src/syntax/` and `crates/tsrust-compiler/src/types/syntax.rs`.
Branch `codex/claude-parser-continuations`, based on `2f96cc92`. Worktree
`/Users/seancotter/Desktop/dev/learning/tsrust-claude-parser`. Nothing in the
original checkout was edited. This document is the parser owner's report. It
does not accept the compiler nesting work, S12–S21 or any formal measurement.

## Commits

| Order | Commit | Content |
| --- | --- | --- |
| 1 | `e88cfef8` | One scheduler for the whole grammar, annotation work frames, admission policy, worklists, linear scans, witnesses, parity and discovery tools |
| 2 | `528b1397` | Packed generic closers rewritten once per annotation; wrapped Function types moved, not cloned; 256 KiB stack witness |
| 3 | `feff5b5c` | Hidden declarations and parameter patterns stay with their own scope (two defect repairs) |
| 4 | `c6020a2b` | Whole packed closers stay one token; explicit type argument lists read through their `>` |
| 5 | this commit | This handoff |

## Architecture

### One scheduler

`syntax/scheduler.rs` owns one heap work stack (`Machine::work`) and typed
result stacks: expressions, statements, statement lists, parameter lists,
functions and match statements. `Parser::run` is the only driver. It asserts
that it is not already active, so a grammar step that tried to start a second
driver fails immediately in every test.

A grammar step never waits for a child on the native stack. It pushes its
continuation frame, then the child request, and returns. The continuation
holds the parent's partial construction: operands, clause lists, spans, the
closure header, the pattern frame, the declaration frame. Token position,
pending declarations and pattern scopes stay in `Parser`.

| File | Productions |
| --- | --- |
| `expression_work.rs` | Precedence, unary, grouping, comma, postfix, calls, constructors (`new`), indexes, arrays, records, templates, conditionals, arrow and function primaries |
| `statement_work.rs` | Blocks, braceless bodies, labels, `if`/`else if`, `while`, `do`, `try`, `for`, `for...of`/`in`, `switch` with its lowering, `match`, jumps |
| `declaration_work.rs` | Declarators and declaration lists, binding patterns with defaults and rest, parameter lists with defaults, named functions, named function expressions, closure completion |
| `scheduler.rs` | Frames, result stacks, driver, unwinding and statement recovery, entry points |

Removed: the recursive `block`, `body`, `block_statement`, every control
statement function, `statement`, the declarator and pattern functions,
`callable_parameters`, `closure_signature`, `function_primary`, the second
recursive postfix chain (`postfix`, `call_step`, `index_step`, …) and
`set_syntax.rs`.

### Recovery and scopes

An error pops suspended frames up to the nearest `BlockAfter` frame. That
frame holds the token position and the extents of every parser and result
stack at the start of its statement. Recovery truncates to those extents,
resynchronizes with the unchanged `recover_statement`, and continues the same
block. Frames that had replaced parser state restore it when they are popped:
closure value bodies, named function expressions, blocks, braceless bodies and
switch frames restore pending declarations. Recovery order, the 63-error cap
and every message are unchanged.

### Annotation parser

`types/syntax.rs` keeps the same grammar on its own heap frames: unions,
tuples, groups, applications, arrow types, `Function<R, [A]>`, access wrappers
and array suffixes. The parameter-access state (`access_permitted`, `access`,
`wrapper_end`) keeps its original order of updates. Annotations cannot contain
expressions, so this machine is a leaf of the scheduler: it never calls back
into it.

A packed closer such as `>>` or `>>=` is read in place while the annotation is
parsed. A closer that is consumed whole stays one token. Only a closer that
the annotation ends inside (`x: Box<int>= 1`, `(a): Box<int>=>a`) is rewritten
as its separate `>` tokens and its rest, so the caller continues at an
ordinary token. Explicit type argument lists are read through their closing
`>` here, so `f<Box<Box<int>>>(x)` inserts no token.

### Worklists outside the scheduler

Enum initializer evaluation, destructuring assignment lowering, switch clause
validation (`end_switch_clause`, `switch_break`), the test registration return
check and `replace_pattern_source` are loops over explicit stacks. Source
order of hidden pattern names and of the first reported error is preserved.

### Admission policy

`syntax/nesting_policy.rs` holds every nesting bound.

- `NestingPolicy::CONSUMERS` (128) is used by every existing entry point:
  `parse`, `parse_module`, `diagnostics`, `lex`, `type_syntax::parse`,
  `format`. Messages, codes and spans of every limit diagnostic are unchanged.
- `NestingPolicy::UNBOUNDED` parses the same grammar without limits:
  `syntax::parse_unbounded`, `syntax::diagnostics_unbounded`, `lex_with`,
  `type_syntax::parse_with`. Only tests and probes use it today.

The parser itself needs no bound. The bound stays at the public boundary
because later consumers of the tree still recurse (see integration
dependencies). Retiring a limit is now one decision at one place, not a parser
change. Gated by the policy: expression, block, braceless body, `else if`,
type annotation, template and unary nesting (128); match arm nesting; the
switch clause count (120, because clauses lower to a nested `if` chain);
destructuring pattern nesting (32). Not gated, because they are count limits
and not nesting: 32 labels, 128 type parameters, 128 explicit type arguments,
63 recovered errors.

Recommendation for integration: destructuring pattern nesting (32) and lexer
template nesting (128) no longer protect anything in the parser, and binding
patterns lower to flat statements. They can be retired first. They are
documented source limits, so the owner decides.

## Behavior changes

Everything else is byte-identical in the parity corpus (see evidence).

1. `return:` and `throw:` never parse as labels. Before, a function body
   accepted `return: ;` as a labeled empty statement and a closure body
   reported `Expected an expression.` Both now report the error.
2. The hidden declaration of a named function expression follows the
   statement that holds the expression. Before, a named function expression in
   an `if`, `while`, `for`, `for...of` or `switch` header, or in an argument
   that precedes a block-bodied callback, was declared inside that body or
   callback and failed with `Unknown function "name#expressionN"`.
3. Destructured parameters open the body of their own callable. Before, an
   anonymous `function` expression in a default took the pattern declarations
   of the enclosing parameter list (`Binding name "c" is already defined`), and
   a function declared inside a default's closure body did the same.

Changes 2 and 3 only affect programs that failed before.

## Integration dependencies

These are outside the parser. Each blocks retiring `CONSUMERS`.

1. **Checker and emitter recursion (Codex).** Fourteen integration suites
   abort or fail identically on `2f96cc92` and on this branch; the parser is
   not on the failing path for sources at or below the admission bound. See
   the suite table below.
2. **`callables/local_functions.rs` and `callables/module_state.rs`.** The
   public `parse` runs them after the parser. `Resolver::function`, `body`,
   `expression` and `remove_lifted_placeholders` recurse over the tree.
   `parse_unbounded` skips them. Observe with
   `parser.py --stages core,lowering`.
3. **Formatter (`formatting/`).** Token based and not recursive, but its
   bracket matching rescans: time grows quadratically with nesting depth for
   blocks, records, conditionals and patterns. It lexes and probes
   annotations with `CONSUMERS`.
4. **Derived `Debug`** for `Expr`, `Stmt` and `ValueAnnotation` recurses. The
   parity dump renders on a large stack for that reason. No compiler path
   depends on it.
5. **Token storage.** An annotation that ends inside a packed token
   (`x: Box<int>= 1`, `(a): Box<int>=>a`, written without a space) still
   inserts tokens into the token vector. Cost is one tail move per such
   annotation, so many of them in one large file grow quadratically (see
   `generic-closers`). The spaced spelling that the formatter produces does
   not insert. A lexer that emits single `>` tokens and lets the expression
   parser join adjacent ones would remove the rest; that changes the public
   `lex` stream and needs coordination with the formatter and the editor.
6. **AST node size.** `Expr` is 304 bytes and `Stmt` is 688 bytes. Peak memory
   per nesting level is dominated by the tree, not by parser frames (112 bytes
   per frame). Not a parser change.
7. **`types/aliases.rs`.** Alias resolution repeats a pass per chain link, and
   every alias stores its full expansion, so a chain of `n` aliases costs
   `n²`. Expansion also nests `Group` nodes past the annotation bound: a chain
   of 2,000 grouped aliases passes `check` today. Annotation consumers already
   accept that depth.

No shared interface changed. `parse_prefix` and `parse_parameter_prefix`
(`pub(crate)`) take the policy as a fourth argument.

## Validation

Machine: this Mac, 8 GiB memory, 8 CPUs. Toolchain `rustc 1.96.1`. Every build
used the private target `/private/tmp/tsrust-claude-parser-target` (owner
decision, 2026-10-13) with `CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=1`, serial
tests and no `RUST_MIN_STACK`. The baseline comparison built `2f96cc92` plus
the parity dump module in `/private/tmp/tsrust-claude-parser-baseline` with
its own target. All builds are unoptimized. No formal timing was run.

### Parity with `2f96cc92`

Corpus: 80,933 inputs (1,260 tracked `.tsr`, 14,196 test literals, 423 grammar
snippets, 1,666 limit-boundary probes, 63,388 seeded mutations). Nine
observations per input, each parsed on a 2 MiB thread.

| Revision | Compared with | Identical observations | Different |
| --- | --- | ---: | --- |
| `e88cfef8` | `2f96cc92` | 727,318 | 22, all from five mutants that contain `return :` or `throw :` |
| `528b1397` | `e88cfef8` | 728,397 | 0 |
| `feff5b5c` | `528b1397` | 728,375 | 22, all from the two grammar snippets that hold the repaired defects, and their mutants |
| `c6020a2b` | `feff5b5c` | 726,782 | 1,615 expression observations; full-text comparison shows that only the internal token cursor index differs |

The baseline overflowed its 2 MiB stack in 1,057 observations of 25 probe
families; every one completes now. Smallest sampled size that overflowed: 31
for closure blocks, closure defaults and function expressions; 100 for `for`,
`switch`, `match` and `Function<…>` types; 119 for arrow types and nested
`else`; 126 for `do`. Sampled sizes were 31–34, 100, 119–121, 126–130 and 200,
so a smaller true threshold is possible. These are debug-build observations
below the 128 guard. The new parser has no abort in any revision.

### Unit and integration checks

- Syntax and type-syntax unit tests at `c6020a2b`: 36 passed, 2 opt-in tools
  ignored. Twelve are new nesting witnesses in `syntax/nesting_tests.rs`. They
  include 35 grammar families at 10,000 levels on the ordinary stack, a mixed
  shape that keeps its expression count per layer, deep errors with exact
  locations and restored scopes, recovery inside every enclosing block, and
  one run of parse, clone, recovery and drop on a 256 KiB thread.
- Compiler integration suites at `feff5b5c`: 62 run, 48 passed. Type-related
  subset rerun at `c6020a2b`: 19 run, 16 passed.
- The 14 suites that do not pass have the same passing tests and the same
  failing test on `2f96cc92`. One fails an assertion
  (`analysis::explicit_limits_reject_deep_source_without_panicking`). Thirteen
  overflow the stack in a depth witness and leave their later tests unrun:
  `array_of`, `assignment_values`, `comma_contexts`,
  `compound_assignment_values`, `conditional_expressions`, `for_of`,
  `non_null_assertions`, `nullish_coalescing`, `operators`, `rest_parameters`,
  `templates`, `tuple_types`, `type_assertions`. They are not counted as
  passed. They are original failures in the checker and emitter.
- Editor suites at `c6020a2b`: 10 parse-related suites run, 10 passed.
- `node tools/format-project.mjs` before each of the four source commits:
  0 files formatted, 0 fingerprints updated.
- Not run: the complete compiler and LSP regressions, native benchmark
  workloads and any timing protocol.

### Geometric discovery

Parser stages under the unbounded policy, one fresh process per case, sizes
64 to 1,048,576 by factors of four. At `feff5b5c`: 72 families, 560 cases,
545 complete, 15 operational timeouts at the 60-second case budget, no stack
overflow and no crash. A timeout is an unfinished observation, not a limit.

| Family | Largest complete size | Source bytes | Parse s | Recovery s | Peak RSS MiB | Next size |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| parentheses | 1048576 | 2097231 | 4.25 | 4.18 | 514 | not run (size cap) |
| blocks | 1048576 | 2097231 | 2.67 | 3.34 | 1005 | not run (size cap) |
| arrays | 1048576 | 2097226 | 3.21 | 4.59 | 539 | not run (size cap) |
| records | 1048576 | 4194378 | 4.56 | 4.85 | 777 | not run (size cap) |
| calls | 1048576 | 10485839 | 4.31 | 4.94 | 1265 | not run (size cap) |
| indexes | 1048576 | 3145819 | 4.42 | 4.38 | 1091 | not run (size cap) |
| templates | 1048576 | 7340106 | 1.90 | 3.14 | 1409 | not run (size cap) |
| binary-chain | 1048576 | 2097224 | 2.92 | 3.41 | 610 | not run (size cap) |
| exponent-chain | 1048576 | 3145799 | 3.31 | 3.76 | 672 | not run (size cap) |
| assignment-chain | 1048576 | 2097226 | 2.95 | 3.43 | 924 | not run (size cap) |
| conditional-chain | 1048576 | 4194378 | 5.74 | 6.58 | 1465 | not run (size cap) |
| conditional-nested | 1048576 | 4194378 | 5.04 | 5.85 | 1154 | not run (size cap) |
| unary-chain | 1048576 | 1048650 | 1.09 | 1.24 | 595 | not run (size cap) |
| field-chain | 1048576 | 2097226 | 1.39 | 1.59 | 400 | not run (size cap) |
| call-chain | 1048576 | 2097226 | 2.31 | 2.44 | 518 | not run (size cap) |
| index-chain | 1048576 | 3145802 | 3.91 | 4.26 | 715 | not run (size cap) |
| optional-chain | 1048576 | 3145802 | 1.31 | 1.86 | 1073 | not run (size cap) |
| comma-chain | 1048576 | 2097226 | 3.12 | 3.56 | 1037 | not run (size cap) |
| arrow-chain | 1048576 | 3145802 | 3.33 | 4.16 | 1382 | not run (size cap) |
| closure-bodies | 262144 | 5767247 | 4.71 | 7.30 | 1184 | 1048576: operational-timeout in recovery |
| function-expressions | 1048576 | 20971594 | 7.83 | 9.03 | 1369 | not run (size cap) |
| named-function-expressions | 1048576 | 27263050 | 18.96 | 20.56 | 1085 | not run (size cap) |
| closure-defaults | 262144 | 4980810 | 5.00 | 5.45 | 1314 | 1048576: operational-timeout in recovery |
| named-functions | 1048576 | 22020169 | 9.63 | 9.57 | 1504 | not run (size cap) |
| named-defaults | 1048576 | 10485860 | 3.99 | 4.76 | 1251 | not run (size cap) |
| if-nested | 1048576 | 7340096 | 6.71 | 7.71 | 1398 | not run (size cap) |
| if-braceless | 1048576 | 5242948 | 4.73 | 6.02 | 648 | not run (size cap) |
| else-if-chain | 1048576 | 12582983 | 8.04 | 9.05 | 1167 | not run (size cap) |
| loops-nested | 262144 | 13631552 | 8.46 | 8.73 | 1490 | 1048576: operational-timeout in parse |
| try-nested | 1048576 | 25165896 | 9.92 | 11.87 | 1208 | not run (size cap) |
| switch-nested | 1048576 | 18874438 | 13.20 | 14.25 | 1481 | not run (size cap) |
| switch-clauses | 1048576 | 23006213 | 12.12 | 14.33 | 1635 | not run (size cap) |
| match-nested | 262144 | 4980800 | 3.59 | 5.04 | 1940 | 1048576: operational-timeout in clone |
| match-arms | 262144 | 3558980 | 1.74 | 2.83 | 1401 | 1048576: operational-timeout in clone |
| labels-nested | 1048576 | 10423290 | 4.73 | 5.27 | 957 | not run (size cap) |
| pattern-arrays | 1048576 | 2097226 | 4.72 | 4.62 | 1397 | not run (size cap) |
| pattern-records | 1048576 | 4194378 | 5.02 | 6.14 | 931 | not run (size cap) |
| pattern-defaults | 262144 | 2621514 | 5.23 | 5.51 | 781 | 1048576: operational-timeout in clone |
| pattern-parameters | 1048576 | 2097233 | 6.96 | 8.11 | 1101 | not run (size cap) |
| pattern-assignment | 1048576 | 2097220 | 6.92 | 5.93 | 1106 | not run (size cap) |
| statements-many | 1048576 | 6291520 | 9.32 | 14.93 | 1088 | not run (size cap) |
| declarators-many | 1048576 | 17700794 | 7.42 | 8.34 | 1170 | not run (size cap) |
| alias-chain | 4096 | 75652 | 6.67 | 8.16 | 940 | 16384: operational-timeout in parse |
| interface-chain | 1024 | 37673 | 1.18 | 1.20 | 277 | 4096: operational-timeout in parse |
| enum-initializer | 1048576 | 5242915 | 10.10 | 9.13 | 908 | not run (size cap) |
| generic-closers | 16384 | 1283290 | 12.80 | 12.89 | 246 | 65536: operational-timeout in parse |
| defaults-closures-control | 262144 | 21495887 | 14.37 | 22.12 | 881 | 1048576: operational-timeout in parse |
| annotations-generics-defaults | 4096 | 589903 | 2.07 | 2.10 | 162 | 16384: operational-timeout in recovery |
| literals-mixed | 262144 | 9699407 | 5.82 | 6.68 | 1504 | 1048576: operational-timeout in clone |
| statements-mixed | 65536 | 8912975 | 5.43 | 6.79 | 1250 | 262144: operational-timeout in dispose |
| everything-mixed | 4096 | 745545 | 2.92 | 2.88 | 266 | 16384: operational-timeout in recovery |
| type-groups | 1048576 | 2097155 | 2.50 | - | 302 | not run (size cap) |
| type-applied | 1048576 | 5242883 | 2.40 | - | 508 | not run (size cap) |
| type-arrays | 1048576 | 2097155 | 1.97 | - | 311 | not run (size cap) |
| type-tuples | 1048576 | 2097155 | 2.36 | - | 620 | not run (size cap) |
| type-arrows | 1048576 | 4194307 | 3.23 | - | 700 | not run (size cap) |
| type-arrow-parameters | 1048576 | 9437187 | 5.54 | - | 911 | not run (size cap) |
| type-functions | 1048576 | 10485763 | 2.60 | - | 491 | not run (size cap) |
| type-function-arguments | 1048576 | 16777219 | 7.27 | - | 898 | not run (size cap) |
| type-union-width | 1048576 | 4194303 | 1.50 | - | 328 | not run (size cap) |
| type-mixed | 262144 | 22020099 | 10.00 | - | 693 | 1048576: operational-timeout in dispose |
| deep-missing-operand | 1048576 | 32505938 | diagnostic | 17.65 | 1081 | not run (size cap) |
| deep-closure-error | 1048576 | 25165911 | diagnostic | 18.42 | 968 | not run (size cap) |
| deep-default-error | 1048576 | 19923020 | diagnostic | 17.37 | 946 | not run (size cap) |
| deep-pattern-error | 1048576 | 2097228 | diagnostic | 3.51 | 898 | not run (size cap) |
| deep-type-error | 1048576 | 7340114 | diagnostic | 5.14 | 346 | not run (size cap) |
| errors-every-block | 1048576 | 15728709 | diagnostic | 5.64 | 661 | not run (size cap) |
| errors-every-closure | 1048576 | 20971589 | diagnostic | 8.61 | 335 | not run (size cap) |
| unclosed-blocks | 1048576 | 1048645 | diagnostic | 1.37 | 335 | not run (size cap) |
| unclosed-parentheses | 1048576 | 1048653 | diagnostic | 2.58 | 787 | not run (size cap) |
| unclosed-mixed | 1048576 | 30408771 | diagnostic | 10.68 | 536 | not run (size cap) |
| extra-closers | 1048576 | 3145797 | diagnostic | 4.90 | 549 | not run (size cap) |

Parse time grows faster than input size in five families. `alias-chain` and
`interface-chain` are costs of alias resolution and of inherited field lists.
`generic-closers`, `annotations-generics-defaults` and `everything-mixed` are
token insertions for compact closers. The same families at `c6020a2b`:

| Family | Largest complete size | Source bytes | Parse s | Recovery s | Peak RSS MiB | Next size |
| --- | ---: | ---: | ---: | ---: | ---: | --- |
| generic-closers | 16384 | 1283290 | 8.73 | 9.20 | 230 | 65536: operational-timeout in parse |
| generic-closers-spaced | 262144 | 22957626 | 13.85 | 11.56 | 912 | 1048576: operational-timeout in parse |
| annotations-generics-defaults | 16384 | 2359375 | 14.90 | 14.23 | 493 | 65536: operational-timeout in parse |
| everything-mixed | 4096 | 745545 | 1.72 | 1.67 | 237 | 16384: operational-timeout in recovery |
| type-mixed | 262144 | 22020099 | 9.69 | - | 1182 | 1048576: operational-timeout in type-parse |

Peak RSS is for the whole probe process: the token vector, two trees during
clone and the test harness. It is not a parser high-water mark. The 1,048,576
cases run close to the physical memory of this machine, so their durations
include paging.

Consumers at `c6020a2b`, sizes to 65,536 with a 30-second case budget:

- Local function resolution overflows the 2 MiB stack at 256 nested named
  functions and at 256 nested named function expressions; 64 completes.
- The formatter completes 4,096 levels in 3 to 12 seconds and does not finish
  16,384 levels of blocks, records, conditionals, closures, `if`, `switch` or
  `match` in 30 seconds. Flat input of 65,536 statements formats in 1.4 s.
- The bounded public parse returns its `unsupported` diagnostic for every
  nested family, as before.

## Evidence

Closed records are outside Git, in
`benchmarks/runtime/results/claude-parser-evidence/` of this worktree (an
ignored path, 142 MB). Import them through the single SQLite writer before the
worktree is removed. `MANIFEST.sha256` lists 199 files; its own SHA-256 is
`dd5b752f8329cce86e37c6998e2ab7233070375567e52d2dcd4d38ba75b7e814`.

| Record | Content |
| --- | --- |
| `parity-corpus.tar.gz` | The 80,933 inputs, SHA-256 `20fb784809aa2f8ac76db65fff4e64e1f5eaa66435c8b5b82934b8d64ba9da78`, built by `benchmarks/compiler/parser-parity/corpus.py` at `2f96cc92` |
| `parity-baseline.*` | Baseline fingerprints, log and the 1,057 aborted observations |
| `parity-<commit>.*` | Fingerprints and logs for each commit; `compare.py` and `cursor-only.sh` reproduce the comparisons |
| `suites-<commit>/` | One log per integration suite and a summary that counts aborts; `abort-compare.py` compares per-test outcomes |
| `discovery-1-…` | First run during `e88cfef8` work, before the linear-scan repairs |
| `discovery-2-…` | Unfinished: the driver's own source generator was quadratic and stalled; repaired in `feff5b5c` |
| `discovery-3-feff5b5c-core.*` | The first table above |
| `discovery-5-c6020a2b-*` | Closer families and consumer stages at the final source commit |
| `PROBES.sha256` | Hashes of the unit-test executables and of the CLI used for formatting |

Commands:

```sh
export CARGO_TARGET_DIR=/private/tmp/tsrust-claude-parser-target CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1
cargo test -p tsrust-compiler --lib --no-run
python3 benchmarks/compiler/parser-parity/corpus.py --output CORPUS
python3 benchmarks/compiler/parser-parity/dump.py --probe TEST_EXE --corpus CORPUS --output NEW.tsv
python3 benchmarks/compiler/nesting-discovery/parser.py --probe TEST_EXE --output NEW.json \
  --stages core,recovery --seconds-per-case 60 --seconds-total 4200
cargo test -p tsrust-compiler --lib -- --test-threads=1 syntax:: types::syntax
cargo test -p tsrust-compiler --test SUITE -- --test-threads=1
TSRUST=$CARGO_TARGET_DIR/debug/tsrust node tools/format-project.mjs
```

## Cleanup after merge

The owner asked for cleanup after merging. Remove, in this order: the
evidence directory after its SQLite import; the worktrees
`tsrust-claude-parser` and `/private/tmp/tsrust-claude-parser-baseline`
(`git worktree remove`); the targets `/private/tmp/tsrust-claude-parser-target`
and `/private/tmp/tsrust-claude-parser-baseline-target`. The formatting runs
used a symlink to the original checkout's `benchmarks/runtime/node_modules`;
it is removed.
