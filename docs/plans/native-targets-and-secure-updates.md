# Native presentation targets and secure updates

Owner steering, 2026-10-27. This plan records decisions, open questions and priority
without claiming unimplemented targets are supported. Continue the active backend
prototype in docs/agent-goal.md; future Slint/embedded work does not replace its
Electron, Expo, lifecycle, migration or performance acceptance.

## Context and architectural boundary

The owner wants optional automated desktop-update setup backed by a Tsrust-hosted
service, analogous to configurable static web serving. They also want embedded as
a later target, considering Slint or LVGL, and potentially a desktop choice between
Electron and Slint. All targets must retain usable generated SDKs and secure update
paths. Frontend projects remain ordinary user-owned projects with normal dependency
management. Tsrust owns initial integration and scoped integration migrations.

Keep the checked operation manifest, API compatibility major, codecs and named
operations independent of UI toolkit, SDK language and transport. The existing
TypeScript/HTTP SDK and optional Query Core/React entries are accepted; Rust and
C SDK generation are future work, not delivered capabilities. Native UI objects,
callbacks, OS handles and resource owners never become wire DTOs. Do not revive
the frozen universal renderer or promise shared React/Slint/LVGL components.
Reuse upstream layouts/widgets with renderer-specific examples and templates.

Treat a build target as several explicit choices: execution platform, UI toolkit,
SDK language/transport and packaging/update mechanism. A future desktop engine
setting can select Electron or Slint without changing backend contracts. A Slint
UI can consume a local checked adapter or a remote SDK; LVGL may need a C-facing
SDK/FFI. Decide those mappings from an actual target witness, rather than adding
one speculative universal adapter interface now. HTTP is not mandatory for local
in-process calls or constrained firmware; preserve the same checked semantics.

## Priority and sequencing

| Priority | Work | Entry/exit evidence |
| --- | --- | --- |
| P0 now | Finish current Electron lifecycle, sandboxed named bridge and portable Windows packaging proof | Real SQLite CRUD; readiness/failure/crash/quit; drain/orphan handling; packaged app outside the repository without Rust/Node. No physical computer control. |
| P0 now | Preserve target-neutral SDK/config seams and update-safe lifecycle | No renderer types in manifests; independent SDK generation; ordinary user tooling; stop native backend before updater-controlled restart. |
| P1 during desktop integration | Optional automated update setup and Tsrust feed/artifact serving | Select an update-capable packaging format, generate upstream updater configuration/artifacts, mount a read-only update root without SPA fallback, and test local feed/lifecycle behavior. Keep real signing/public deployment prerequisites explicit. |
| P1 prototype completion | Finish Expo/RNR Android, cancellation, convenience commands, scoped migration/rollback and backend-family validation | Original numbered prototype requirements remain required. Do not lose them to target research. |
| P2 after baseline prototype | Evaluate Slint as an optional desktop engine | Resolve licensing and packaging first; one real desktop task app using a generated Rust SDK or checked local adapter; measured footprint/startup; upgrade proof with the same persisted data. |
| P3 hardware pilot | Embedded Linux with Slint or LVGL | Select one board/display/input/OS and run on hardware; prove SDK behavior, deployment, signed update, interrupted-install recovery and boot health confirmation. A simulator alone is insufficient. |
| P4 separately scoped | MCU/RTOS/bare-metal profile | Choose hardware, RAM/flash/allocator and bootloader first. Prove either a remote-client-only SDK or a bounded local Tsrust runtime subset; do not assume today's std/Tokio/SQL runtime is no_std-compatible. |

Embedded Linux can reuse a hosted Rust environment in principle. MCU support is a
different runtime/toolchain project even if Slint/LVGL already renders there.
Slint's MCU guide requires target HAL/drivers and an allocator. Capabilities and
resource limits must be explicit; unsupported modules should reject at generation/
build time. Keep numeric ranges, optional/null handling, ownership, error mapping,
payload limits and compatibility identical where a profile claims support.

## Update ownership and implementation boundary

Tsrust can automate setup, not replace mature platform installers or bootloaders.
Generate/update the selected toolkit's normal integration files and show a scoped
migration diff. Do not rewrite unrelated manifests, locks, scripts or UI code.

