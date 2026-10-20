// Checked public declarations for the bundled std:testing module.
// These declarations are parsed by the compiler; native execution uses Rust backends.
pub(crate) const SOURCE: &str = r####"import { assertionDetails, checkAssertion, shareCallback, numericValue, lengthValue, hasLength } from "std:testing-runtime";

import { TrackedResource as NativeTrackedResource, trackedResource as nativeTrackedResource, resourceId as nativeResourceId, closeResource as nativeCloseResource, releaseCount as nativeReleaseCount, releaseOrder as nativeReleaseOrder, liveCount as nativeLiveCount } from "std:testing-runtime";
/**
An opaque, uniquely owned synthetic native resource for testing destruction. Construct with trackedResource(); inspect its identity with resourceId(resource); consume it with closeResource(resource). It exposes no fields or receiver methods. Scope exit and replacement destroy native storage automatically. releaseCount and releaseOrder observe destruction without retaining the resource. Cloning, field projection, worker transfer and started-task transport are rejected. Direct local async ownership is supported. This instrument does not represent asynchronous backend cleanup.

Ownership: moves transfer the resource; ReadOnly inspection borrows it. Its identity is a copied integer, not a resource handle.
*/
export type TrackedResource = NativeTrackedResource;

/**
Creates a unique synthetic native resource with a positive process-local identity and real destructor. Takes no arguments. Returns an owned opaque resource. Identity exhaustion terminates native execution before reuse. Scope exit, replacement and consuming closeResource release its native allocation. This test instrument is not a file, database or UI backend.

The resource cannot be cloned, projected or transferred to worker threads or started tasks. Direct local ownership and borrowing apply. No asynchronous cleanup completion is implied.

```tsrust
const r = trackedResource();
const key = resourceId(r);
closeResource(r);
strictEqual(releaseCount(key), 1);
```

Ownership: resource operations use checked borrowing or consumption; audit integers copy without retaining resources.
*/
export function trackedResource():TrackedResource {return nativeTrackedResource();}

/**
Returns the positive identity of a live resource. Takes one borrowed resource and does not change or release it. The returned integer does not retain the resource.

The resource cannot be cloned, projected or transferred to worker threads or started tasks. Direct local ownership and borrowing apply. No asynchronous cleanup completion is implied.

```tsrust
const r = trackedResource();
const key = resourceId(r);
closeResource(r);
strictEqual(releaseCount(key), 1);
```

Ownership: resource operations use checked borrowing or consumption; audit integers copy without retaining resources.
*/
export function resourceId(resource:ReadOnly<TrackedResource>):int {return nativeResourceId(resource);}

/**
Consumes one resource and releases its native allocation before recording destruction. Returns void. Repeated closeResource calls or later use of the consumed value is a compile-time error. Automatic scope cleanup uses the same destructor.

The resource cannot be cloned, projected or transferred to worker threads or started tasks. Direct local ownership and borrowing apply. No asynchronous cleanup completion is implied.

```tsrust
const r = trackedResource();
const key = resourceId(r);
closeResource(r);
strictEqual(releaseCount(key), 1);
```

Ownership: resource operations use checked borrowing or consumption; audit integers copy without retaining resources.
*/
export function closeResource(resource:Owned<TrackedResource>):void {nativeCloseResource(resource);}

/**
Returns the number of observed destructions for one identity, normally zero while live and exactly one after release. Takes an int of any value. Unknown IDs return zero. Does not retain resources or reset the audit.

The resource cannot be cloned, projected or transferred to worker threads or started tasks. Direct local ownership and borrowing apply. No asynchronous cleanup completion is implied.

```tsrust
const r = trackedResource();
const key = resourceId(r);
closeResource(r);
strictEqual(releaseCount(key), 1);
```

Ownership: resource operations use checked borrowing or consumption; audit integers copy without retaining resources.
*/
export function releaseCount(id:int):int {return nativeReleaseCount(id);}

