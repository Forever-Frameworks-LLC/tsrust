# Tsrust documentation

Start with [installation](../website/src/content/docs/start/installation.md), then create a
project and work through an [example](../examples/README.md). Tsrust is experimental;
these guides distinguish supported behavior from planned work.

## Build an application

- [Install the CLI and editor](../website/src/content/docs/start/installation.md)
- [Managed CLI/LSP installation and updates](../website/src/content/docs/tools/install-and-update.md)
- [Create a backend HTTP application](../website/src/content/docs/learn/first-service.md)
- [Understand project structure](../website/src/content/docs/start/project-structure.md)
- [Use the CLI](../website/src/content/docs/tools/cli.md)
- [Inspect and prepare native build tools](../website/src/content/docs/tools/rust-setup.md)
- [Explore the standard library](../website/src/content/docs/library/overview.md)
- [Understand string semantics](../website/src/content/docs/language/strings.md)
- [Use collections](https://forever-frameworks-llc.github.io/tsrust/reference/library/collections/)
- [Work with Rust](../website/src/content/docs/tools/rust-packages.md)
- [Set up VS Code](../editors/vscode/README.md)

## UI applications

The earlier Tsrust UI renderer and frontend templates were removed. Native UI
returns as GPUI and GPUI Kit components written in `.tsrx`; see
[the GPUI application platform plan](plans/gpui-application-platform.md).

## Contribute

[Contributing](../CONTRIBUTING.md) covers setup and targeted checks.
[Development workflow](development-workflow.md) defines acceptance, formatting,
checkpoints and evidence retention. [Release automation](guides/releases.md)
describes CI, artifacts and publication.

| Directory | Contents |
| --- | --- |
| [guides/language/](guides/language/) | Language semantics, library behavior and coverage |
| [guides/tooling/](guides/tooling/) | Editor, packages, deployment and toolchain guides |
| [guides/](guides/) | Additional practical library and tooling guides |
| [plans/](plans/) | Design proposals and implementation plans |
| [milestones/](milestones/) | Feature acceptance and evidence summaries |
| [measurements/](measurements/) | Measurement methodology and analysis |
| [Compiler fixtures](../crates/tsrust-compiler/tests/fixtures/) | Executable examples and rejection witnesses |
| [Runtime benchmarks](../benchmarks/runtime/) | Matched inputs, independent oracles and drivers |
| [Analytics storage](../benchmarks/storage/) | Canonical SQLite evidence import and restoration |

## Design and progress

- [Language vision](plans/product-vision.md) and [roadmap](roadmap.md)
- [Current standard-library roadmap](plans/standard-library-roadmap.md)
- [Feature coverage](guides/language/feature-coverage.md)
- [GPUI application platform plan](plans/gpui-application-platform.md)
- [Compiler source layout](guides/compiler-source-layout.md)
- [Rust toolchain policy](guides/tooling/rust-toolchain-policy.md)

Current behavior guides and accepted evidence describe implemented behavior.
Plans do not silently settle open decisions. Historical “next” statements can be
superseded by [the handoff](agent-progress.md), the single continuation note.
Benchmark measurements live in `benchmarks/measurements.sqlite`;
Markdown reports are optional exports. Full regressions and timing belong at
meaningful milestones, not after each small edit.