The hosted Tsrust service may expose a configured public/read-only release root
beside its APIs and web mounts. Reuse Router.serveStatic, explicit non-overlapping
prefixes and canonical-root/path protections. Update feeds must never receive a
web app's HTML fallback. Keep mutable feed metadata fresh and artifacts immutable;
publish complete artifacts before atomically advancing feed/channel metadata.
No upload/admin endpoint is implied by a serving option. A bundled desktop backend
does not automatically become a public update server.

Keep app identity, OS/architecture, channel, package version, API major and security
rollback counters distinct. An SDK compatibility failure is not authority to
change feed URLs or download/execute code. Update trust belongs to the configured
release channel and platform verifier. Preserve stable app identity and userData;
package the UI, bundled backend and matching SDK together. Plan database migrations
separately: an old binary may not be compatible with a newly migrated database.

| Target | Candidate maintained update path | Acceptance needed before support claim |
| --- | --- | --- |
| Electron Windows | Forge update-capable installer, initially evaluate Squirrel.Windows plus Electron autoUpdater/update-electron-app; compare MSIX when selecting the final installer | Correct installer events and feed layout; HTTPS; explicit signing configuration; verify what signatures the chosen path actually enforces; version checks; backend drain before apply/restart. A portable ZIP alone is not this installed-update proof. |
| Electron macOS | Electron/Squirrel.Mac with signed/notarized artifacts and generated metadata | Actual macOS signing/install/update witness; no Windows-only proxy for this claim. |
| Slint desktop | Platform-native signed installer/updater selected independently of rendering toolkit | Do not assume an Electron-specific updater works for a Slint executable. Prove update/restart, data retention and verification per OS. |
| Embedded Linux | Evaluate RAUC or an established board/distribution-native updater | Signed bundles, configured trust anchors, board compatibility, redundant A/B or recovery storage, power-loss recovery and boot-health confirmation. Secure boot/rollback enforcement depend on the complete boot chain. |
| MCU/RTOS | Evaluate MCUboot/Zephyr or the silicon vendor's maintained secure OTA path | Verified images at boot, protected provisioned public keys, security counters as supported, image slots/recovery, power-cut and bad-image tests on the selected device. No custom generic firmware flasher in the Tsrust server. |

Production signing keys stay outside repositories, app bundles, SDKs and devices;
use the selected platform's signer/secret provider. Devices contain only required
verification material. Plan key rotation/revocation and compromise recovery with
the selected verifier. TLS and a checksum alone do not establish an independent
publisher trust chain. Never silently bypass verification, platform trust stores
or rollback policy to make a demo pass. Local unsigned/test-key evidence must be
identified as such and cannot stand in for production signed-update acceptance.

The current Electron work has only a ZIP maker configured so far. The updater
request requires revisiting that packaging choice before claiming auto-update
support. Downloaded Electron itself is a toolchain dependency, not evidence that
the Tsrust application can update. No public publishing, release tags, signing
service enrollment or trust-store modifications are authorized by this plan.

## Decision/questions register

These questions are deliberately persisted for the relevant phase. They are not
requests to interrupt the owner's current work. Research or prototype within known
scope first; ask only when a consequential choice actually blocks that phase.