/**
Returns the one-based destruction position for one identity, or zero if no destruction was observed. Takes an int of any value. Unknown IDs return zero. Order compares synchronous destruction events, not asynchronous backend completion.

The resource cannot be cloned, projected or transferred to worker threads or started tasks. Direct local ownership and borrowing apply. No asynchronous cleanup completion is implied.

```tsrust
const r = trackedResource();
const key = resourceId(r);
closeResource(r);
strictEqual(releaseCount(key), 1);
```

Ownership: resource operations use checked borrowing or consumption; audit integers copy without retaining resources.
*/
export function releaseOrder(id:int):int {return nativeReleaseOrder(id);}

/**
Returns created resources minus observed destructions in the current process. Takes no arguments. Does not change or retain resources. Audit storage contains only integer identities and lasts until process exit; use bounded test processes.

The resource cannot be cloned, projected or transferred to worker threads or started tasks. Direct local ownership and borrowing apply. No asynchronous cleanup completion is implied.

```tsrust
const r = trackedResource();
const key = resourceId(r);
closeResource(r);
strictEqual(releaseCount(key), 1);
```

Ownership: resource operations use checked borrowing or consumption; audit integers copy without retaining resources.
*/
export function liveCount():int {return nativeLiveCount();}


/**
Fails when value is falsy. Accepts a supported source value and an optional string message. False, numeric zero, NaN, empty strings, null and undefined are falsy; arrays and records are truthy, including empty containers. Returns void on success. Failure stops the current test body; outside a running test it stops native execution with AssertionError followed by the message. No receiver is changed. The default message is "Expected a truthy value". Wrong arity or a non-string message is a compile-time error.

@param value - Value whose truthiness is checked.
@param message - Optional failure description.

```tsrust
import { ok } from "std:testing";
function main(){ok("é");console.log("passed");}
// passed
```

Ownership: the value and message are borrowed when they own data; scalars are copied.
*/
export function ok<T>(value:T, message:string="Expected a truthy value"):void {
    checkAssertion(Boolean(value), message);
}

/**
Fails unless actual and expected compare equal with Object.is semantics. Accepts two values of one inferred concrete type and an optional string message. Number NaN equals itself; positive zero and negative zero differ. Strings compare exact Unicode text, scalars compare values, and arrays and records compare identity rather than contents. Null and undefined are supported as homogeneous concrete types. Data unions compare their active alternatives; callable values are unsupported. Returns void on success without changing either value. Failure stops the current test body; outside a running test it stops native execution with AssertionError followed by the message. The default message is "Expected values to be strictly equal". Wrong arity, incompatible argument types or a non-string message is a compile-time error. Integers preserve their signed 64-bit values without conversion to Number. The CLI test report includes expected and received values on failure. Strings are quoted; supported arrays and records are inspected without changing identity comparison. Opaque values are labeled as not expanded.

@param actual - Observed value.
@param expected - Expected value of the same concrete type.
@param message - Optional failure description.

```tsrust
import { strictEqual } from "std:testing";
function main(){strictEqual(Number.NaN,Number.NaN);strictEqual("é","é");console.log("passed");}
// passed
```

Ownership: both values and the message are borrowed when they own data; scalars are copied.
*/
export function strictEqual<T>(actual:T, expected:T, message:string="Expected values to be strictly equal"):void {
    if (!Object.is(actual, expected)) { checkAssertion(false, assertionDetails(actual, expected, message)); }
}

/**
Fails when actual and expected compare equal with Object.is semantics. Accepts two values of one inferred concrete type and an optional string message. Number NaN equals itself; positive zero and negative zero differ. Strings compare exact text; arrays and records compare identity. Homogeneous null and undefined are supported; data unions compare their active alternatives; callable values are unsupported. Returns void on success without changing either value. Failure stops the current test body; outside a running test it stops native execution with AssertionError followed by the message. The default message is "Expected values to differ". Wrong arity, incompatible argument types or a non-string message is a compile-time error. Integers preserve their signed 64-bit values.

@param actual - Observed value.
@param expected - Value that must differ, of the same concrete type.
@param message - Optional failure description.

```tsrust
import { notStrictEqual } from "std:testing";
function main(){notStrictEqual(0.0,-0.0);console.log("passed");}
// passed
```

Ownership: both values and the message are borrowed when they own data; scalars are copied.
*/
export function notStrictEqual<T>(actual:T, expected:T, message:string="Expected values to differ"):void {
    checkAssertion(!Object.is(actual, expected), message);
}

