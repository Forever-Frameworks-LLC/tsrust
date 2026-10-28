# Native documentation development profile

Accepted on Windows x86_64, 2026-10-27. This repairs retained documentation startup;
UI expansion remains frozen and the active product remains the backend framework.

## Cause and change

The cached docs application c4f1c2e842e44471 reproduced exit 0xc00000fd in 2.716s.
Its PE stack reserve is 1,048,576 bytes. Application-only Rust assembly against the
same existing runtime identifies a 609,792-byte Page frame and a reachable nested
initial-route call path reserving 1,569,904 bytes. Large unoptimized temporaries
exceed the default stack; increasing the linker stack reserve was not the repair.

Native UI support already uses development opt-level=1, with dependencies at 2.
Both normal CLI native application compilation and the dev supervisor now select
application opt-level=1 when the checked features contain ui-native. Backend-only
and non-rendering development remain at 0; release remains at 3. Overflow checks
remain enabled. Both executable cache identities include the selected optimization,
so the old crashing artifact cannot satisfy the corrected build. Runtime cache
identity is unchanged because its compilation profile did not change.

The controlled optimized executable keeps the same stack reserve. Its largest
simple decoded unwind frame is 100,344 bytes, versus 609,800 bytes in the baseline
(including saved nonvolatile registers). This is a diagnostic comparison, not a
matched performance benchmark or proof of arbitrary component-depth stack bounds.

## Acceptance

- Normal CLI unit runner: profile selection regression 1/1 passed.
- Backend compiler/LSP all-target check and workspace all-target check: both
  successful terminal build-finished, zero warnings and errors.
- Whole-project formatting before checkpoint: zero files/fingerprints changed.
- Rebuilt CLI formatted the backend std:testing array smoke source, passed format
  --check and executed its assertions successfully. Initial native compilation in
  the sandbox failed with archive rename access denied; outside-sandbox execution
  passed. The first format invocation printed source; --write then --check repaired
  the verification procedure before executing formatted source.
- Controlled opt-level=1 docs executable rendered the real native window. Getting
  started navigation changed visible content successfully. Preview remained visible
  for interaction, then Alt+F4 closed the owned process with exit 0.
- Rebuilt CLI, from docs/: tsrust dev --target native --verbose. Checked 1,668
  modules, prepared native runtime offline, compiled and launched PID 36060 in
  217,391ms. Application cache c332a85856722893 has debug plus explicit opt-level=1.
  The actual Tsrust documentation window was observed through the window inventory.
  Capturing that temporary executable's window required app approval, which timed
  out; no capture/input workaround was attempted. The earlier controlled preview
  provided the rendered-content/navigation evidence. Ctrl+C stopped the owned dev
  session; subsequent window inventory confirmed no documentation previews remain.

Diagnostic experiments first lacked serde externs, then the native Windows import
search path. Supplying exact existing dependencies repaired those experiment
commands; these were not generated CLI link failures. An exhaustive assembly path
walk was cancelled and replaced with bounded discovery. An initial ten-second
subprocess probe ended with its parent job; it was not classified as a crash.

Current gate logs: target/docs-profile-{unit,backend-check,workspace-check,cli-build}-current.log;
JSON diagnostics: target/docs-profile-{backend,workspace}-diagnostics-current.jsonl.
Concise frame/cache evidence: target/docs-stack-profile-comparison-current.json and
 target/docs-profile-dev-cache-current.json. Disposable assembly/emitted-source
experiments are not historical analytics and may be pruned. No full executable
regression/performance matrix was restarted, and nothing was imported into SQLite.

## Remaining scope

Keep the original UI source intact. The backend goal still requires server-owned
API registration without a public generated/server prerequisite, remaining HTTP
SDK/body/testing work, binding cache migration and trusted installation/update
acceptance across the required platforms. This startup milestone does not prove
those unrelated requirements or the owner's entire live VS Code Problems panel.

## Installed consumer and retention

Source checkpoint: cc8a0024, all changes included. Validated development CLI installed
with expected-old/staged/new SHA256 guards and a rollback copy. Installed CLI hash:
90D6BB34FD06398096F0A61ADA139AC3B91C6781C3198712A79E15F8543829BF.
Installed --version and check of the original stdlib/ui/api.tsr passed. LSP was not
changed by this CLI-only repair; retain its previously validated installed hash
53A93B85E008A4C0795709ECEEF311C34723D33E4D9894BCED265A2343DD8F15.
This guarded development refresh is not the pending production update mechanism.

Pruned 346,247,105 bytes of disposable emitted source/assembly/helper/experimental
binary/log outputs. Removed the superseded crashing docs executable cache (110,564,134
bytes) after checking resolved paths, reparse ancestry/members and live executable
owners. Removed the old CLI rollback copy after the installed check and live-owner
check. Preserved current executable cache c332a85856722893, native support caches,
source pack, concise evidence, generated review apps and mapped owner LSP images.
No archive directories or analytics imports were created.
