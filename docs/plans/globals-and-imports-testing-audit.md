# Testing API and private bridge review

Status: reviewed 18 bundled public exports, 29 matcher members and 14 private
native bindings. Keep public testing APIs imported from `std:testing`, with matcher
methods on returned assertion objects. Do not introduce ambient test globals or
advertise private runtime imports. Local aliases remain ordinary imports.

## Public exports

Signatures below come from the current editor catalog and match bundled source
in `crates/tsrust-compiler/src/stdlib/testing/api.rs`. Assertions return void on
success and signal AssertionError on failure; the runner catches failures and
continues eligible tests. They do not return Result. Registration callbacks are
synchronous; no async/timeouts/options compatibility is implied. Runtime support
is selected testing/assertion/runner/hooks code, never a user dependency flag.

| Symbol/signature | Effects and ownership | Placement |
|---|---|---|
| `ok<T>(value: T, message: string = "Expected a truthy value"): void` | Truthiness assertion; borrows owned value/message; failure stops current test. | Retain explicit import |
| `strictEqual<T>(actual: T, expected: T, message: string = "Expected values to be strictly equal"): void` | Strict equality assertion; borrows both operands and message; failure includes details. | Retain explicit import |
| `notStrictEqual<T>(actual: T, expected: T, message: string = "Expected values to differ"): void` | Negated strict equality; borrows operands/message; no implicit clone. | Retain explicit import |
| `fail(message: string = "Failed"): void` | Unconditional assertion failure; borrows optional message. | Retain explicit import |
| `expect<T>(actual: T): Expectation<T>` | Transfers actual into a reusable assertion object; retains one actual value for positive/negative checks. | Retain explicit import |
| `test(name: string, callback: Owned<Function<void>>): void` | Borrows/copies display name; consumes owned synchronous void callback for later execution. | Retain explicit import |
| `it(name: string, callback: Owned<Function<void>>): void` | Alias of test with identical retention and failure behavior. | Retain explicit import |
| `runTests(): int` | Execute registered tests; returns failure count as int; runner captures test failures. | Retain explicit import |
| `describe(name: string, callback: Mutable<Function<void>>): void` | Runs registration callback immediately through mutable borrow; suites group later tests. | Retain explicit import |
| `suite(name: string, callback: Mutable<Function<void>>): void` | Alias of describe; same registration-time mutation and ordering. | Retain explicit import |
| `beforeAll(callback: Owned<Function<void>>): void` | Consumes owned callback retained for one setup per scope. | Retain explicit import |
| `afterAll(callback: Owned<Function<void>>): void` | Consumes owned callback retained for one teardown per scope. | Retain explicit import |
| `beforeEach(callback: Owned<Function<void>>): void` | Consumes owned callback retained and reused before each scoped test. | Retain explicit import |
| `afterEach(callback: Owned<Function<void>>): void` | Consumes owned callback retained and reused after each scoped test. | Retain explicit import |
| `before(callback: Owned<Function<void>>): void` | Alias of beforeAll; same retained owner. | Retain explicit import |
| `after(callback: Owned<Function<void>>): void` | Alias of afterAll; same retained owner. | Retain explicit import |
| `type Expectation<T>` | Owned callable record; methods borrow retained state. Not a constructor or ambient global. | Retain exported type; dedicated member catalog |
| `type NegatedExpectation<T>` | Owned callable record; methods borrow retained state. Not a constructor or ambient global. | Retain exported type; dedicated member catalog |

## Matcher members

Each row is a distinct catalog symbol. Methods return void, leave the assertion
object reusable and fail through the assertion runner. Expected owned values are
borrowed; numeric thresholds are copied. Nonnumeric actuals fail numeric matchers
under both positive and negated views. Unsupported lengths fail both views.
Wrong types/arity are compile-time diagnostics. Negation changes the condition,
not validation. No method creates a new public global or imported duplicate.

