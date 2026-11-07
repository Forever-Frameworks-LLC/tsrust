<!-- tsrust-docs
{"id":"reference-standard-library-testing","group":"standard-library","order":20,"status":"partial","targets":["web","native"],"api":[{"receiver":"std:testing","name":"deepStrictEqual"},{"receiver":"std:testing","name":"notDeepStrictEqual"},{"receiver":"std:testing","name":"rejects"},{"receiver":"std:testing","name":"throws"},{"receiver":"std:testing","name":"TrackedResource"},{"receiver":"std:testing","name":"trackedResource"},{"receiver":"std:testing","name":"resourceId"},{"receiver":"std:testing","name":"closeResource"},{"receiver":"std:testing","name":"releaseCount"},{"receiver":"std:testing","name":"releaseOrder"},{"receiver":"std:testing","name":"liveCount"},{"receiver":"std:testing","name":"ok"},{"receiver":"std:testing","name":"strictEqual"},{"receiver":"std:testing","name":"notStrictEqual"},{"receiver":"std:testing","name":"fail"},{"receiver":"std:testing","name":"expect"},{"receiver":"std:testing","name":"test"},{"receiver":"std:testing","name":"it"},{"receiver":"std:testing","name":"runTests"},{"receiver":"std:testing","name":"describe"},{"receiver":"std:testing","name":"suite"},{"receiver":"std:testing","name":"beforeAll"},{"receiver":"std:testing","name":"afterAll"},{"receiver":"std:testing","name":"beforeEach"},{"receiver":"std:testing","name":"afterEach"},{"receiver":"std:testing","name":"before"},{"receiver":"std:testing","name":"after"},{"receiver":"Expectation","name":"toBe"},{"receiver":"Expectation","name":"toBeTruthy"},{"receiver":"Expectation","name":"toBeFalsy"},{"receiver":"Expectation","name":"toBeNull"},{"receiver":"Expectation","name":"toBeUndefined"},{"receiver":"Expectation","name":"toBeDefined"},{"receiver":"Expectation","name":"toBeNaN"},{"receiver":"Expectation","name":"not"},{"receiver":"Expectation","name":"toBeTypeOf"},{"receiver":"Expectation","name":"toBeGreaterThan"},{"receiver":"Expectation","name":"toBeGreaterThanOrEqual"},{"receiver":"Expectation","name":"toBeLessThan"},{"receiver":"Expectation","name":"toBeLessThanOrEqual"},{"receiver":"Expectation","name":"toBeCloseTo"},{"receiver":"Expectation","name":"toHaveLength"},{"receiver":"NegatedExpectation","name":"toBe"},{"receiver":"NegatedExpectation","name":"toBeTruthy"},{"receiver":"NegatedExpectation","name":"toBeFalsy"},{"receiver":"NegatedExpectation","name":"toBeNull"},{"receiver":"NegatedExpectation","name":"toBeUndefined"},{"receiver":"NegatedExpectation","name":"toBeDefined"},{"receiver":"NegatedExpectation","name":"toBeNaN"},{"receiver":"NegatedExpectation","name":"toBeTypeOf"},{"receiver":"NegatedExpectation","name":"toBeGreaterThan"},{"receiver":"NegatedExpectation","name":"toBeGreaterThanOrEqual"},{"receiver":"NegatedExpectation","name":"toBeLessThan"},{"receiver":"NegatedExpectation","name":"toBeLessThanOrEqual"},{"receiver":"NegatedExpectation","name":"toBeCloseTo"},{"receiver":"NegatedExpectation","name":"toHaveLength"}]}
-->
# Testing

Use assertions on observed values and independent cleanup counts. Resource observation belongs to std:testing; no separate testing-resources module is needed.

Tests and hooks accept synchronous callbacks returning `void` and asynchronous
callbacks returning `Task<void>`. Run a `*.test.tsr` file with `tsrust test`.
Each returned task finishes before the next hook or test. Suite registration
through `describe` or `suite` stays synchronous.

```tsrust
import { beforeAll, test, strictEqual } from 'std:testing';
import { delay } from 'std:futures';

let ready = false;
beforeAll(async () => {
  await delay(0);
  ready = true;
});
test('async setup', async () => {
  await delay(0);
  strictEqual(ready, true);
});
```

Failed assertions and thrown values after `await` fail the current test. Teardown
still runs and later tests continue. Owned captures persist between repeated hook
invocations; module state is borrowed for the entire invocation, including awaits.
Passing a constructed task instead of a callback, returning a non-void task result,
or retaining borrowed local data in the registry is rejected.

## Structural and asynchronous assertions