/**
Always stops the current native execution with AssertionError followed by the supplied string message. Takes zero or one argument; the default message is "Failed". A non-string argument or more than one argument is a compile-time error. It does not return normally and changes no receiver.

@param message - Optional failure description.

```tsrust
import { fail } from "std:testing";
function main(){fail("wrong sum");}
// execution fails with AssertionError: wrong sum
```

Ownership: the message is borrowed.
*/
export function fail(message:string="Failed"):void {
    checkAssertion(false, message);
}


/**
Calls a synchronous callback once and fails unless it throws. Takes a consuming single-use zero-argument callback returning void, plus an optional failure message (default "Expected function to throw"). Any thrown value satisfies this assertion; the value is discarded. A callback that returns normally causes AssertionError after the callback completes. Returns void and does not retain the callback. Errors from asynchronous work are not awaited or observed; async callbacks do not match this signature.

```tsrust
import { throws } from "std:testing";
function main(){throws(()=>{throw "expected";});console.log("passed");}
// passed
```

Callback-local resources release through ordinary checked exception cleanup. The assertion's own no-throw failure is outside the catch and cannot satisfy itself. No error matcher or coercion is performed.

Ownership: consumes the callback and drops it before returning; the message is borrowed.
*/
export function throws(callback:Once<Function<void>>,message:string="Expected function to throw"):void {
    let didThrow=false;
    try {callback();} catch {didThrow=true;}
    checkAssertion(didThrow,message);
}


export type Expectation<T> = {
 /**
Provides negated identity, truthiness, falsiness, null, undefined, defined and NaN checks. Matchers return void and fail when their positive condition succeeds. The assertion object remains reusable. Chained .not.not is currently unsupported.

Ownership: both views share one retained actual value; reading not borrows the assertion object without cloning its payload.
*/
 not:NegatedExpectation<T>;
/**
Fails unless the retained actual value and expected compare equal with Object.is semantics. Takes one expected value of the same concrete type. Returns void on success and leaves the assertion object reusable. Number NaN equals itself; positive and negative zero differ. Strings compare exact text; arrays and records compare identity. Integers retain signed 64-bit values. Data unions compare their active alternatives; callable values are unsupported. Wrong arity or incompatible types are compile-time errors. Failure stops the current test body; outside a running test it stops native execution with AssertionError.

@param expected - Expected value with the same concrete type as actual.

```tsrust
import { expect } from "std:testing";
function main(){expect(Number.NaN).toBe(Number.NaN);console.log("passed");}
// passed
```

Ownership: the assertion object and expected values are borrowed. The assertion object owns its captured actual value.
*/
 toBe: Function<void, [expected: ReadOnly<T>]>;
 toBeTruthy: Function<void>;
 toBeFalsy: Function<void>;
 toBeNull: Function<void>;
 toBeUndefined: Function<void>;
 toBeDefined: Function<void>;
 toBeNaN: Function<void>;
 toBeTypeOf:Function<void,[expected:ReadOnly<string>]>;
 toHaveLength:Function<void,[expected:number]>;
 toBeCloseTo:Function<void,[expected:number,numDigits?:number]>;
 toBeGreaterThan:Function<void,[expected:number]>;
 toBeGreaterThanOrEqual:Function<void,[expected:number]>;
 toBeLessThan:Function<void,[expected:number]>;
 toBeLessThanOrEqual:Function<void,[expected:number]>;

};

