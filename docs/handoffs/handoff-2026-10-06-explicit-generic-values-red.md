# Explicit generic value applications: red checkpoint

Read AGENTS.md, docs/development-workflow.md and the generic function/value adapter
plans. Accepted implementation remains fe4d339. This checkpoint adds a real failing
requirement and strict TypeScript reference; no compiler source is changed.

The explicit-callable-payload fixture retains one captured identity<T> environment
and invokes identity<Function<int,[value:int]>> twice with different arrow payloads.
Assertions require returned callbacks to produce 8 and 9. Strict TypeScript/Node
passes with counter output 1,2. Native rejects the second call during source cache
drain: type, Explicit type arguments require a declared generic function. The
retained failing test is generic_callable_payloads_support_explicit_source_type_applications.

## Cause and implementation plan

Generic functions Cache retains explicit argument annotations by call span. Its
source drain rejects explicit applications whose names are lexical bindings rather
than known declaration templates. That happens before checked origin discovery.
Simply deleting the guard would silently infer types and ignore explicit arguments;
that is not an implementation.

Retain explicit applications for checked value dispatch. Resolve a bound value's
original generic declaration before validating type-argument count and source
annotations. Substitute enclosing generic bindings into retained call metadata when
materializing source bodies. Pass explicit checked type arguments into deferred
preparation rather than inferring a replacement. Reuse complete callable annotation
reconstruction, record materialization and checked/captured cache identities.
Contextual argument checking must use the explicit specialization, preserving
required/default/rest arity, access and result constraints. Ordinary nongeneric
values must still reject explicit type arguments, including unused/dead declarations
under the existing source validation policy. No runtime argument/default/factory
execution may occur during preparation. Keep this work in dedicated preparation
helpers outside shared recursive frames.

Add scalar and callable explicit application witnesses, wrong-count/type/access and
same-T mismatch rejections, explicit versus inferred method reuse, result-only
arguments, captures across int/string types and generic enclosing substitutions.
Cover project/import aliases and concrete editor signature/navigation. Verify once
and borrowed capture restrictions remain unchanged. Extend matched support, guards
and independent oracle before acceptance. Run affected controls, eight depth canaries
and whole editor after dispatch changes. Preserve original failure and reference
bytes in SQLite. No timing or broad compiler regression is needed for this chunk.

## State and continuation

The failed native log and passing strict reference are stored exactly in SQLite
under /private/tmp/tsrust-explicit-callable-payload-red.log and
/private/tmp/tsrust-explicit-callable-reference.log. Loose logs and generated Node
output are removed. Five consumers still represent fe4d339. Full Gate173 remains
open. Four owner fixture edits remain untouched and unstaged.

Continue Tsrust from this red checkpoint. Implement explicit type applications on
retained generic values using their original declaration and checked capture context.
Never discard explicit arguments or reconstruct factory environments. Read workflow
and both generic value plans; preserve owner fixtures. Complete focused native/Node,
editor, matched support, depth checks, SQLite evidence and explicit-path commit.
