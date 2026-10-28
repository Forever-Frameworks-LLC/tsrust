---
title: Run project tests
description: Discover native tests and understand output, failures and reporter settings.
---

Run `tsrust test` at the project root to discover `.test.tsr` and `.spec.tsr` files.
Use explicit files or directories to narrow the run. Native tests require the
[prepared Rust tools and sources](../rust-setup/).

```sh
tsrust test
tsrust test app/tests
```

## Read results

`tsrust test` groups results by file, marks passing and failing tests, shows failure
sections and reports file/test totals. Durations measure test execution, excluding
compilation. Strict equality failures show expected and received values; object
comparison still uses identity, even when printed contents look alike. Console output from each test and its per-test hooks appears
below that test's result, with stdout/stderr labels and the original source file,
line and column. Imported helpers point to their own source files. Multiline output
is indented beneath its source location. Test names are not repeated beside stream
labels; suite setup/teardown output retains its separate scope label. Output remains
on its original stream; logs are retained when a test fails. Colors follow
terminal support and `--color auto|always|never`. Set `TSRUST_TEST_REPORTER=plain` for
the compact line reporter used by older scripts. Direct `runTests()` calls retain
that compact output unless the pretty reporter is selected through the environment.