/**
Creates an assertion object for actual. Takes one supported concrete value and
returns an object whose toBe(expected) method checks Object.is equality and whose
toBeTruthy() and toBeFalsy() methods check Boolean conversion.
The toBeNull(), toBeUndefined() and toBeDefined() methods check nullish state;
null is defined because only undefined fails toBeDefined().
The toBeNaN() method accepts only a numeric NaN value without coercion.
Number NaN equals itself; positive and negative zero differ. Strings compare
exact text. Failed assertions stop the current test body; outside a running test they stop native execution. The method returns void on success
and supports repeated checks with the same assertion object.

@param actual - Observed value retained by the assertion object.

```tsrust
import { expect } from "std:testing";
function main(){expect(3).toBe(3);console.log("passed");}
// passed
```

Ownership: scalar actual values copy. Data-owning actual values transfer into
the assertion object; expected values are borrowed. Use clone(text) when retaining
an original string. Array and record identity checks require respecting the same
ownership rules; transferring an object prevents subsequently using that owner.
*/
/**
Negated assertion methods. toBe(expected) takes the same concrete type as actual and succeeds when Object.is equality is false. Zero-argument predicates succeed when their positive condition is false. Failures stop the current test body; subsequent registered tests continue. Wrong types or arity are compile-time errors. Callable values are unsupported.

Ownership: methods borrow the assertion object and expected values. Both identity views share one owned actual value without cloning its payload.
*/
export type NegatedExpectation<T>={
 toBe:Function<void,[expected:ReadOnly<T>]>;
 toBeTruthy:Function<void>;toBeFalsy:Function<void>;toBeNull:Function<void>;
 toBeUndefined:Function<void>;toBeDefined:Function<void>;toBeNaN:Function<void>;
 toBeTypeOf:Function<void,[expected:ReadOnly<string>]>;
 toHaveLength:Function<void,[expected:number]>;
 toBeCloseTo:Function<void,[expected:number,numDigits?:number]>;
 toBeGreaterThan:Function<void,[expected:number]>;
 toBeGreaterThanOrEqual:Function<void,[expected:number]>;
 toBeLessThan:Function<void,[expected:number]>;
 toBeLessThanOrEqual:Function<void,[expected:number]>;

};
function closeNumericValues(actual:number,expected:number,numDigits:number):boolean {
    if(actual===Number.POSITIVE_INFINITY&&expected===Number.POSITIVE_INFINITY){return true;}
    if(actual===Number.NEGATIVE_INFINITY&&expected===Number.NEGATIVE_INFINITY){return true;}
    return Math.abs(actual-expected)<Math.pow(10.0,-numDigits)/2.0;
}
export function expect<T>(actual:T):Expectation<T> {
    const length=lengthValue(actual);
    const lengthSupported=hasLength(actual);
    const numeric=numericValue(actual);
    const isNumeric=typeof actual==="number";
    const typeName=typeof actual;
    const negativeTypeName=clone(typeName);
    const isNaN=Object.is(actual,Number.NaN);
    const truthy=Boolean(actual);
    const isNull=actual===null;
    const isUndefined=actual===undefined;
    const compare=shareCallback((expected:T):boolean=>{return Object.is(actual,expected);});
    const negativeCompare=clone(compare);
    return {
        not:{
            toHaveLength:(expected:number):void=>{checkAssertion(lengthSupported,"Length comparison requires a numeric length property");checkAssertion(length!==expected,"Expected a different length");},
            toBeCloseTo:(expected:number,numDigits:number=2.0):void=>{checkAssertion(isNumeric,"Numeric comparison requires a numeric actual value");checkAssertion(!closeNumericValues(numeric,expected,numDigits),"Expected values not to be close");},
            toBeGreaterThan:(expected:number):void=>{checkAssertion(isNumeric,"Numeric comparison requires a numeric actual value");checkAssertion(!(numeric>expected),"Expected negated numeric comparison");},
            toBeGreaterThanOrEqual:(expected:number):void=>{checkAssertion(isNumeric,"Numeric comparison requires a numeric actual value");checkAssertion(!(numeric>=expected),"Expected negated numeric comparison");},
            toBeLessThan:(expected:number):void=>{checkAssertion(isNumeric,"Numeric comparison requires a numeric actual value");checkAssertion(!(numeric<expected),"Expected negated numeric comparison");},
            toBeLessThanOrEqual:(expected:number):void=>{checkAssertion(isNumeric,"Numeric comparison requires a numeric actual value");checkAssertion(!(numeric<=expected),"Expected negated numeric comparison");},
            toBe:(expected:T):void=>{checkAssertion(!negativeCompare(expected),"Expected values not to be identical");},
            toBeTruthy:():void=>{checkAssertion(!truthy,"Expected a non-truthy value");},
            toBeFalsy:():void=>{checkAssertion(truthy,"Expected a non-falsy value");},
            toBeNull:():void=>{checkAssertion(!isNull,"Expected a value other than null");},
            toBeUndefined:():void=>{checkAssertion(!isUndefined,"Expected a value other than undefined");},
            toBeDefined:():void=>{checkAssertion(isUndefined,"Expected undefined");},
            toBeNaN:():void=>{checkAssertion(!isNaN,"Expected a value other than NaN");},
            toBeTypeOf:(expected:string):void=>{checkAssertion(negativeTypeName!==expected,"Expected a different typeof result");}
        },
        toBeGreaterThan:(expected:number):void=>{checkAssertion(isNumeric,"Numeric comparison requires a numeric actual value");checkAssertion(numeric>expected,"Expected numeric comparison");},
        toBeGreaterThanOrEqual:(expected:number):void=>{checkAssertion(isNumeric,"Numeric comparison requires a numeric actual value");checkAssertion(numeric>=expected,"Expected numeric comparison");},
        toBeLessThan:(expected:number):void=>{checkAssertion(isNumeric,"Numeric comparison requires a numeric actual value");checkAssertion(numeric<expected,"Expected numeric comparison");},
        toBeLessThanOrEqual:(expected:number):void=>{checkAssertion(isNumeric,"Numeric comparison requires a numeric actual value");checkAssertion(numeric<=expected,"Expected numeric comparison");},
        toBeCloseTo:(expected:number,numDigits:number=2.0):void=>{checkAssertion(isNumeric,"Numeric comparison requires a numeric actual value");checkAssertion(closeNumericValues(numeric,expected,numDigits),"Expected values to be close");},
        toHaveLength:(expected:number):void=>{checkAssertion(lengthSupported,"Length comparison requires a numeric length property");checkAssertion(length===expected,"Expected matching length");},
        toBe:(expected:T):void=>{checkAssertion(compare(expected),"Expected values to be identical");},
        toBeTruthy:():void=>{checkAssertion(truthy,"Expected a truthy value");},
        toBeFalsy:():void=>{checkAssertion(!truthy,"Expected a falsy value");},
        toBeNull:():void=>{checkAssertion(isNull,"Expected null");},
        toBeUndefined:():void=>{checkAssertion(isUndefined,"Expected undefined");},
        toBeDefined:():void=>{checkAssertion(!isUndefined,"Expected a defined value");},
        toBeNaN:():void=>{checkAssertion(isNaN,"Expected NaN");},
        toBeTypeOf:(expected:string):void=>{checkAssertion(typeName===expected,"Expected matching typeof result");}
    };
}



