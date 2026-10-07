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

## Lexical scope foundation

lexical_generic_scopes.rs derives inherited type parameters from the lifted declaration catalog, independently of value capture plans. It walks lexical owners iteratively, keeps original declaration spans, applies nearest-scope shadowing and returns outer-to-inner ordering for stable instance keys. A nested declaration's own type parameters remain unchanged. Two unit controls pass after formatting their source: mixed nesting/shadowing/siblings and capture-free inheritance.

This module is a prerequisite, not yet wired into specialization. The concrete integer/string echo witness remains red. Do not accept generic transport merely because scope discovery works. Next, contextualize each lifted declaration using its enclosing instance's lexical bindings, substitute inherited annotations, and preserve the nested declaration's own generic signature. Cache contextual identities separately from explicit argument identities. Retain the original declaration identity for named capture requests and editor spans. Rebase lexical identities for linked modules; do not use spelling or enclosing value capture types as a substitute for type identity.

Materialize capture-free and captured nested declarations through the same lexical path, including uncalled declaration validation. Calls and value uses must resolve the same contextual declaration. Recursive peers need their contextual mapping installed before bodies are walked. A nested function's own generic variables shadow the enclosing variables and must not be replaced by the enclosing substitution. Native tests must prove independent integer/string instances and shadowed explicit generics before this prerequisite becomes accepted behavior.

## Contextual specialization implementation (not accepted)

lexical_generic_instances.rs now contextualizes lifted declarations before each enclosing instance is drained. It substitutes inherited annotations while leaving the declaration's own explicit type parameters unchanged. Recursive peer names are installed as a group before body traversal. Contextual generic templates retain inherited bindings for later explicit or inferred application. Concrete applications retain their original source declaration and a separate concrete lexical owner; named capture preparation uses that owner rather than searching for a drained source template.

The original unused nested annotation witness and concrete integer/string echo now pass. Formatted native controls also cover capturing reads, forwarded peers, explicit and inferred nested generic calls, saved function values, recursion, uncalled captures and deeper nesting. Editor checks retain source parameter help and definition locations while outer calls expose concrete result types. The original invocation-write rejection test is still red and deliberately remains separate from this successful lexical batch.

This is implementation progress, not feature acceptance. Complete linked-module identity, capturing generic value/escape and ownership/disposal controls, independent TypeScript evidence and matched workload support before acceptance. Source rejections must still validate every unused lifted declaration. Preserve bounded checks already completed and supplement affected checks after repairs instead of restarting all suites.

## Focused lexical transport acceptance

The lexical dependency now has focused acceptance; see handoff-2026-10-07-lexical-generic-transport.md. Linking preserves inherited type names and validates nested explicit applications in the innermost function. Escaped ordinary and generic values retain independent owned captures, and O0/O3 failure disposal observes each original owner once. Public arity and repeated consumption rejection controls pass. Current bounded evidence is 177 compiler tests and the whole 327-test editor, plus five matched cases with 124 observations, all untimed. The original eight invocation-effect reds remain required implementation work.
