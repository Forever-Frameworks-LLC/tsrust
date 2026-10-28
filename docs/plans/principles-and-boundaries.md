# Principles and boundaries

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Groundwork.

**Scope:** Define compatibility boundaries, early non-goals, and decision priorities.

**Planning deliverable:** One agreed priority order and a compatibility/non-goal policy.

**Dependencies to coordinate:** [product-vision](product-vision.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-49"></a>

## 49. Source Compatibility

Decide how closely valid TypeScript should resemble valid source in this language.
Potential goal:
A developer should be able to paste ordinary strict TypeScript business logic and have most syntax parse, even when semantic changes are required.
Do not promise arbitrary TypeScript compatibility.
Potential compatibility levels:
Syntax-compatible
API-compatible
Semantically compatible
Runtime-compatible
Clearly distinguish them.

<a id="core-50"></a>

## 50. Initial Non-Goals

Suggested early non-goals:
* browser DOM compatibility
* full npm compatibility
* full Node API compatibility
* dynamic eval
* JavaScript prototype semantics
* JIT compilation
* runtime monkey patching
* arbitrary native addons
* reflection-heavy frameworks
* frontend UI framework support
* exact JavaScript numerical semantics
* exact JavaScript event-loop semantics
These can be revisited later.

<a id="core-62"></a>

## 62. Decision Rule

Whenever choosing between JavaScript compatibility and the language’s foundational goals, evaluate in this order:
Does it preserve memory safety?
Does it avoid tracing GC?
Can ownership remain understandable?
Can the compiler optimize it effectively?
Does it preserve predictable performance?
Can developers understand the behavior?
Does it retain familiar TypeScript ergonomics?
Can it interoperate cleanly with the ecosystem?
Exact JavaScript compatibility comes after these concerns.