import { registerTest, runTests as executeTests, beginSuite, endSuite, registerHook } from "std:testing-runtime";

/**
Registers a synchronous named test and returns void. The callback runs when runTests is called, in registration order. Names may repeat. Registration during test execution is an error. Wrong arity or callback types are compile-time errors. Ownership: the name is borrowed and copied into the registry; the callback transfers into the registry. Module storage is borrowed only while the callback runs, including access through called functions. Borrowed local captures and borrowed callback inputs cannot escape into the registry.

```tsrust
import {test,expect,runTests} from "std:testing";function main(){test("sample",():void=>{expect(3).toBe(3);});console.log(runTests());}
// PASS sample; 1 passed, 0 failed, 0 skipped; 0; 
```
*/
export function test(name:string, callback:Owned<Function<void>>):void { registerTest(name,callback); }

/**
Registers a synchronous named test and returns void. The callback runs when runTests is called, in registration order. Names may repeat. Registration during test execution is an error. Wrong arity or callback types are compile-time errors. This is an alias of test. Ownership: the name is borrowed and copied into the registry; the callback transfers into the registry. Module storage is borrowed only while the callback runs, including access through called functions. Borrowed local captures and borrowed callback inputs cannot escape into the registry.

```tsrust
import {it,expect,runTests} from "std:testing";function main(){it("sample",():void=>{expect(3).toBe(3);});console.log(runTests());}
// PASS sample; 1 passed, 0 failed, 0 skipped; 0; 
```
*/
export function it(name:string, callback:Owned<Function<void>>):void { registerTest(name,callback); }

