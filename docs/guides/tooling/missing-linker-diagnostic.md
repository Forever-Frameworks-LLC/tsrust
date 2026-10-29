# Missing native linker diagnostic: retained acceptance

This record preserves the original diagnostic-repair evidence. Current user
instructions are in [native build setup](../../../website/src/content/docs/tools/rust-setup.md#repair-a-missing-native-linker).
The historical paths and case counts below describe that acceptance checkpoint,
not the current suite or available build outputs.

## Evidence

- `target/editor/missing-linker-red.log`: the real pinned backend with an empty PATH originally produced the misleading compiler-defect message.
- `target/editor/missing-linker-acceptance.log`: all 23 acceptance tests pass, including the real missing-linker witness, missing-backend setup, debug/release execution and pinned backend selection.
- `target/editor/missing-linker-generated-rejection.log`: the additional generated-code rejection test passes and retains E0308 details without the missing-linker message. Together the retained runs cover all 24 current acceptance tests.
- `target/editor/missing-linker-build-debug.log` and `missing-linker-build-release.log`: refreshed compiler/editor consumers.

The exact baseline is `archive/missing-linker-diagnostic-start`. This is a CLI diagnostic repair, not a new runtime operation or numbered feature gate. No new workload or timing is needed to compare a failed toolchain launch; existing benchmark sources and SQLite measurements remain unchanged. Gate 127 remains the next feature gate.
