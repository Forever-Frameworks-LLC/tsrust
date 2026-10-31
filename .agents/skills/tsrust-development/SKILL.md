---
name: tsrust-development
description: Resume tsrust compiler, LSP, native UI and documentation work from existing acceptance evidence, with local checkpoints and deliberate remote validation.
---

# Tsrust development

Read root and applicable nested AGENTS.md, docs/development-workflow.md, and
docs/agent-progress.md first. Read the current continuation handoff, relevant
plans and acceptance records rather than treating chronological notes as a queue.
Use `.agents/skills/ponytail/SKILL.md` for implementation and
`.agents/skills/caveman/SKILL.md` for chat. Keep persisted prose complete.
Use other installed skills only when their actual workflow fits the section;
do not introduce React or a replacement frontend framework into std:ui work.

## Resume and scope

Inspect Git status, diffs and recent commits. Preserve owner edits. One agent
writes this tree at a time. Verify progress claims against source and evidence.
Existing paths and running processes from macOS handoffs are historical: discover
the Windows environment, pinned tools, available memory and linkers locally.
Do not mark unavailable macOS/Linux/native-desktop checks as passed on Windows.

docs/agent-progress.md is the single handoff note; keep it current and do not
create separate handoff files. When the
owner invokes the continuation goal, investigate memory with bounded resources
before resuming Table acceptance or broad editor/preview work. Record evidence
and stop conditions before launching a reproducer. Inspect retention/invalidation
and queued analysis; do not hide growth with arbitrary caps or weaker coverage.

Derive a finite completion checklist from current plans, especially
docs/plans/documentation-and-component-library.md, UI_VISION.md,
docs/plans/compiler-completion-plan.md and docs/roadmap.md. Separate required
scope from proposals, explicit deferrals and design decisions. Do not silently
expand scope to arbitrary TypeScript compatibility or all future vision items.

## Work loop

Choose the next dependency-ready coherent section. Trace its compiler/runtime,
editor, examples, documentation and benchmark implications. Implement the section
and focused regression witnesses before batched affected checks, as required by
the development workflow. Repair affected failures and preserve passed evidence.
Reserve full regressions/timing for meaningful feature-family milestones.

Use one Cargo target with CARGO_INCREMENTAL=0 and CARGO_BUILD_JOBS=1. Run the
required whole-project formatter before source checkpoints; if tools are missing,
record that blocker instead of bypassing formatting. Reuse existing SQLite
analytics tools and verify imports before removing closed loose records.

Maintain docs/agent-progress.md before context exhaustion or any handoff: source
identity, active section, changed files, exact checks/results, pending checks,
resource observations, blockers and the next executable action. Keep it concise;
link evidence rather than duplicating it. Distinguish implementation, acceptance,
measurement and remote validation. Continue to the next section while useful work
is available; repeated no-progress attempts require a changed hypothesis or a
specific blocker, not an endless retry. Never claim completion with required
checks unavailable or pending.

## GitHub validation and release

Local commits are the normal durable checkpoints; elapsed time never justifies a
push. Do not alter CI triggers to hide failures or use skip-ci as a substitute
for validation. Never force-push, merge or publish a release merely to checkpoint.

Before a remote validation, record the exact commit, locally completed checks,
the specific uncertainty local tools cannot resolve (for example Linux/macOS
packaging or hosted runner behavior), the smallest appropriate workflow, and the
last run for equivalent inputs. Batch a coherent milestone and reuse unchanged
evidence. The owner's goal permits a necessary remote deployment test, not
periodic synchronization. Prefer CI/manual workflow or prerelease artifact tests
over publication. Inspect current triggers before pushing: main and PR updates
currently trigger CI; v* tags trigger the publishing release workflow. Do not
dispatch and push for duplicate runs. Re-run only for changed inputs, a diagnosed
repair, or a demonstrated transient failure. Capture results and resume locally.

A release tag/published release needs explicit owner release authorization with
a version and intended public outcome. Do not use production publication to test
installation. If current workflows cannot test an artifact without publishing,
prepare a concrete non-publishing validation change or report the limitation.