/**
Runs registered synchronous tests in registration order, consumes the pending registry, prints each result and a summary, and returns the sum of failing tests and failed beforeAll/afterAll hooks. Failed assertions stop that test; later tests continue. Empty registries return zero. Recursive execution is an error. No arguments are accepted. Ownership: pending callbacks are consumed.

```tsrust
import {test,expect,runTests} from "std:testing";function main(){test("sample",():void=>{expect(3).toBe(3);});console.log(runTests());}
// PASS sample; 1 passed, 0 failed, 0 skipped; 0; 
```
*/
export function runTests():int { return executeTests(); }

/**
Groups synchronous tests under a named suite. The zero-argument void callback runs immediately to register its tests and nested suites; test bodies run later, in source order. Returns void. Names may repeat or be empty. Calling describe during test execution is an error. Options, timeouts and async callbacks are unsupported and rejected by arity/type checks.

@param name - Displayed suite name, prefixed to descendant test names.
@param callback - Reusable synchronous registration callback.

```tsrust
import {describe,test,expect,runTests} from "std:testing";
function main(){describe("math",():void=>{test("sum",():void=>{expect(2+3).toBe(5);});});console.log(runTests());}
// PASS math > sum
// 1 passed, 0 failed, 0 skipped
// 0
```

Ownership: the name is borrowed and copied into the registry. The callback is mutably borrowed only during registration; each registered test transfers its own callback into the registry.
*/
export function describe(name:string,callback:Mutable<Function<void>>):void { beginSuite(name);callback();endSuite(); }

/**
Groups synchronous tests under a named suite. The zero-argument void callback runs immediately to register its tests and nested suites; test bodies run later, in source order. Returns void. Names may repeat or be empty. Calling suite during test execution is an error. Options, timeouts and async callbacks are unsupported and rejected by arity/type checks.

@param name - Displayed suite name, prefixed to descendant test names.
@param callback - Reusable synchronous registration callback.

```tsrust
import {suite,test,expect,runTests} from "std:testing";
function main(){suite("math",():void=>{test("sum",():void=>{expect(2+3).toBe(5);});});console.log(runTests());}
// PASS math > sum
// 1 passed, 0 failed, 0 skipped
// 0
```

Ownership: the name is borrowed and copied into the registry. The callback is mutably borrowed only during registration; each registered test transfers its own callback into the registry.

This is an alias of describe.
*/
export function suite(name:string,callback:Mutable<Function<void>>):void { describe(name,callback); }