| Symbol | Signature | Condition and disposition |
|---|---|---|
| `Expectation.toBe` | `toBe(expected: T): void` | Object.is identity; NaN matches NaN, signed zeros differ; expected is borrowed. Retain receiver member. |
| `Expectation.toBeTruthy` | `toBeTruthy(): void` | Boolean truthiness. Retain receiver member. |
| `Expectation.toBeFalsy` | `toBeFalsy(): void` | Boolean falsiness. Retain receiver member. |
| `Expectation.toBeNull` | `toBeNull(): void` | Actual is null. Retain receiver member. |
| `Expectation.toBeUndefined` | `toBeUndefined(): void` | Actual is undefined. Retain receiver member. |
| `Expectation.toBeDefined` | `toBeDefined(): void` | Actual is not undefined; null counts as defined. Retain receiver member. |
| `Expectation.toBeNaN` | `toBeNaN(): void` | Actual is numeric NaN without coercion. Retain receiver member. |
| `Expectation.not` | `not: NegatedExpectation<T>` | Borrowed negative view sharing retained actual; no second payload clone; chained .not.not unsupported. Retain receiver member. |
| `Expectation.toBeTypeOf` | `toBeTypeOf(expected: ReadOnly<string>): void` | Captured typeof result matches borrowed expected text. Retain receiver member. |
| `Expectation.toBeGreaterThan` | `toBeGreaterThan(expected: number): void` | Numeric actual > expected. Retain receiver member. |
| `Expectation.toBeGreaterThanOrEqual` | `toBeGreaterThanOrEqual(expected: number): void` | Numeric actual >= expected. Retain receiver member. |
| `Expectation.toBeLessThan` | `toBeLessThan(expected: number): void` | Numeric actual < expected. Retain receiver member. |
| `Expectation.toBeLessThanOrEqual` | `toBeLessThanOrEqual(expected: number): void` | Numeric actual <= expected. Retain receiver member. |
| `Expectation.toBeCloseTo` | `toBeCloseTo(expected: number, numDigits?: number): void` | Numeric closeness; precision defaults 2; strict abs difference < 10^(-precision)/2, with same-sign infinity exception. Retain receiver member. |
| `Expectation.toHaveLength` | `toHaveLength(expected: number): void` | Captured numeric length equals expected; strings count UTF-16 units, arrays elements, tuples slots. Retain receiver member. |
| `NegatedExpectation.toBe` | `toBe(expected: T): void` | Negates: Object.is identity; NaN matches NaN, signed zeros differ; expected is borrowed. Retain receiver member. |
| `NegatedExpectation.toBeTruthy` | `toBeTruthy(): void` | Negates: Boolean truthiness. Retain receiver member. |
| `NegatedExpectation.toBeFalsy` | `toBeFalsy(): void` | Negates: Boolean falsiness. Retain receiver member. |
| `NegatedExpectation.toBeNull` | `toBeNull(): void` | Negates: Actual is null. Retain receiver member. |
| `NegatedExpectation.toBeUndefined` | `toBeUndefined(): void` | Negates: Actual is undefined. Retain receiver member. |
| `NegatedExpectation.toBeDefined` | `toBeDefined(): void` | Negates: Actual is not undefined; null counts as defined. Retain receiver member. |
| `NegatedExpectation.toBeNaN` | `toBeNaN(): void` | Negates: Actual is numeric NaN without coercion. Retain receiver member. |
| `NegatedExpectation.toBeTypeOf` | `toBeTypeOf(expected: ReadOnly<string>): void` | Negates: Captured typeof result matches borrowed expected text. Retain receiver member. |
| `NegatedExpectation.toBeGreaterThan` | `toBeGreaterThan(expected: number): void` | Negates: Numeric actual > expected. Retain receiver member. |
| `NegatedExpectation.toBeGreaterThanOrEqual` | `toBeGreaterThanOrEqual(expected: number): void` | Negates: Numeric actual >= expected. Retain receiver member. |
| `NegatedExpectation.toBeLessThan` | `toBeLessThan(expected: number): void` | Negates: Numeric actual < expected. Retain receiver member. |
| `NegatedExpectation.toBeLessThanOrEqual` | `toBeLessThanOrEqual(expected: number): void` | Negates: Numeric actual <= expected. Retain receiver member. |
| `NegatedExpectation.toBeCloseTo` | `toBeCloseTo(expected: number, numDigits?: number): void` | Negates: Numeric closeness; precision defaults 2; strict abs difference < 10^(-precision)/2, with same-sign infinity exception. Retain receiver member. |
| `NegatedExpectation.toHaveLength` | `toHaveLength(expected: number): void` | Negates: Captured numeric length equals expected; strings count UTF-16 units, arrays elements, tuples slots. Retain receiver member. |

