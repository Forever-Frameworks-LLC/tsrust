# Nested named functions

Declaration scope, forward calls, result inference and recursive declarations now
live in [Declare nested functions](https://forever-frameworks-llc.github.io/tsrust/language/functions/#nested-functions).

Captured state, direct calls, stored callback loans and returned factories now live
in [Capture state in nested functions](https://forever-frameworks-llc.github.io/tsrust/language/functions/#captures).
Array storage also has a [dedicated guide](https://forever-frameworks-llc.github.io/tsrust/language/function-values/#callable-arrays).
Optional/default/rest behavior for stored captured functions now lives in
[Preserve parameters in captured callbacks](https://forever-frameworks-llc.github.io/tsrust/language/functions/#parameters-of-captured-functions).
Concrete specialization and unannotated generic values now live in
[Specialize a generic function value](https://forever-frameworks-llc.github.io/tsrust/language/generics/#specialization)
and [Keep function values generic](https://forever-frameworks-llc.github.io/tsrust/language/generics/#generic-function-values).
The source-flow, selection and replacement sections below are still being audited.

Adapting existing generic environments now lives in
[Adapt an existing generic callback](https://forever-frameworks-llc.github.io/tsrust/language/generics/#adapting-existing-generic-values).

Fresh/selected construction, inferred results, aggregate destinations and nullable
owning callbacks now live in [Adapt fresh generic factory results](https://forever-frameworks-llc.github.io/tsrust/language/generics/#fresh-generic-factories).

## Track callable contracts through source flow

Known callable argument counts are checked even inside generic declarations that
are not instantiated. Loops retain contracts from previous iterations and from
for updates. A `continue` runs the for update or post-test condition; a `break`
reaches the loop exit. Labeled exits target their enclosing labeled loop or block.
A saved alias retains its initializer's contract set, while a definite assignment
replaces the current set. Each known reachable signature must accept the call.
These source checks preserve ownership rules and do not make incompatible concrete
callable environments interchangeable. Try, catch and finally retain these contracts through normal and abrupt exits.
Cleanup executes before return, throw, break or continue resumes. A cleanup that
returns or throws replaces the pending completion. Failed checked expressions
enter handlers before destination assignments commit. Expression writes and
captured effects remain part of the source-flow audit.


Named assignment invocation now lives in
[Call a named assignment result](https://forever-frameworks-llc.github.io/tsrust/language/functions/#call-a-named-assignment-result).
Lexical type inheritance and callable transport through factories now live in
[Use enclosing generic types](https://forever-frameworks-llc.github.io/tsrust/language/generics/#enclosing-type-parameters).

## Selecting owned functions

A conditional expression can select between different owned functions with a
compatible parameter and result contract. The selected function keeps its own
capture values. Only the selected branch executes, and selecting a function does
not invoke it.

```tsrust
function main() {
    const offset = 3;
    const chosen = false
        ? ((value: int): int => value + offset)
        : ((value: int): int => value * offset);
    console.log(chosen(4));
}
```

This prints `12`. The selected function owns its copied scalar capture. If either
alternative consumes its captured owner when invoked, the common function value
also permits only one invocation. Selection does not implicitly clone an owner.
Borrowed environments use the scoped family described below. Polymorphic
function alternatives remain pending. Neither is converted to an owned function
implicitly.

## Selecting scoped borrowed functions

A conditional can also select concrete functions that borrow local data. The
selected function keeps its original captures and invocation access. Repeated
calls update the original captured variable. The family cannot outlive borrowed
storage or be transferred into an owning callback contract. Keep its handle in
an inner scope when later code needs exclusive access to the captured variables.

```tsrust
function main() {
    let left = 0;
    let right = 10;
    {
        const first = (): int => { left += 1; return left; };
        const second = (): int => { right += 2; return right; };
        const selected = false ? first : second;
        console.log(selected(), selected());
    }
    console.log(left, right);
}
// 12 14
// 0 14
```

Only the chosen alternative executes. Selection preserves each alternative's
argument defaults and callback failures. Readonly families can be lent to
ReadOnly<Function<...>> parameters; mutable families can be lent to
Mutable<Function<...>> parameters. Selection does not clone captured data or create
shared mutation cells. Consuming borrowed alternatives, polymorphic selection and
replacement across distinct polymorphic origins remain pending.

## Single-use callbacks and capture lifetimes

`Once<Function<Result, [arguments]>>` permits one invocation, including an invocation
that consumes owned captures. A concrete named function that only invokes its
callback can also accept an environment borrowing values from its caller. The
compiler checks the function body and forwarding destinations before admitting
those borrows. Returning or storing a callback requires an owning environment;
an unknown destination keeps that requirement. A callback signature used as a
higher-order value does not itself prove that its implementation cannot retain an
argument.

```tsr
function run(action: Once<Function<string>>): string {
    return action();
}

function retain(action: Once<Function<string>>): Once<Function<string>> {
    return action;
}

function main() {
    const values = [1];
    const text = "Pat";
    console.log(run((): string => {
        console.log(values.length);
        return text;
    }));
    console.log(values.length); // values remains available after run.

    const savedText = "Sam";
    const saved = retain((): string => savedText);
    console.log(saved());
}
// 1
// Pat
// 1
// Sam
```

The returned string transfers out of the callback. The first callback borrows the
array only for the scoped call. The retained callback owns its string so it remains
valid after `retain` returns. A second invocation of either single-use callback is
rejected. These checks do not insert an implicit clone.

## Replacing capture-free generic values

A mutable local may receive a fresh reference to another capture-free generic declaration. The replacement keeps its own behavior. Calls validate declarations reachable on that path; a saved value keeps the earlier declaration. Assignment does not implicitly clone an owned callable: initialize the mutable local again after transferring it into a saved local.

```tsrust
function first<T>(value: T): T { return value; }
function second<T>(left: T, right: T): T { return right; }
function choose(flag: boolean): int {
    let action = first;
    const saved = action;
    action = first;
    if (flag) { action = second; }
    if (flag) { return saved<int>(3) + action<int>(4, 7); }
    return saved<int>(3) + action<int>(4);
}
// choose(false) is 7; choose(true) is 10.
```

A call reachable through several declarations must have compatible prepared arguments and results. Each declaration receives its own checked body and error transport. Borrowing the callee prevents replacement during argument evaluation. Fresh references to capturing generic declarations may replace compatible generic values. Each environment retains its captured values and independently checked body. Existing conflicting capture loans still reject replacement. Owned factory calls preserve point-sensitive capture state across repeated calls and independent instances. Broader owned environment composition remains under development.

When reachable declarations use the same input type with different access modes, the call requires the strongest access of those declarations. For example, a reading arm and a consuming arm make the argument owned for that call. Each selected arm borrows or moves the single prepared value as needed. This requirement also appears in signature help. Omitted defaults retain the selected declaration’s initializer; rest values retain source order without implicit clones.

Compatible selected generic declarations may receive nullable scalar arguments through different read or consuming parameter access. Calls use the strongest reachable access, then adapt the value for the selected body. Callback arguments must have the same structural signature and ownership contract; each selected body retains its own behavior and failures. A callback returning a different type or requiring different callable ownership remains incompatible.


## Transferring mutable factories through arrays

An owned callback can remain reusable when every checked completion restores a captured value that the call transfers out. Each factory instance retains its own capture state. Removing a callback from an array with `shift()!` or `pop()!` transfers that callback; the assertion raises an error if the array is empty. Reinserted callbacks retain updates made before reinsertion. Indexed array values remain borrowed views and do not provide this ownership transfer.

```tsrust
function one<T>(value: T): T { return value; }
function two<T>(value: T, other: T): T { return other; }

function factory<A>(initial: A) {
    let action = initial;
    return (restore: boolean) => {
        const previous = action;
        if (restore) { action = one; }
        else { action = two; }
        return previous;
    };
}

function main() {
    const instances = [factory(one)];
    instances.push(factory(one));
    const first = instances.shift()!;
    const second = instances.shift()!;
    const original = first(false);
    const independent = second(true);
    const changed = first(true);
    console.log(original<int>(1), independent<int>(2), changed<int>(3, 9));
}
```

This prints `1 2 9`. The first call returns its previous one-argument callback and installs the two-argument callback. The second factory still returns its original one-argument callback. Calling the first factory again returns its updated callback. No captured owner is implicitly cloned. A missing restoration branch or a throw before restoration retains the consuming invocation requirement.

Owned callbacks may also capture another owned factory callback. Transferring the outer callback from an array preserves the child callback state for that instance. Independent instances do not share child updates. Borrowed lexical captures retain their existing shared identity and lifetime restrictions.
