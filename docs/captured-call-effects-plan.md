# Captured callable writes

This continues Gate173 after e2f5289. It does not change the ownership model or accept the complete gate.

## Reproduced defects

The focused generic_call_capture_effects suite preserves eight incorrectly accepted calls after a captured callable changes from a one-argument declaration to a two-argument declaration. The cases cover named and arrow writers, transitive and recursive calls, conditional invocation, writes before throw, synchronous collection callbacks and argument evaluation. All eight precision controls pass: unused declarations, unused arrows, saved snapshots, explicit restoration, lexical shadowing, restoration inside a writer, callee snapshots before argument effects and declarations after a call.

A separate witness exposes a dependency: a nested function cannot explicitly name its enclosing generic type parameter. The lexical resolver lifts that function into an independent declaration, while generic specialization only knows the lifted declaration's own parameters. Do not hide this defect by changing the witness to an inferred result.

## Implementation gates

1. Preserve lexical generic environments across lifting and specialization. Keep inherited parameters separate from explicit call type-argument counts. Concrete instances must substitute the enclosing instance's arguments in nested annotations and capture types. Cover generic shadowing, multiple factory instances, nested recursion and native owned results. Simply appending inherited parameters to the public generic signature would produce incorrect arity and is not a repair.
2. Add an isolated callable-effect index keyed by declaration identity and lexical captured binding identity. A declaration contributes a body to the index, never an executed write. Preserve trusted arrow provenance and distinguish unknown callback contracts from known bodies. Reuse source graph evaluation rather than scanning textual writes into caller state.
3. Apply known invocation effects after arguments, using the saved callee snapshot. Summaries need normal and exceptional exits separately, ordered overwrites and finite joins. Recursive groups require a finite fixed point; transitive calls must not depend on declaration order. Unknown invocations conservatively affect reachable captured roots without invalidating unrelated local bindings.
4. Model synchronous built-in callback invocation through compiler contracts. Begin with forEach, including empty, singleton and unknown-length inputs. Complete other relevant callback methods with their short-circuit and cardinality rules before claiming collection-wide effects. Never execute a closure merely because it appears as an argument or declaration.
5. Finish editor diagnostics, related declarations and navigation. Preserve source spans through summaries. Run formatted successful sources, native ownership/failure/disposal controls and independent TypeScript checks. Extend the existing matched family only for newly executable behavior; rejection-only controls do not need artificial timing scenarios.

Each gate retains red witnesses, runs bounded affected compiler checks and the complete editor before acceptance. Dispatch changes require all eight depth canaries. Format the entire project before checkpoints. Keep failed and repaired evidence in SQLite with exact readback. Full Gate173 also retains optional expression regions, stored callable fields, concrete environment replacement and lifetime/context work.

## Current checkpoint

No compiler source is changed by this checkpoint. The isolated red run has three tests: one precision test passes; invocation effects and enclosing generic annotations fail. Both original and isolated failed logs are retained in SQLite. This is a test and planning checkpoint, not feature acceptance or measured performance.
