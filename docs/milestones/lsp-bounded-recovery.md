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
