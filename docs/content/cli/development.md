<!-- tsrust-docs
{"id":"reference-cli-development","group":"cli","order":20,"status":"partial","targets":["web","native"]}
-->
# Development and troubleshooting

`tsrust dev` watches the project, reuses compiler/dependency work and replaces the preview after a successful build. A failed rebuild keeps the previous application available and reports the error.

Use `--target web --port 4317` for a browser preview. Choose native explicitly in a project with multiple enabled targets when that is the surface you want to inspect. Web development needs the pinned Wasm tooling described in [installation](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/docs/guides/tooling/retained-ui-setup.md).

Build time, asset transfer and main-thread responsiveness are separate costs. The docs use lazy article construction and compressed dev assets. HTML-first rendering and executable splitting are not implemented yet.

Troubleshooting needs expanded coverage for toolchain, graphics, imports, cache permissions and editor diagnostics.
