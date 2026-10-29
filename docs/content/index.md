<!-- tsrust-docs
{"id":"introduction","group":"getting-started","order":0,"status":"partial","targets":["web","native"],"examples":["counter"]}
-->
# Retained UI documentation

This retained application demonstrates the original browser/native UI and checked
examples. Current backend documentation is maintained separately in the public
Starlight site. UI expansion is outside the active backend product scope.

## Start with the application

[Install the tools](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/website/src/content/docs/start/installation.md), then read [project structure](https://github.com/Forever-Frameworks-LLC/tsrust/blob/main/website/src/content/docs/start/project-structure.md). The project configuration defines its entrypoint, targets and theme explicitly.

## Learn through working examples

The counter example uses local state, real application callbacks and project-owned Button/Input/Label source. Its displayed code comes from the same file as the preview.

[State and ownership](ui/foundations/state.md) explains retained state and cleanup. [Theming](ui/foundations/theming.md) explains light/dark tokens and current customization limits.

## Components

Start with [Button](ui/components/button.md), [Card](ui/components/card.md) and [Input](ui/components/input.md). Each reference lists supported props and includes its local source.

## Language reference

The [strings reference](standard-library/strings.md) covers `trim`, including arguments, results and Unicode behavior.