| ID / priority | Question | Needed before / current default |
| --- | --- | --- |
| U1 / P0 | Which Windows installer/update path should the initial managed desktop use: Squirrel or MSIX? | Before updater implementation. Prefer the smallest officially maintained Forge-compatible proof; preserve portable ZIP as a separate artifact. |
| U2 / P1 | Who controls update hosting, HTTPS origin, app identity, signing credentials and release approval? Public feed or authenticated/private distribution? | Before real deployment/signing. Prepare local opt-in configuration; never publish automatically or embed API bearer credentials in feeds. |
| U3 / P1 | Download automatically and apply on next quit, prompt for restart, or use a maintenance window? Are mandatory security updates required? | Before updater UX. Default to no forced interruption of work and drain the local backend before an approved restart. |
| U4 / P1 | Stable/beta channels, staged rollouts, offline updates, rollback retention and minimum supported API/database versions? | Before production channel policy. Start with one explicit channel and immutable versioned artifacts; no fleet dashboard or rollout engine unless needed. |
| S1 / P2 | Is Slint primarily a smaller desktop alternative, an embedded UI, or both? Which OSes and performance budgets matter? | Before desktop Slint pilot. Keep Electron as the active prototype target. |
| S2 / P2 | What Slint license/distribution model fits proprietary/open-source apps and a code-generating toolchain? | Before vendoring Slint or distributing its adapter. Current royalty-free terms distinguish embedded use and exposing toolkit APIs. Do not assume every target has the same terms. |
| E1 / P3 | Which actual board/SoC, Linux distribution or RTOS, display/resolution, touch/input, RAM, flash and GPU are first? | Before renderer/runtime selection. Embedded Linux pilot precedes a general MCU promise. |
| E2 / P3 | Does the device host Tsrust logic locally, act as a client of a remote backend, or both? Which links: HTTP/TLS, serial, BLE, USB or another constrained transport? | Before SDK language/transport prioritization. Keep manifest and SDK generation independent of renderer. |
| E3 / P3 | Slint or LVGL for that witness? Is permissive licensing required, and is LVGL Pro tooling desired? | Before toolkit/toolchain pinning. Compare actual bindings, maintenance, licenses, input/rendering support and measured resource use. LVGL core and commercial tooling are distinct. |
| E4 / P3 | What secure boot, protected key storage, A/B slots, recovery access and anti-rollback facilities exist? Who provisions keys and owns field recovery? | Before promising secure OTA on hardware. Select the maintained boot/update stack from those capabilities. |
| E5 / P4 | Which language/runtime features fit the MCU profile: allocator, async executor, networking/TLS, storage and bounded messages? | Before local no_std support. Remote SDK-only use is a separate valid target, not proof that the complete backend runtime runs on the MCU. |

## Verification gates

Desktop update setup needs feed parsing/no-update/error cases, target/channel
selection, interrupted download, rejected/tampered artifact, unchanged user data,
backend shutdown ordering, failed update/recovery and successful version A to B
installation using the selected upstream mechanism. Platform-required signing
must be proven where credentials/platform access exist, with remaining gaps stated.

Each future target needs a generated SDK consumer outside the Tsrust workspace,
compatible old/new endpoint combinations, strict input/result validation, actual
UI interaction, ownership/resource cleanup, packaging/install proof and a scoped
upgrade/rollback witness preserving owner edits. Embedded gates additionally need
hardware measurements, power interruption, boot fallback/health and invalid-image
rejection; mocks and UI simulators are supporting evidence only.

## Primary research, checked 2026-10-27

- [Electron update guide](https://www.electronjs.org/docs/latest/tutorial/updates)
  documents static-hosted update artifacts and platform-specific feeds.
- [Electron autoUpdater](https://www.electronjs.org/docs/latest/api/auto-updater)
  documents Squirrel/MSIX differences, macOS signing and update/restart behavior.
- [Official update-electron-app](https://github.com/electron/update-electron-app)
  is the initial convenience integration candidate, not a new updater implementation.
- [Slint licensing](https://github.com/slint-ui/slint/blob/master/LICENSE.md)
  and [licensing FAQ](https://github.com/slint-ui/slint/blob/master/FAQ.md) require
  target/distribution-specific review before selection.
- [Slint MCU guide](https://docs.slint.dev/latest/docs/rust/slint/docs/mcu/)
  describes no_std prerequisites, allocator and hardware integration.
- [LVGL licensing](https://lvgl.io/docs/open/introduction/license) distinguishes
  MIT core from commercial tooling; versions/bindings still need admission review.
- [RAUC](https://rauc.io/) documents signed embedded-Linux bundles and redundant
  updates. It is a candidate, not a device update stack already selected here.
- [MCUboot image tooling](https://docs.mcuboot.com/imgtool.html) and
  [Espressif port](https://docs.mcuboot.com/readme-espressif.html) describe key,
  signature and rollback-counter integration. Use current port-specific docs;
  MCUboot explicitly notes its old signed-images page is outdated.
