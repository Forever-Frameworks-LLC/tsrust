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
The current recommendation is one `tsrust.config.json` with build, lint and format
sections, discovered by walking to the nearest project root. Candidate settings:
entry points, include/exclude paths, compiler parameters, binary name, targets,
indent width and quote style. Scaffolding would write standard defaults.

This configuration shape remains a design recommendation, not an implemented or
accepted interface. Split files only when actual use justifies them. Configuration
must not silently weaken ownership rules or change current project formatting.
