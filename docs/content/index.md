<!-- tsrust-docs
{"id":"introduction","group":"getting-started","order":0,"status":"partial","targets":["web","native"],"examples":["counter"]}
-->
# Build with Tsrust

Write checked TypeScript-style application code, compose editable UI components and use Rust-backed services. This documentation shares its application source between browser and native targets.

## Start with the application

[Install the tools](getting-started/installation.md), then read [project structure](getting-started/project-structure.md). The project configuration defines its entrypoint, targets and theme explicitly.

## Learn through working examples

The counter example uses local state, real application callbacks and project-owned Button/Input/Label source. Its displayed code comes from the same file as the preview.

[State and ownership](ui/foundations/state.md) explains retained state and cleanup. [Theming](ui/foundations/theming.md) explains light/dark tokens and current customization limits.

## Library status

The pinned library inventory contains 64 components, three recipes and 30 blocks. Most remain pending. A navigation destination does not imply an implemented control or a working preview.

Existing [Button](ui/components/button.md), [Card](ui/components/card.md) and [Input](ui/components/input.md) pages describe their actual supported subsets. Complete Nova parity is ongoing work.

## Language reference

The [strings reference](standard-library/strings.md) begins with `trim`, including arguments, output, Unicode behavior and measurement status. Benchmark data will come from curated canonical records; missing measurements never become invented comparisons.
