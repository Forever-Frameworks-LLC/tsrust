# Optional browser JavaScript bridge

Status: deferred design, requested by the owner on 2026-10-19. This is not an
implemented import facility. Continue the shared landing and component library
before implementing this bridge. Filename suffixes and configuration syntax remain
undecided; illustrative `.web.ts` / `.native.rs` / `.embedded.rs` examples do not
establish a module-resolution convention.

## Product contract

Use ordinary imports in `.tsr` and `.tsrx`. The compiler owns extraction, checked
bindings, dependency preparation and browser loading. Application code does not
register global `window` functions, insert script tags or manage loader handles.
Browser modules may use browser APIs internally.

Node is a build/development tool for type stripping. It is not an application
server runtime or a native/embedded runtime. Type stripping is not type checking,
Rust compilation, dependency bundling or a DOM implementation. Begin with local
ES modules and Node tooling; minification, tree shaking and package resolution
need separate justification. Never execute imported application JavaScript during
checking, completion or hover.

Keep core component behavior in shared compiled code with platform adapters. JS
is an optional browser integration, not the implementation dependency of the core
component library. Native and embedded apps do not acquire Node, a JS engine or a
hidden WebView to satisfy a browser import.

## Imports and target implementations

- Check every function used against every enabled `ui.target`, including targets
  other than the current build target. A module existing on a target does not
  imply that every export exists there.
- A shared module can supply browser JS and compiled implementations behind one
  checked public contract. Resolve target implementations deterministically; reject
  ambiguity and missing exports. Do not silently choose a browser implementation
  for native or embedded.
- Required behavior needs an implementation on each target that uses it. Missing
  printing, editing, persistence or other functionality is a compile error.
- A decorative enhancement may intentionally be omitted on a target through an
  explicit project-owned implementation. Never generate a no-op fallback or infer
  optionality from a function returning void. A no-op implementation is a product
  decision visible in source and documentation.
- Target implementations must agree on exported signatures, data representation,
  error behavior, async behavior and ownership. A matching signature alone does
  not establish equivalent behavior; run shared behavioral cases on each backend.
- Ordinary synchronous imports load before constructing the dependent component.
  Load at route/component boundaries where useful. Do not silently change a
  synchronous function into an asynchronous first call. An explicitly asynchronous
  contract uses the language's checked task/cancellation rules.
- Resolve static data and type-only imports separately from executable imports.
  Cycles, top-level side effects, initialization failures and unsupported module
  forms require explicit policy and diagnostics before acceptance.

## Boundary and lifetime requirements

Start with a bounded, explicit ABI. Preserve exact integer widths, decimal values,
Unicode, binary data and nullable distinctions. In particular, never turn a 64-bit
integer into a JS number silently. Unsupported representations must reject with a
readable conversion requirement. DOM objects cannot become ordinary shared Nodes
or checked resources by declaration alone.

Retained browser callbacks belong to a component or another checked owner. Release
listeners, callback registrations and other resources on disposal. Late module
loads or promise completions cannot resurrect disposed components or extend state
ownership. Describe cancellation honestly: a noncancelable JS operation may finish,
but its disposed consumer cannot receive results. Do not claim cancellation or
resource release from a type annotation without an independently observable witness.

## Compiler and LSP are one implementation section

The owner explicitly requires immediate feedback when a function is absent from
one of the project's selected runtimes. This is a prerequisite, not later polish.

- Use the compiler's resolved export/capability graph for diagnostics, completion,
  hover, signature help, definitions and builds. Do not create a separate editor
  approximation of target support.
- Report an unavailable function at its import or use, name every affected target,
  identify the implementation/export that is missing, and include related source
  locations for the import chain and target configuration. Explain the fix: supply
  the missing implementation/export, use a capability available on all selected
  targets, or deliberately change project targets. Never suggest disabling a target
  as an automatic fix.
- Detect incompatible parameter/return types, ownership, errors and async contracts
  across implementations before execution. Point to both conflicting declarations.
- Hover shows per-target availability and definitions. Completion clearly marks
  unavailable exports rather than presenting them as portable. Signature help
  must not borrow a convenient signature from another target.
- Definition navigation exposes the selected implementation and the other configured
  implementations. Rename/find references cover the shared export relationship.
- React to unsaved module, declaration and project-target edits. Invalidate affected
  target/export caches when flags, source, declarations or resolution inputs change.
  No Cargo build, Node execution or network request is needed to report availability.
- Resolve availability from cached module/export facts before expensive body and
  ownership analysis. A slow full-project check must not delay a missing-target
  diagnostic. Retain useful availability results while the edited function body is
  incomplete. Measure edit-to-diagnostic latency on a representative multi-target
  project, including changing one export and toggling one target; do not claim
  instant feedback from a small isolated hover test alone.
- Checks and editor operations remain read-only. A code action may propose a concrete
  adapter scaffold, but it cannot silently install dependencies, create fake behavior,
  modify targets or claim that a scaffold implements the missing feature.

Example diagnostic shape (illustrative):

```text
printInvoice is unavailable for target "embedded".
Imported through src/app/invoice-actions.tsrx -> src/invoice/printing.
Provide an embedded implementation exporting printInvoice with the shared
contract, or explicitly remove this feature from the embedded application.
Related: tsrust.config.json ui.target.embedded; printing browser declaration.
```

## Acceptance before publishing support

Implement compiler, runtime boundary, editor, CLI/dev watching, examples and
documentation together. Batch targeted checks after that section is complete.
Include web-only, web/native and web/native/embedded projects; a missing module,
missing single export, incompatible signatures and unsaved target changes; normal
and lazy initialization failures; independent callback/resource release; disposed
load/promise consumers; exact conversion failures and native/embedded rejection.
Verify formatted application examples and real browser execution without server-side
JS. Keep existing resource, cancellation and cleanup evidence; rerun only affected
checks. Do not advertise the bridge until these requirements pass.