/**
Registers a synchronous setup callback for the current suite, including descendants, or the current file at top level. Runs once in registration order before tests in that scope. A failed setup stops later setup hooks, blocks descendant test bodies, reports those tests as failed, and still runs teardown for the scope whose setup was attempted. Scopes with no tests do not run hooks. Takes one owned reusable zero-argument callback returning void; returns void and changes the pending hook registry. There are no optional arguments or defaults. Registration during test or hook execution is an error. Wrong arity or callback types are compile-time errors. Context parameters, cleanup return values, timeouts and async callbacks are unsupported.

@param callback - Owned reusable synchronous zero-argument callback returning void.

```tsrust
import {beforeAll,test,runTests} from "std:testing";function main(){beforeAll(():void=>{console.log("hook");});test("sample",():void=>{});console.log(runTests());}
// hook
// PASS sample
// 1 passed, 0 failed, 0 skipped
// 0
```

Ownership: the callback transfers into the registry and is released when the registry is consumed. Owned mutable captures persist across hook invocations. Module storage is borrowed only while each hook runs, so hooks and tests can read or update the same module state in execution order. Borrowed local captures and borrowed callback inputs cannot escape into the registry.

Hooks and tests can share module state:

```tsrust
import {beforeAll,test,strictEqual,runTests} from "std:testing";
let count=0;
function main(){beforeAll(():void=>{count=7;});test("shared state",():void=>{strictEqual(count,7);count=8;});runTests();strictEqual(count,8);}
// PASS shared state; 1 passed, 0 failed, 0 skipped
```

*/
export function beforeAll(callback:Owned<Function<void>>):void { registerHook(0,callback); }
/**
Registers a synchronous teardown callback for the current suite, including descendants, or the current file at top level. Runs once after the scope finishes, in reverse registration order. It also runs after a failed beforeAll in that scope. All teardown hooks are attempted even if earlier teardown fails. Hook failures are reported after the phase and make runTests and the test command fail. Descendant scopes blocked before setup do not run their hooks; scopes with no tests do not run hooks. Takes one owned reusable zero-argument callback returning void; returns void and changes the pending hook registry. There are no optional arguments or defaults. Registration during test or hook execution is an error. Wrong arity or callback types are compile-time errors. Context parameters, cleanup return values, timeouts and async callbacks are unsupported.

@param callback - Owned reusable synchronous zero-argument callback returning void.

```tsrust
import {afterAll,test,runTests} from "std:testing";function main(){afterAll(():void=>{console.log("hook");});test("sample",():void=>{});console.log(runTests());}
// PASS sample
// hook
// 1 passed, 0 failed, 0 skipped
// 0
```

Ownership: the callback transfers into the registry and is released when the registry is consumed. Owned mutable captures persist across hook invocations. Module storage is borrowed only while each hook runs, so hooks and tests can read or update the same module state in execution order. Borrowed local captures and borrowed callback inputs cannot escape into the registry.
*/
export function afterAll(callback:Owned<Function<void>>):void { registerHook(1,callback); }
/**
Registers a synchronous setup callback before every test in the current suite and its descendants, or the current file at top level. Ancestor setup runs before nested setup; multiple hooks in a scope run in registration order. The callback is reused across tests and retains its owned mutable state. A failure stops later setup and skips the test body, but afterEach callbacks still run for the full test scope chain. Scopes with no tests do not run hooks. Takes one owned reusable zero-argument callback returning void; returns void and changes the pending hook registry. There are no optional arguments or defaults. Registration during test or hook execution is an error. Wrong arity or callback types are compile-time errors. Context parameters, cleanup return values, timeouts and async callbacks are unsupported.

@param callback - Owned reusable synchronous zero-argument callback returning void.

```tsrust
import {beforeEach,test,runTests} from "std:testing";function main(){beforeEach(():void=>{console.log("hook");});test("sample",():void=>{});console.log(runTests());}
// hook
// PASS sample
// 1 passed, 0 failed, 0 skipped
// 0
```

Ownership: the callback transfers into the registry and is released when the registry is consumed. Owned mutable captures persist across hook invocations. Module storage is borrowed only while each hook runs, so hooks and tests can read or update the same module state in execution order. Borrowed local captures and borrowed callback inputs cannot escape into the registry.
*/
export function beforeEach(callback:Owned<Function<void>>):void { registerHook(2,callback); }
/**
Registers a synchronous teardown callback after every test in the current suite and its descendants, or the current file at top level. Nested teardown runs before ancestor teardown; multiple hooks in a scope run in reverse registration order. The callback is reused across tests and retains its owned mutable state. Teardown still runs after a test assertion or beforeEach fails. All teardown callbacks are attempted; their errors join the test errors, and the test counts as one failure. Tests blocked by beforeAll do not run beforeEach or afterEach. Scopes with no tests do not run hooks. Takes one owned reusable zero-argument callback returning void; returns void and changes the pending hook registry. There are no optional arguments or defaults. Registration during test or hook execution is an error. Wrong arity or callback types are compile-time errors. Context parameters, cleanup return values, timeouts and async callbacks are unsupported.

@param callback - Owned reusable synchronous zero-argument callback returning void.

```tsrust
import {afterEach,test,runTests} from "std:testing";function main(){afterEach(():void=>{console.log("hook");});test("sample",():void=>{});console.log(runTests());}
// hook
// PASS sample
// 1 passed, 0 failed, 0 skipped
// 0
```

Ownership: the callback transfers into the registry and is released when the registry is consumed. Owned mutable captures persist across hook invocations. Module storage is borrowed only while each hook runs, so hooks and tests can read or update the same module state in execution order. Borrowed local captures and borrowed callback inputs cannot escape into the registry.
*/
export function afterEach(callback:Owned<Function<void>>):void { registerHook(3,callback); }

