# Project diagnostics milestone

Required after S08 collections acceptance and before starting S09. The user
requested project-wide checking and multiple errors per file, including files not
open in the editor. This is additional roadmap work, not S08 acceptance evidence.

## Required behavior

- Discover project `.tsr` files recursively, with the formatter's existing safe
  exclusions as the starting point. Avoid build outputs, dependencies and symlink
  cycles. Include unopened files and loaded module dependencies.
- Report multiple independent diagnostics per file. Recover at safe parser and
  analysis boundaries, suppress misleading cascades, and preserve ownership
  checking. A recovered diagnostic pass must never authorize native execution.
- Publish diagnostics for all affected project files. Recheck after edits, saves,
  file creation/deletion and dependency changes. Clear stale diagnostics, honor
  unsaved editor buffers and prevent older check results from replacing newer ones.
- Provide equivalent project checking through the CLI, with deterministic ordering
  and nonzero status when errors exist. Preserve useful single-file commands.
- Cover multiple syntax/type/ownership errors, multiple files, import failures,
  stale-result cancellation and editor-visible recovery. Refresh installed consumers
  and verify open and unopened files before accepting the milestone.

## Later configuration and scaffolding direction

The user is open to project configuration and `tsrust new <name>` scaffolding.
The current recommendation is one `tsrust.config.json` with build, check and format
sections, discovered by walking to the nearest project root. Candidate settings:
entry points, include/exclude paths, compiler parameters, binary name, targets,
indent width and quote style. Scaffolding would write standard defaults.

This configuration shape remains a design recommendation, not an implemented or
accepted interface. Split files only when actual use justifies them. Configuration
must not silently weaken ownership rules or change current project formatting.

## Current implementation checkpoint

The user selected `check` as the sole compiler checking command; `lint` is removed.
`check` accepts files and directories and defaults to the current directory. Shared
formatter/check discovery excludes `.git`, `node_modules`, `target` and symlinks,
canonicalizes and deduplicates files, and sorts diagnostics by source location.
Checking continues after errors in other files and deduplicates shared dependency
errors. The `main` entry point is optional in the language. Declaration-only and
empty files check, build and run; running without `main` exits successfully without
calling any declared functions. An existing `main` must still have no parameters
and no return value. Imported modules do not supply the entry point.

The editor command is named **Tsrust: Check File** and invokes `check`. This
checkpoint does not complete the milestone: multiple diagnostics within one file,
workspace-wide LSP scheduling/publication, stale-result handling and installed
consumer verification remain pending. Configuration/scaffolding is still future
work. Preserve the complete milestone requirements above.

Focused checkpoint validation: 92 project tests, six recursive-check/optional-entry
tests, 28 formatting tests and one editor library-diagnostic test pass. The whole
project formatter changes zero files and zero fingerprints. Optional-entry tests
execute formatted empty/declaration-only sources, preserve existing entry signature
rejections, and prove a dependency's `main` is not invoked. The editor no longer
suppresses entry diagnostics for library buffers. Closed diagnostic logs use the
`tsrust-optional-main-*` and `tsrust-project-check-*` names in SQLite.
