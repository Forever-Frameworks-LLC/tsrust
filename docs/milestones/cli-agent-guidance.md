# Launcher and generated agent guidance

Owner request, 2026-10-26: restore the familiar interactive CLI on Windows and
make project/component creation supply usable agent instructions. Based on local
source checkpoint a1a6cc4e; no release or hosted workflow is involved.

## Accepted behavior

The screenshot is the existing explicit TERM=dumb fallback. The automation shell
sets TERM=dumb while user/machine settings are empty, consistent with the VS Code
window inheriting it from launch. The Windows setup example now clears TERM in
new Windows terminals. Existing shells keep their previous environment, so open a
new terminal. Intentional dumb terminals still get numbered prompts; redirected
input still prints help. Rich terminals print the branded header when wide enough,
including short panels, and retain adaptive menu rows. Windows input mode is held
raw across the complete prompt and restored exactly on normal cancellation/error
returns; cursor visibility is restored too.

`tsrust new` creates root AGENTS.md for console and UI projects. It covers actual
commands, configured entrypoints, ownership, verification and native/web setup.
`tsrust ui add` creates missing root guidance, a concise AGENTS.md component index,
and versioned references such as button-3.0.0-default.AGENTS.md. Each reference
records the installed source/import locations, checked exports, required/optional
props, available behavioral documentation and dependencies. They reuse the checked
documentation API and existing virtual-source provider, including staged .tsrx
imports; no second parser or copied component bodies is introduced.

References describe the original installed API. Customized local source and owner
instructions remain authoritative. Existing root guides and edited references are
preserved. New index links append without rewriting owner bytes. Re-adding the same
components preserves guides, source and lock bytes. New versions get separate
references. Dry runs publish nothing. Guidance joins the existing installation
transaction with stale-file checks, staging, rollback and lock-last publication.
Metadata checking uses effective configuration and theme inputs before publication.
Windows parent paths occupied by files now fail during planning, not later writes.

## Verification

- All 17 installer cases pass across retained batches. Initial 15 passed; two setup
  failures were diagnosed and repaired. Five related setup cases pass afterward.
- The complete current-catalog guidance witness verifies checked API/props and
  byte-preserved owner instructions/references. The versioned dependency-import
  witness also passes after the staged-configuration repair.
- Four focused launcher helper tests pass.
- `py -3 -B tools/smoke-windows-cli.py CLI` first reproduces the old 14-row missing
  branding, then passes for numbered fallback, branding, arrow/Enter/Esc, Ctrl+C,
  exact input-mode restoration and visible cursor. It owns an isolated hidden
  console, checks its client process list before clearing it, and closes children.
- `node tools/smoke-toolchain.mjs CLI` verifies new root guidance plus scaffold,
  check, formatting, native run, project tests and bundled JSON.
- `node tools/smoke-cli-agents.mjs CLI` verifies real UI scaffolding, inherited prop
  references, preserved owner guide/source/lock bytes, no-op re-adds, no-write dry
  runs, custom component paths, formatting and complete project checking.
- The normal CLI was refreshed after final entrypoint wording, installed by exact
  byte comparison, and verified to generate the reviewed template. SHA-256:
  616c928d888970c69a6752ae69d2afc6a814224dae10842662d40c87eb6e63c4.
- Whole-project formatting and installed format-check pass with zero source or
  benchmark-fingerprint changes. Existing owner project examples/windows-setup/wut
  stays unrelated and unstaged.

Current logs are concise gate evidence under target/windows-cli-*.log; they are
not analytics records. No benchmarks were timed or imported. The separate generated
native UI application linker failure for windows.0.52.0.lib remains open in the
Windows installation evidence; it is not hidden by these launcher/guide passes.

Accepted local checkpoint: 48e96ff9. The installed CLI also generated actual guides
in the ongoing target/windows-ui-setup fixture, without changing its components.
The redirected integration witness passes separately. Post-checkpoint cleanup
removed four stopped CLI test executables/symbols (1,036,318,208 logical bytes;
1,036,324,864 observed free bytes), preserving all installed/warm native consumers.
