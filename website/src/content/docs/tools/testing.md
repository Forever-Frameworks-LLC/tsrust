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

## Read failures and logs

Results are grouped by file, with passing/failing test markers, failure sections
and file/test totals. Reported durations measure execution, excluding compilation.

Console output from a test and its per-test hooks appears below that test's result.
Each entry labels stdout or stderr and the original source file, line and column.
Imported helpers point to their own source files; multiline output is indented
beneath the location. Logs remain available when a test fails and stay on their
original output stream. Suite setup/teardown output has its own scope label.

## Control terminal output

Use `tsrust test --color never` to disable ANSI color, or select `auto` or `always`.
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
