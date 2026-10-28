# Bounded editor language-server recovery

Date: 2026-10-26. Source checkpoint before changes: `be1dec4e`.

## Report and evidence

The owner regularly sees "Connection to server got closed. Server will not be
restarted." Normal VS Code logs for both project windows report an exit at
17:12:19 with code `4294967295` (Windows unsigned representation of `-1`). Neither
Tsrust output contains a Rust panic or stack-overflow report. Simultaneous exits
suggest a shared external cause, but the logs do not prove that cause.

The extension had a concrete recovery defect: its custom error handler returned
`DoNotRestart` after every unexpected exit and continued through all protocol
errors. The screenshot therefore did not establish that a restart budget had been
exhausted. No runtime/compiler crash fix is claimed here.

## Change and validation

Remove the custom handler and use `vscode-languageclient` 9.0.1's existing bounded
error/restart handling, explicitly allowing four attempts. Its standard policy
stops after a fifth close within three minutes. A manual restart creates a new
client and resets the budget. No custom supervisor/retry loop is added; normal
stop/disposal semantics remain with the language client.

The focused VS Code test runs in a private workspace/profile. It locates only
`tsrust-lsp` children of its own extension host and terminates those processes.
For each automatic recovery it edits valid then invalid current text and verifies
diagnostics change, rather than trusting stale diagnostics or process existence.
It verifies four automatic restarts, the bounded fifth stop, manual restart and
current-text diagnostics. It never terminates a user's normal server.

- `npm run compile` and `tsc -p tsconfig.test.json` pass.
- Focused development-host recovery integration passes with installed Windows
  VS Code and the existing verified `target/debug/tsrust-lsp.exe`.
- `npm run package` passes. The exact VSIX passes the same focused test after
  installation into a private test profile.
- The verified VSIX is installed into the owner's normal profile. Initial Computer
  Use app approval timed out; the owner then explicitly requested another attempt.
  The project window's extension host was refreshed without saving/editing source,
  and its status returned to `Tsrust: checking enabled` with no Problems reported.
  The setup window's host was also refreshed and returned to checking enabled
  with no Problems. The tested crash/recovery sequence remains confined to isolated
  test profiles.

The installed LSP executable was not replaced by this extension-only fix. No
compiler/editor matrix, benchmark, CI, registry publication or public release ran.
Current logs under `target/lsp-recovery-*.log` are disposable diagnostics, not
analytics inputs. The historical long-lived memory report and actual cause of the
two observed exits remain unresolved.


## macOS documentation-audit acceptance — 2026-10-28

The current recovery integration passes in a temporary workspace/profile using the
installed macOS VS Code executable and the current `target/debug/tsrust-lsp`.
Four automatic restarts, stopped fifth recovery, explicit manual restart and
current-text diagnostics all pass; the runner exits 0. Its process selection is
restricted to direct LSP children of its own extension host. No owner workspace,
settings or normal editor processes were controlled. Expected stream-close errors
occur while deliberately terminating the test servers; they are not hidden.
Disposable log: `/tmp/tsrust-macos-editor-recovery.log`.

The full isolated editor integration also passes, including actual check/run tasks,
formatting and format-on-save, compiler-backed quick fixes, unsaved imports,
hover/signatures/completions, definitions/references, outline, missing-server
recovery and current-text diagnostics. Review found mojibake in the Unicode fixture;
it now creates a real U+1F600 surrogate pair and U+00E9 from explicit JavaScript
escapes, with an assertion that the fixture has more UTF-16 units than code points.
The diagnostic range still selects exactly `true` after those characters. Both
module and standard-library native task checks pass. Disposable log:
`/tmp/tsrust-macos-editor-integration.log`; runner exits 0.

These are short interactive acceptance tests, not long-session memory measurements.
The repaired surrogate-pair fixture has not yet been re-executed on Windows/Linux.