/**
Registers a synchronous setup callback for the current suite, including descendants, or the current file at top level. Runs once in registration order before tests in that scope. A failed setup stops later setup hooks, blocks descendant test bodies, reports those tests as failed, and still runs teardown for the scope whose setup was attempted. Scopes with no tests do not run hooks. This is an alias of beforeAll. Takes one owned reusable zero-argument callback returning void; returns void and changes the pending hook registry. There are no optional arguments or defaults. Registration during test or hook execution is an error. Wrong arity or callback types are compile-time errors. Context parameters, cleanup return values, timeouts and async callbacks are unsupported.

@param callback - Owned reusable synchronous zero-argument callback returning void.

```tsrust
import {before,test,runTests} from "std:testing";function main(){before(():void=>{console.log("hook");});test("sample",():void=>{});console.log(runTests());}
// hook
// PASS sample
// 1 passed, 0 failed, 0 skipped
// 0
```

Ownership: the callback transfers into the registry and is released when the registry is consumed. Owned mutable captures persist across hook invocations. Module storage is borrowed only while each hook runs, so hooks and tests can read or update the same module state in execution order. Borrowed local captures and borrowed callback inputs cannot escape into the registry.
*/
export function before(callback:Owned<Function<void>>):void { beforeAll(callback); }

/**
Registers a synchronous teardown callback for the current suite, including descendants, or the current file at top level. Runs once after the scope finishes, in reverse registration order. It also runs after a failed beforeAll in that scope. All teardown hooks are attempted even if earlier teardown fails. Hook failures are reported after the phase and make runTests and the test command fail. Descendant scopes blocked before setup do not run their hooks; scopes with no tests do not run hooks. This is an alias of afterAll. Takes one owned reusable zero-argument callback returning void; returns void and changes the pending hook registry. There are no optional arguments or defaults. Registration during test or hook execution is an error. Wrong arity or callback types are compile-time errors. Context parameters, cleanup return values, timeouts and async callbacks are unsupported.

@param callback - Owned reusable synchronous zero-argument callback returning void.

```tsrust
import {after,test,runTests} from "std:testing";function main(){after(():void=>{console.log("hook");});test("sample",():void=>{});console.log(runTests());}
// PASS sample
// hook
// 1 passed, 0 failed, 0 skipped
// 0
```

Ownership: the callback transfers into the registry and is released when the registry is consumed. Owned mutable captures persist across hook invocations. Module storage is borrowed only while each hook runs, so hooks and tests can read or update the same module state in execution order. Borrowed local captures and borrowed callback inputs cannot escape into the registry.
*/
export function after(callback:Owned<Function<void>>):void { afterAll(callback); }
"####;