`deepStrictEqual(actual, expected, message?)` borrows two values of the same
concrete supported data type. Arrays and tuples compare ordered elements; records
compare fields without considering construction order; data unions compare their
active alternatives. Number leaves use Object.is semantics (NaN equals NaN;
positive and negative zero differ). Integers retain their exact signed64-bit
values. Map values compare structurally under the Map's existing key lookup
rules; Set comparison uses existing membership rules. Insertion order does not
change equality. Object keys in Maps/Sets retain identity semantics, so cloning
an object key can change membership. Unsupported callable or opaque native values
are rejected even when nested. Recursive record comparison uses a work stack;
comparison has no hidden depth or item cap. `notDeepStrictEqual` reverses the
success condition. Neither assertion clones or serializes inputs.

`await rejects(factory, message?)` consumes a single-use zero-argument task
factory and awaits its result. Any checked thrown payload satisfies it and is
discarded, including records/unions; normal completion raises AssertionError.
The assertion's own failure is outside its catch. Direct native unwinding in the
factory is not caught. Cancelling the outer rejects task cancels its operation;
it does not make it pass. Awaiting a started Job transports that job's panic or
cancellation as a checked AsyncError. That checked payload does satisfy rejects,
just like other checked throws, even if the job also prints panic diagnostics.
No matching or filtering by an error's code field occurs. This preserves the
existing Job error boundary instead of treating a checked error as direct unwind.

```tsrust
import {deepStrictEqual,notDeepStrictEqual,rejects} from 'std:testing';
async function main():Task<void>{
  deepStrictEqual([[1,2]],[[1,2]]);
  notDeepStrictEqual([1,2],[1,3]);
  await rejects(async()=>{throw {code:'expected',retry:false};});
}
```

## Selecting tests and reporting results

`tsrust test [paths...] --filter <text>` selects names containing a literal,
case-sensitive substring of the full displayed suite/test path, joined by
` > `. This is not a regular expression; metacharacters have no special meaning.
Unmatched suites run no hooks. Zero selected tests succeeds and reports skipped
counts. File discovery and module registration still occur before filtering.
An empty filter selects all names.

`--reporter pretty|plain|json` selects the run reporter. `--json` remains compiler
diagnostic JSON and is independent. The JSON reporter writes newline-delimited
JSON (one complete object per line), without ANSI styling. Stable events:

| event | fields |
| --- | --- |
| test | name, status (`passed`/`failed`), messages (string array), durationMs |
| hook | name, status, messages, durationMs |
| output | name (empty outside a test/hook), stream (`stdout`/`stderr`), location, text |
| summary | passed, failed, hookFailures, skipped, filesPassed, filesFailed, durationMs |

Console text, including controls and Unicode, is escaped as JSON data. Both
captured stdout and stderr are output events on the reporter's stdout. Compiler
errors keep the separate diagnostic channel. Registration-time console output
is also an output event. Native third-party writes that bypass console are not
rewritten by this reporter. Failure messages and console locations retain source
information; durations are observations, not stable snapshot values.

Pass related files together to compile one native runner:

```sh
tsrust test app/tests/api.test.tsr app/tests/storage.test.tsr --reporter json --profile
```

`--profile` separates setup/build cost from test execution. After a successful
build, the JSON reporter starts with a `build` event containing `durationMs`,
`checkMs`, `emitMs`, `backendMs`, `cacheMs`, `nativeBuildMs`, `cache` (`hit` or
`miss`) and `testFiles`. Total setup time includes discovery, so individual stages
need not sum to it. A cache hit reports zero native build time. Test, hook and
summary durations continue to measure execution separately. Pretty/plain
reporters print the build profile on stderr. Without `--profile`, the existing
report event format is unchanged.

Keep paths, working directory and build environment stable to reuse the native
cache. Source, backend or environment changes can invalidate it. These durations
help locate expensive stages; comparable performance measurements still need
warmup, repeated samples and matched work in each engine.

`--timeout <milliseconds>` sets an explicit nonnegative integer deadline for
async test bodies. There is no default deadline and no policy maximum below the
backend duration/Instant range. A timeout drops the callback future, cancels and
awaits its child scope, then runs afterEach/afterAll before continuing tests.
Teardown receives no inherited expired body deadline. Hooks are not timed by this
option. Zero retains cooperative backend timeout semantics: an immediately ready
future may complete before the timer is checked. Borrowed tasks release loans during settlement; completed side effects
are not rolled back. Synchronous or blocking callbacks cannot be preempted by a
cooperative async deadline and still require a process watchdog. The `=value`
forms of all three options are accepted as well.
