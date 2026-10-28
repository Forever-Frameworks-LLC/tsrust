# Tsrust documentation

Start with [installation](../website/src/content/docs/start/installation.md), then create a
project and work through an [example](../examples/README.md). Tsrust is experimental;
these guides distinguish supported behavior from planned work.

## Build an application

- [Install the CLI and editor](../website/src/content/docs/start/installation.md)
- [Managed CLI/LSP installation and updates](../website/src/content/docs/tools/install-and-update.md)
- [Create a backend HTTP application](../website/src/content/docs/backend/applications.md)
- [Understand project structure](../website/src/content/docs/start/project-structure.md)
- [Use the CLI](guides/project-cli.md)
- [Inspect and prepare native build tools](../website/src/content/docs/tools/rust-setup.md)
- [Explore the standard library](guides/language/standard-library.md)
- [Understand string semantics](guides/language/string-semantics.md)
- [Use collections](guides/collections.md)
- [Work with Rust](guides/rust-packages.md)
- [Set up VS Code](../editors/vscode/README.md)

## Run the documentation app

This directory retains the original standalone UI project, with a shared web/native page.
It is outside the active backend product scope.
`tsrust.config.json` declares the targets and theme. `src/main.tsr` mounts
`src/app/page.tsrx`; editable components live under `src/components/ui/`.

```sh
cd docs
tsrust dev --target web --port 4317
# On a supported graphical desktop:
tsrust dev --target native
```

The landing page is at `/`; documentation starts at `/docs`. Install the
[retained UI prerequisites](guides/tooling/retained-ui-setup.md) first. The current
native custom titlebar requires macOS. The app and component library remain
in development; a page destination does not imply a finished implementation.

Retained UI references: [themes](content/ui/foundations/theming.md) and
[components](content/ui/components/).

## Contribute

[Contributing](../CONTRIBUTING.md) covers setup and targeted checks.
[Development workflow](development-workflow.md) defines acceptance, formatting,
checkpoints and evidence retention. [Release automation](guides/releases.md)
describes CI, artifacts and publication.

| Directory | Contents |
| --- | --- |
| [content/](content/) | Markdown displayed by the documentation app |
| [guides/language/](guides/language/) | Language semantics, library behavior and coverage |
| [guides/tooling/](guides/tooling/) | Editor, packages, deployment and toolchain guides |
| [guides/](guides/) | Additional practical library and tooling guides |
| [src/](src/) | Shared documentation UI and executable examples |
| [plans/](plans/) | Design proposals and implementation plans |
| [milestones/](milestones/) | Feature acceptance and evidence summaries |
| [handoffs/](handoffs/) | Current and retained historical continuation notes |
| [measurements/](measurements/) | Measurement methodology and analysis |
| [Compiler fixtures](../crates/tsrust-compiler/tests/fixtures/) | Executable examples and rejection witnesses |
| [Runtime benchmarks](../benchmarks/runtime/) | Matched inputs, independent oracles and drivers |
| [Analytics storage](../benchmarks/storage/) | Canonical SQLite evidence import and restoration |

## Design and progress

- [Language vision](plans/product-vision.md) and [roadmap](roadmap.md)
- [Current standard-library roadmap](plans/standard-library-roadmap.md)
- [Feature coverage](guides/language/feature-coverage.md)
- [UI vision](../UI_VISION.md)
- [Documentation and component-library plan](plans/documentation-and-component-library.md)
- [Compiler source layout](guides/compiler-source-layout.md)
- [Rust toolchain policy](guides/tooling/rust-toolchain-policy.md)

Current behavior guides and accepted evidence describe implemented behavior.
Plans do not silently settle open decisions. Historical “next” statements can be
superseded by the current handoff. Original benchmark records live in SQLite;
Markdown reports are optional exports. Full regressions and timing belong at
meaningful milestones, not after each small edit.
