# Project-owned UI

This is the theme/component workflow produced by `tsrust new counter --ui`.
`app/page.tsrx` owns the state and event logic. `components/ui` contains ordinary,
editable source; primitives and ownership checks stay in `std:ui`.

Run `tsrust run web.tsr` for the assertion-based HTML example, or `tsrust dev`
for the GPU window. The bootstrap explicitly selects dark mode. The JSON file is
compiled into the application, independent of the reusable renderer dependency
build. Changing theme data reuses that build.

Use `tsrust ui list --bundled`, then `tsrust ui add button@1.0.0 --dry-run` in a
newly scaffolded project. This repository fixture is hand-maintained source, not
an installed lockfile; the installer refuses to overwrite it. New projects have
an installation lock with exact version, source, license and toolchain provenance.
