# Paired toolchain installation and updates

Implementation contract, 2026-10-28. Implemented with focused macOS and Windows
acceptance in [the milestone](../milestones/paired-toolchain-installation.md).
Linux and published-release acquisition remain pending. The live-LSP update
witness passes on macOS and Windows; Linux execution remains pending. This addresses
outcome 4 of backend-only-reset.md; it does not introduce a package manager or
background update service.

## Installation and ownership

Use `tsrust toolchain install --root <new-directory>` for an explicit prebuilt
installation. This avoids colliding with `tsrust update <crate>`. The user adds
that installation's bin directory to PATH; do not edit shell profiles, global
Rust settings or package-manager directories. Check/update/rollback operate only
on a validated managed installation. Unmanaged executables receive upgrade or
reinstallation guidance; never infer write permission from their location alone.

Store each verified compiler/LSP pair and notices in a versioned directory. A
small launcher dispatch path in the existing CLI reads one versioned installation
record; the LSP launcher selects the server from the same record. Both launchers
must reject unsupported record versions, traversal and modified metadata. Keep
launcher protocol version 1 explicit. A release requiring a newer protocol must
fail with reinstallation guidance rather than reinterpret old fields.

Updates stage a complete pair before atomically replacing that one record. Never
replace a running CLI or LSP file. Running processes finish on their original
version; newly started processes select the committed pair. Preserve the prior
verified pair for explicit rollback. Concurrent mutations share one installation
lock. A failed or interrupted stage leaves the current record untouched; cleanup
removes only installer-owned incomplete directories. Windows locked old binaries
must not prevent switching to a fully staged new pair, and cleanup may retain a
locked previous directory until it can safely be removed.

## Release trust and bounded acquisition

Fetch latest stable metadata over verified HTTPS from the fixed official GitHub
repository API. Require a published, immutable, non-prerelease release, valid
semantic version, supported host target, exactly one matching asset, expected
asset URL, bounded size and a SHA-256 digest supplied by GitHub's API. Do not
accept arbitrary release origins, disable TLS validation, or trust a checksum
file next to an untrusted archive. Refuse automatic downgrades. Explicit rollback
selects the retained verified prior pair and performs no network access.

GitHub documents immutable release state and asset digests in its
[release API](https://docs.github.com/en/rest/releases/releases). Trust is in HTTPS,
the official repository and GitHub's release metadata, not independent signing
or notarization. Keep those limitations explicit. Check and update are explicit
commands; ordinary compile/dev/LSP startup never contacts GitHub.

Download asynchronously with connection/total timeouts and strict compressed and
expanded byte/file limits. Follow only required HTTPS GitHub asset redirects;
never forward credentials to a redirect. Verify the archive digest before
extracting. Use a maintained archive reader, reject links, special files,
traversal, duplicate/case-colliding names and unexpected top-level paths. Verify
manifest schema, clean release identity, host, separate compiler/LSP versions,
file hashes/sizes and required notices before selecting or executing a pair.
Do not install Rust or run package scripts during installation.

## Acceptance

Implement CLI help, ownership diagnostics and an installation guide with the
commands. Exercise pure release/manifest rejection controls and real isolated
installation/dispatch. Verify no-update, failed download, corrupt archive,
wrong host, untrusted origin, absent digest, interrupted stage, concurrent update,
rollback, both executable versions, and preservation of owner files. Hold a real
LSP open while updating and prove new launches select the new pair while the old
process remains usable. Execute that witness on Windows as well as macOS/Linux;
fixtures alone cannot establish OS replacement semantics. Reuse the existing
non-publishing platform validation and package smoke once implementation is ready.
No public release is created merely to test the updater.


## Documentation smoke integration

`tools/smoke-toolchain.mjs`, already used by all four package jobs, now verifies
explicit offline dependency setup and native doctor, then reads the actual
quickstart/testing examples from canonical site Markdown. It formats and executes
them, runs the documented POSIX shell or Windows PowerShell reporter command, and
checks the documented failing greeting reports both expected and received values.
All work stays in an isolated temporary project/cache and is removed afterward.
The completed smoke passes on macOS with the current debug consumers; this is
local behavior evidence, not a release-package or fresh-machine claim. Windows and
Linux execution of the new documentation branch remain pending. No new workflow
or duplicate compiler build was introduced; the existing non-publishing package
validation can exercise it when the coherent platform section is ready.
