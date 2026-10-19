# Current component selection and safe batch installation

Owner clarification on 2026-10-19 supersedes the original row-padding behavior:
selection lists have space before and after the complete list, with compact
individual rows. Single, multiple, searchable, target and plain fallback selectors
share this behavior. Visible item counts use one row per option in short terminals.
The plain batch prompt accepts comma-separated choices. Earlier padding evidence
below describes its historical source checkpoint.

Catalog listing, browsing, interactive addition and project scaffolds use one
current version per component and source variant. Browser rows use component
names; details retain version, dependencies and license. Interactive addition uses
Space to toggle several items and Enter to install their combined dependency
closure. Nothing is written until selection and target setup complete. Scripts
can pass several names, optionally pinning the current exact version. Historical
versions are not new installation choices; compatible existing source remains.

First installations keep ordinary filenames. Upgrades preserve old files and
application imports, publishing the new version as `card-v3.tsrx`, for example.
Minor/patch filenames use hyphens so imports remain valid source specifiers.
Repeated installations find the exact lock identity and reuse its actual paths.
Missing recorded files and occupied unrecorded versioned paths fail preflight.

New bundled source resolves relative imports against the complete selected
dependency graph and recorded paths. Only parsed module specifiers change; string
examples, comments, existing components and application imports remain owned.
Immutable payload/template hashes and generated source/path provenance remain
separate. The existing transactional publication and rollback machinery serves
the entire batch. Theme ownership and target selection rules stay explicit.

Focused witnesses cover terminal padding and viewport sizing, keyboard batch
selection, empty selection, cancellation, plain fallback, current-only catalogs,
numeric version ordering, idempotence, old source preservation, dependency import
rewriting, collision preflight and missing-file rejection. Project checks exercise
new scaffolds and versioned component imports. No language/runtime behavior or
component implementation changes in this section; existing UI rendering and
ownership evidence applies without a new GPU/browser or full regression pass.

Focused acceptance passes on 2026-10-19: 37 CLI unit witnesses, seven real
pseudoterminal/redirected-command witnesses and five affected UI project witnesses.
The project batch includes a checked scaffold, its generated native screen test,
read-only import setup diagnostics, explicit web targets and checked versioned
imports after a multi-component upgrade. Terminal witnesses preserve terminal
flags and cursor visibility on success and cancellation, including short and
color-disabled terminals. No SQL suites, complete compiler/LSP matrix or timing
were repeated. Original logs and source identities use `tsrust-ui-terminal-*` in
canonical SQLite.

Accepted source checkpoint: `80355ccd`. Installed CLI/LSP, workspace server override
and both shared-target executables are rebuilt from that commit and independently
hashed. Both live VS Code servers restart with exact settings restoration. The
installed CLI reports six unique current catalog entries; a dry run in the user's
standalone counter proposes `button-v2.tsrx` and `card-v3.tsrx` without changing
source, configuration or lock bytes. The installed counter check and formatting
remain clean. Cleanup removes 76 obsolete test/native outputs (2649.7 MiB), verifies
all five consumers remain unchanged and finds no owned build/test/browser process.


The clarified list spacing passes four focused prompt/fallback checks and all seven
terminal witnesses on 2026-10-19. Rich prompts add padding only around active/error
lists; submitted/cancelled summaries and text inputs keep their default layout.
Compact item rows restore the short-terminal visible count without changing
selection, search, validation, cancellation or terminal restoration behavior.
