# Reactive props

Run `tsrust dev --target web` or `tsrust dev --target native` from this project.
The same Page uses exact derived values, a typed live custom prop, dynamic styles
a locally editable Alert with live variant/title classes, and a replacement Button that forwards its owned children and handler once.

`tsrust run src/inspect.tsr` checks exact integer/nullable values and initial
composition. Browser transitions have an independent TypeScript oracle in
`benchmarks/ui-jsx/reactive-props-reference.ts`. Native model tests independently
observe node identity, activation, invalid updates and evaluator release.

Component factories run once. `useProp` reads incoming live values; `computed`
defines derived values. Returning the tree retains state without exposing handles.
