# Error handling

Use `try` to run code that can fail, `catch` to handle its error, and `finally` to run cleanup before leaving the region. A try statement needs a catch clause, a finally clause, or both. Catch may omit its binding when the error value is not needed.

```ts
function value(): int {
    try {
        throw "failed";
    } catch (error) {
        console.log(error);
        return 4;
    } finally {
        console.log("cleanup");
    }
}

function main() {
    console.log(value());
}
```

Output:

```text
failed
cleanup
4
```

Finally runs after normal completion and before a pending return, throw, break or continue takes effect. A return or throw produced by finally replaces the pending completion. Cleanup should normally complete without producing another exit so the original result or error is preserved.

Catch handles failures from its try body, including errors propagated by called functions. An error thrown by catch proceeds through finally and then to an enclosing handler. An error thrown by finally goes to an enclosing handler; the catch belonging to that same try statement does not handle it.

When nothing in the try body can fail, the catch body never runs. Its statements are still checked like any other code, so a type or name error there is reported. In such a handler the error name has the type of a runtime error, with `name` and `message`.

## Error values and ownership

An explicit throw can carry a supported checked value. The compiler infers the error alternatives that can reach a handler. Narrow alternatives before using operations specific to one type, for example `typeof error === "string"` before a string method. Error contracts must resolve; unknown callable contracts produce a diagnostic instead of silently hiding possible failures.

Throwing an owned value transfers it. Catch receives the propagated value, and rethrowing transfers it again. Existing borrowing and move restrictions still apply. Error handling does not introduce implicit cloning or extend a borrowed value's lifetime.

A pending return is evaluated before finally runs. Cleanup sees the current
collection contents, including entries added after a clear. If cleanup completes
normally, the saved return value is returned; a return or error from cleanup
replaces it. Borrowed callback captures released by clear can be mutated after
normal cleanup on a path that continues past the try statement. The compiler
preserves the owners needed for cleanup without cloning them. A pending break
or continue also runs cleanup before reaching its loop destination. A break,
continue, return or error from cleanup replaces the pending exit. Normal
fallthrough can mutate captures released by cleanup before the next iteration. An initially
empty Map declared immediately before a while loop can also release its
callback captures when every iteration clears it in unconditional finally.
It remains empty after break or zero iterations. Maps that retain entries
across iterations still require additional storage-boundary support.




## Runtime failures

Prepared exception paths convert supported arithmetic and array-bound failures into structured error values. These values have `name`, `code`, `message`, `source`, `start` and `end` fields. Project execution identifies the original module and its byte range; arithmetic ranges identify the failing operator. Source-only compiler APIs use a source placeholder when no filename is available.

An uncaught checked error runs applicable finally blocks, reports the error, and exits unsuccessfully. This does not guarantee recovery from allocation failure, process termination, native defects or every external native binding. Native APIs require an audited error contract before they can participate in checked propagation.

## Testing and cleanup

Use `std:testing` assertions in executable checks. A typed error escaping a registered test is reported as a failure, teardown runs, and later tests continue. A reusable stored callback can transfer into an owned registration; it cannot be used again after transfer. A single-use callback cannot satisfy a reusable callback contract.

The current runner is synchronous. Async execution and process lifecycle handlers are separate future work. Completion transport preserves distinct return, error and loop exits so later suspension support can retain those semantics.

## Array callbacks

Checked errors from array callbacks propagate through the surrounding try region.
This includes map, filter, flatMap, predicates, reductions and sorting. A callback
error stops further callback execution; applicable finally blocks still run.
Normal short-circuiting remains unchanged. Callbacks can borrow owned elements,
and returned owners retain each method's usual ownership rules.

Sorting with a fallible comparator compares borrowed values before changing the
receiver. If comparison fails, the receiver retains its original order. Successful
comparison applies a stable permutation without cloning elements. This path uses
additional index storage. Pure comparators retain the existing direct sorting path.
