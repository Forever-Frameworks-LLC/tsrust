# Compiler, modules, and diagnostics

[Documentation index](../../README.md) · [Proposed roadmap](../../roadmap.md) · [Source section map](../../plans/section-map.md)

## Current implementation and retained planning

**Implementation decision:** The compiler and implemented language server are written in Rust. They share the [analysis library](../../../crates/tsrust-compiler/README.md). `tsrust` translates `.tsr` to `.rs`; `rustc` compiles that output into a native executable. Python is not part of the compilation pipeline. The language server provides project diagnostics, recovery, completion and navigation using the shared compiler. See [editor setup](../../../website/src/content/docs/start/editor.md).

**Planning status:** The workstream framing below is retained research. It does not describe missing current compiler/editor functionality; use the linked implementation and editor guides for current behavior.

**Proposed phase:** Core proof.

**Scope:** Define the frontend, analyses, intermediate forms, Rust generation, modules, and source diagnostics.

**Planning deliverable:** A compiler architecture and source-to-diagnostic mapping demonstrated by the core prototype.

**Dependencies to coordinate:** [ownership-and-memory](ownership-and-memory.md), [type-system](type-system.md), [primitives-and-collections](primitives-and-collections.md)

## Current experiment

[Milestone 26](../../milestones/26-recursive-functions.md) adds self and mutual named-function recursion for data parameters/results. Monotone access summaries stabilize before a fresh definitive ownership analysis. Transfers, loans, captures, callbacks outside recursive signatures, iteration, runtime failure ranges, and source guards remain enforced. Milestone 27 subsequently adds ordinary reusable/single-use parameters. Milestone 28 subsequently extends owned parameters too. Milestone 29 subsequently resolves single-origin callable results; termination and tail-call behavior remain outside the guarantee. Acyclic sources retain their existing callee-first analysis path. Public results contain only definitive metadata and require no backend process. Incremental invalidation, partial-source recovery, modules, multiple diagnostics, and position-encoding conversion remain separate language-server prerequisites.

[Milestone 27](../../milestones/27-recursive-callbacks.md) extends self and mutual recursion to reusable and ordinary single-use callback parameters with non-callable results. Exact argument/result/access contracts, declaration aliases, single-use transfers, capture loans, shared iteration, and stable data ownership summaries remain enforced. Fresh closed arrows/factory results and callback permutations are supported. Definitive typed-call analysis bounds each recursive callback function to 128 distinct concrete environment type vectors; repeated values and erased lifetimes count once. Non-owned callback parameters still cannot be captured. Milestone 28 subsequently extends owned callback parameters; milestone 29 subsequently adds single-origin callable-result cycles.

[Milestone 28](../../milestones/28-recursive-owned-callbacks.md) extends named recursion to owned reusable/single-use callback parameters with non-callable results. Abstract templates preserve exact transfer, retention, invocation, and argument contracts. Concrete data-result bodies drain iteratively through the existing global 128-specialization cache; mixed ordinary parameters retain the per-function vector guard. Source arrows with matching capture layouts and callable contracts reuse native types while each construction applies fresh values and loans. Growing environment types receive source diagnostics. Milestone 29 subsequently adds single-origin callable-result cycles.

[Milestone 29](../../milestones/29-recursive-callable-results.md) resolves self/mutual callable-result cycles to one concrete environment origin before definitive checking. Source branch/declaration order does not select native evaluation order. Declared callable shapes support provisional local invocation, including lazy nested factories, but cannot supply concrete returns or captures. Final bodies enforce transfers, invocation, exact signatures, all callback contracts, and source loans; discovery-only environments are removed before public metadata or emission. Mixed/missing origins and recursive inline environment storage remain unsupported.

[Milestone 30](../../milestones/30-array-removal.md) adds ordered `array.remove(index)` with an exact element result. Scalar, string, named-record, and nested-array values transfer out without cloning; the remaining array retains ownership and order. Removal infers root-owner mutation and preserves argument, capture, iteration, callback, recursive, and selection loans. Named receivers permit index effects before final mutation; selected field/index parents stay stable. Negative/out-of-range indices fail with the full removal call's source byte range under every optimization profile. Milestone 31 subsequently adds direct field moves. Optional/range removal and mutable/consuming iteration remain separately scoped.

[Milestone 31](../../milestones/31-record-field-moves.md) adds moves of direct named string/record/array fields with per-path availability. Siblings remain usable; missing fields block overlapping or whole-record access with related transfer locations. Exact field/parent replacement and whole let rebinding restore availability. Branch joins and loop edges retain missing-field state. Field consumption infers owning root parameter/capture contracts; detached argument values release source loans, while borrowed field arguments and existing capture/iteration/selection loans remain conservative. Native places/drop flags preserve initialization, evaluation, and bounds behavior without runtime field-state storage. Indexed and iteration elements remain borrowed; disjoint field loans and partial environment captures remain separately scoped.

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-12"></a>

## 12. Compiler Architecture

Initial architecture:
Source
  ↓
Lexer / Parser
  ↓
AST
  ↓
Name Resolution
  ↓
Typed AST
  ↓
Ownership / Escape Analysis
  ↓
Language IR
  ↓
Rust-oriented IR
  ↓
Generated Rust
  ↓
rustc
  ↓
Native binary
Initially, generating Rust source is acceptable and likely desirable.
Benefits:
* leverage rustc
* leverage LLVM
* leverage platform support
* leverage mature native code generation
* simplify compiler bootstrap
Do not make direct LLVM generation an early requirement.
Research the point at which a custom backend would become beneficial.
The compiler should retain source maps or equivalent metadata so compiler/runtime errors refer to original source files rather than generated Rust.

<a id="core-22"></a>

## 22. Compiler Diagnostics

Compiler errors should be a major differentiator.
Errors should:
* use source-language concepts
* explain why
* point to both cause and consequence
* suggest likely fixes
* avoid leaking generated Rust details
* provide deeper explanation via optional flags
Example:
error: value already transferred

12 | const backup = user;
                    ---- value transferred here

15 | console.log(user.name);
                   ^^^^ `user` is no longer available

Try one of:

const backup = clone(user);   // independent value
const backup = Shared(user);  // shared ownership
Potential advanced command:
lang explain E1042

<a id="core-31"></a>

## 31. Module System

Prefer a simple static module system.
Potential syntax:
import { User } from "./user";
import { App } from "http";
Research:
* filesystem conventions
* package imports
* barrel files
* circular dependencies
* tree shaking
* conditional compilation
* platform-specific modules
Avoid CommonJS-style dynamic loading as a core mechanism.
