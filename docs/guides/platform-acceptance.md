# Native platform acceptance without publication

This is the focused I/O, async, process, networking and compression family check,
not the complete v0.2.0 checklist. It does not create a tag, publish an artifact,
merge a branch or establish throughput. Run it only for an exact locally accepted
checkpoint after the development skill's remote-validation decision and a current
PR/Dependabot check.

Manually dispatch the existing CI workflow with only `platform_acceptance=true`.
The job uses the release host matrix: Linux x86_64, macOS ARM64, macOS x86_64 and
Windows x86_64. Other focused modes and full regressions are mutually exclusive
with this mode. The final CI gate requires all four platform jobs to succeed.

The runner asserts the native Rust host and pinned Node version. Setup fetches
locked dependencies explicitly; subsequent compilation and tests run offline and
locked. One shared Cargo target, one build job and disabled incremental compilation
apply throughout. Each command has a 30-minute limit and an approximately 5 GB
process-tree budget. Counter failures stop the run. Windows uses owned process
tree termination; POSIX uses an owned process group plus tracked descendants.
The watchdog itself needs real validation on each host before its bounds are
described as established platform evidence.

Dependency metadata checks archive identities, notices and declared Rust version
requirements. A missing `rust-version` is recorded as undeclared; it is not proof
of compatibility with the pinned compiler. Successful native compilation and
execution remain separate requirements. Audit findings and retained optional UI
warnings must be reported with their active feature scope, without ignore flags.

Each target is built separately. The runner requires terminal Cargo JSON success,
finds that target's exact test executable, and records executable and CLI/LSP
hashes. It then runs exact cases serially with `--test-threads=3`. Only passes
with matching source, executable and consumer identities can be reused. Ordinary
assertion failures leave independent targets runnable. A resource or watchdog
failure stops continuation. Successful integration executables are removed to
avoid filling the hosted runner; dependency caches remain shared.

Selected compiler and LSP targets cover bytes, files/path/environment, owned I/O,
tasks/channels/threads, subprocesses, DNS, TCP/UDP and compression. The HTTP target
includes a real private-CA verified HTTPS example as well as invalid TLS options.
Generated signal listeners must acknowledge both platform signals and exit
normally. Windows delivery uses a disposable sender attached only to a dedicated
recipient console; it refuses any console containing another process.

The 60,000-byte UDP case runs explicitly on Linux and verifies complete binary
delivery and the sender address. EMSGSIZE is a failure of that supporting-host
witness, never a pass. On other hosts this one case is explicitly excluded; their
ordinary UDP tests still run. This is correctness coverage, not measurement of the
registered `net_udp_large` throughput workload.

Two other exclusions are explicit in results: the opt-in SQL/Docker combination
test belongs to separate acceptance, and the internal Windows console sender is
an ignored helper invoked by the actual generated-listener test. Neither is
counted as a passing independent scope. Unexpected ignored tests fail acceptance.

After all selected cases pass, the runner packages the built debug CLI/LSP as a
temporary development toolchain and exercises them outside the repository with
the existing toolchain smoke test. This proves packaged-consumer behavior, not
optimized release archive notices, signing, installation/update/rollback or the
final v0.2.0 release gate. Those remain separate requirements.

Results and disposable per-command outputs are under
`target/platform-acceptance/<host>-<commit>/`. The CI job retains them briefly for
review. No logs or source snapshots are imported into the measurements database.
Record accepted scopes and remaining work in `docs/agent-progress.md`; do not
create another handoff file.