The UTF-16 length matcher is an intentional testing compatibility exception to
native string byte length. It is documented directly; it must not be generalized
to the language's string indexing contract. Internal shared callback retention
keeps positive and negative identity checks on one actual value; it is not a
public shared-ownership escape hatch and does not make borrowed views clonable.

## Private native bridges

These bindings are inaccessible to user modules: `project.rs::visit` rejects
`std:testing-runtime` unless the importer is bundled `std:testing`. The one private
catalog entry for checkAssertion supports internal indexing, not public completion.
All rows retain private placement. Metadata is in `stdlib/testing/bindings.rs`;
compiler-only observations lower in `emit.rs`, and runner state lives in
`stdlib/testing/registration_runtime.rs` and associated runtime modules.

| Binding | Contract and purpose | Runtime |
|---|---|---|
| `checkAssertion` | `(boolean, borrowed string): void`; signal failed condition | Assertion support |
| `assertionDetails` | `(borrowed actual, borrowed expected, borrowed message): string`; owned failure explanation | Compiler observation |
| `shareCallback` | Consumes eligible owned callback; returns internal shared callback for retained identity checks | Compiler intrinsic; no public type spelling |
| `numericValue` | Borrow actual; return number projection without coercion; unsupported values produce NaN | Compiler observation |
| `lengthValue` | Borrow actual; return numeric length projection | Compiler observation |
| `hasLength` | Borrow actual; return whether numeric length is supported | Compiler observation |
| `containsValue` | Borrow actual/expected; return containment boolean | Compiler observation; no public matcher currently exports it |
| `registerTest` | Borrow name, consume owned void callback; return void | Testing runner |
| `runTests` | No arguments; execute registry, return int failure count | Testing runner; public wrapper is a distinct export |
| `finishTests` | No arguments; finish runner/process outcome; return void | CLI runner bridge |
| `beginSuite` | Borrow name; push registration scope; return void | Suite registry |
| `endSuite` | No arguments; pop registration scope; return void | Suite registry |
| `beginFile` | Borrow name; establish current test file; return void | CLI runner bridge |
| `registerHook` | Copy kind tag, consume owned callback; return void | Hook registry |

`Expectation` and `NegatedExpectation` are real public exports despite lacking
same-group catalog rows: their dedicated member groups and source declarations
supply editor help. They are not missing compiler implementations. `containsValue`
is an internal capability, not permission to advertise a toContain matcher.

## Evidence and limits

The new editor witness checks all 18 public export completions, excludes every
private bridge except the separately public runTests wrapper, and confirms direct
private imports fail during header loading. Existing compiler tests verify private
import rejection, retained actual identity, borrowed expected values, invalid
matcher contracts and recovery after negated assertion failures. Evidence is in
SQLite under `tsrust-api-testing-*`. This audit preserves existing runtime
behavior; complete testing-family measurements are not rerun here.

The audit repaired a stale diagnostic expectation: unconfigured bare/colon aliases
now produce module-load, while private standard-library imports must still produce
module-native with the explicit privacy explanation. The witness now checks these
separately and formats its source before execution.
