---
title: Run project tests
description: Write a native test, select files and interpret failures and captured output.
---

Use `std:testing` to register native tests and assert their results. Test execution
requires the [prepared Rust tools, sources and linker](../rust-setup/).

## Write and run a test

Save this as `greeting.test.tsr`:

```tsrust
import { test, strictEqual } from 'std:testing';

test('joins a greeting', () => {
  strictEqual(['Hello', 'Ada'].join(' '), 'Hello Ada');
});
```

Run it with:

```sh
tsrust test greeting.test.tsr
```

The report contains one passing test named `joins a greeting`. Change the expected
text to `Hello Bob` to see an equality failure with expected and received values.
`strictEqual` compares objects by identity; two objects with identical printed
contents are not necessarily equal.

## Compare data and asynchronous failures

Use `deepStrictEqual` and `notDeepStrictEqual` for structural data rather than
identity. They borrow both inputs and compare values of the same concrete type:
arrays and tuples in order, declared record fields, matching union alternatives,
Map values by existing key lookup, and Set membership regardless of insertion
order. Numbers treat NaN as equal and distinguish positive and negative zero;
integers retain all 64 bits. Callable and opaque native payloads are rejected,
including when nested inside otherwise supported data.

For example, use `deepStrictEqual([[1, 2]], [[1, 2]])` and
`notDeepStrictEqual([1, 2], [1, 3])`.

`await rejects(factory)` invokes a consumed zero-argument task factory once and
succeeds when the returned task throws a checked failure. It observes and discards
the typed failure; normal completion fails the assertion. An optional message
provides the assertion's failure text. Direct native unwinding and cancellation
of the outer assertion are not successful rejections. A started Job's checked
panic or cancellation error does satisfy `rejects`, like other checked payloads.
See the [testing reference](/tsrust/reference/library/testing/) for exact contracts.

## Select tests

| Command | Selection |
| --- | --- |
| `tsrust test` | Discover tests below the nearest configured project root, or the current directory when no project exists. |
| `tsrust test app/tests` | Discover tests below that directory. |
| `tsrust test one.test.tsr two.spec.tsr` | Run the explicitly named files. |

Directory discovery selects `.test.tsr` and `.spec.tsr` files. Explicit files can
have any `.tsr` filename. Discovery skips hidden directories, `target`,
`node_modules`, `archive`, and symlink entries. Repeated references to the same
canonical file run only once. Finding no test files reports an error.

### Filter names

```sh
tsrust test app/tests --filter 'billing > rejects invalid input'
```

The filter is a literal, case-sensitive substring of the displayed full name,
including suite names joined by ` > `. It is not a regular expression. Excluded
tests count as skipped; suites without selected descendants do not run hooks.
File discovery and compilation still happen before name selection.

### Set an explicit asynchronous deadline

```sh
tsrust test app/tests --timeout 1000
```

The value is a nonnegative integer number of milliseconds within the platform
clock's range. There is no default deadline. It applies to async test bodies,
not hooks or synchronous blocking callbacks. On expiration, the runner drops the
body future, cancels and drains child work, then runs teardown. Cleanup may take
longer than the requested deadline. Use an external process watchdog to bound
blocking code and total build/test time.

## Read failures and logs

Results are grouped by file, with passing/failing test markers, failure sections
and file/test totals. Reported durations measure execution, excluding compilation.

Console output from a test and its per-test hooks appears below that test's result.
Each entry labels stdout or stderr and the original source file, line and column.
Imported helpers point to their own source files; multiline output is indented
beneath the location. Logs remain available when a test fails and stay on their
original output stream. Suite setup/teardown output has its own scope label.

## Read machine-readable results

```sh
tsrust test app/tests --reporter json
```

`json` writes newline-delimited JSON: one independent JSON object per line,
not a JSON array. Event names and fields are:

| Event | Fields besides `event` |
| --- | --- |
| `test`, `hook` | `name`, `status` (`passed` or `failed`), `messages` (strings), `durationMs` |
| `output` | `name`, `stream` (`stdout` or `stderr`), `location`, `text` |
| `summary` | `passed`, `failed`, `hookFailures`, `skipped`, `filesPassed`, `filesFailed`, `durationMs` |

Console output is captured as `output` records with its original stream and source
location. Parse each line as JSON; do not assume the `summary` has a `name` field.
Build/check diagnostics and native dependency failures can still appear on stderr.

## Separate build and execution timings

```sh
tsrust test app/tests --reporter json --profile
```

The selected files share one compiled native runner. `--profile` adds an initial
`build` JSON event with `durationMs` for command setup/build, `checkMs`, `emitMs`,
`backendMs`, `cacheMs`, `nativeBuildMs`, `cache` (`hit` or `miss`) and `testFiles`.
The total includes discovery and setup; individual stages need not sum to it.
A cache hit reports zero native build time. Test/hook and summary `durationMs`
continue to measure execution separately. Without `--profile`, JSON events are
unchanged. Pretty/plain reporters put the build profile on stderr.

Keep suite paths, the working directory and the build environment stable to reuse
the native cache. Source, backend or environment changes can invalidate it.
Per-test durations help locate slow cases, but a single correctness run is not a
controlled performance benchmark. Compare repeated, matched workloads when making
performance claims.

## Control terminal output

Use `tsrust test --color never` to disable ANSI color, or select `auto` or `always`.
Select `--reporter pretty`, `--reporter plain` or `--reporter json` explicitly.
To use the compact line reporter, set `TSRUST_TEST_REPORTER=plain` in the process
environment before launching the command. For example:

```sh
# macOS/Linux shell
TSRUST_TEST_REPORTER=plain tsrust test
```

```powershell
# Windows PowerShell
$env:TSRUST_TEST_REPORTER = 'plain'
tsrust test
Remove-Item Env:TSRUST_TEST_REPORTER
```

Direct `runTests()` calls use the compact reporter unless the environment selects
`TSRUST_TEST_REPORTER=pretty`. Most projects should let `tsrust test` manage discovery
and execution instead of calling the runner directly.
