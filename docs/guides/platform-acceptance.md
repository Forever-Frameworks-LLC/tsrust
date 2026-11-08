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

For a Windows repair, also set `platform_windows_only=true` to select only that
host. Its success proves the selected Windows scope; retain the other hosts'
separate evidence. Do not dispatch it on a ref with a live CI run: the workflow's
ref concurrency policy would cancel that run.

The finite selected scope includes 45 compiler integration targets and 16 LSP
integration targets. All seven file targets are explicit: `std_fs`,
`native_file_cleanup`, `native_full_files`, `native_file_writes`,
`native_filesystem_utilities`, `native_buffered_files` and
`native_buffered_file_backend`. They cover public close/ownership, lexical cleanup,
ordinary large-file defaults and explicit limits, writes/views, filesystem utilities
and buffered close/partial I/O. The buffered backend target compiles the actual
`fs.rs` runtime test module at O0/O3; on Windows that includes
`windows_close_and_scope_drop_release_exclusive_open_conflicts`. Its success must
come from the Windows native run, not a source inspection or non-Windows pass.
The generated listener test in `service_sections` separately exercises real
Windows SIGINT/SIGBREAK delivery through its owned console helper.

One explicit selected-test Cargo invocation builds all 61 intended test targets and
the normal CLI/LSP consumers together. Cargo also emits three known same-name
cross-package targets; artifact validation checks all 64 but executes only the 61
intended targets. Existing package/source/profile/uniqueness guards remain strict.
No whole library harness is added. This is focused native standard-library acceptance, not final
backend regressions or complete v0.2 acceptance. The expanded scopes' elapsed
cost still requires measurement within the unchanged 55-minute aggregate budget;
exhaustion preserves completed cases and remains incomplete acceptance.

The additional used-root targets are `url_values`, `url_features`, `regex_text`,
`explicit_locale`, `crypto_primitives`, `structured_logging`, `realtime`,
`owned_iterators`, `stdlib_completion`, `rust_adapter_generation` and `rust_bridge`.
Focused LSP targets `url`, `crypto`, `structured_logging` and `rust_bridge` add
corresponding editor contracts. The one Cargo invocation explicitly selects the
compiler's existing `http-websocket` and `logging` features; SSE-only/ordinary HTTP
activation controls remain in the tests. There are no newly ignored tests or
benchmark runs in these targets.

The existing shared55 runner supplies5 logging,18 realtime and32 iterator cases
at both profiles without a duplicate native runner. Crypto retains its24-case
paired suite; regex's18 catalog examples already share one program per profile.
The selected Rust adapter target retains actual Shlex generation and relocated
empty-Cargo-home builds, alongside nominal buffer/pointer controls. Existing
setup fetches all three locked production graphs and the binding-test graph;
Shlex2.0.1 is already in the production locks. No npm install or Node comparison
packages are added. This does not add the broad compiler/LSP libraries, all
integration targets, deep B18 runner regressions or the full editor matrix.
The remaining locale, URL, collection smoke and adapter/export programs retain
individual native application builds with the shared native cache. Total expanded
four-host cost is unmeasured: a budget stop is incomplete acceptance, not permission
to drop cases or resume the same oversized suite through serial hour-long chunks.

The runner asserts the native Rust host and pinned Node version. Setup fetches
locked dependencies explicitly; subsequent compilation and tests run offline and
locked. One shared Cargo target, one build job and disabled incremental compilation
apply throughout. The runner also sets `TSRUST_CACHE_DIR` to
`target/tsrust-native` for every child, including standalone `tsrust run` boundary
fixtures. A shared Cargo target alone does not prevent standalone commands from
creating temporary native support caches. Other CI acceptance modes use the same
explicit native cache. Tests that intentionally isolate cache behavior retain
their explicit per-command override. Each command has a 30-minute limit and an approximately 5 GB
process-tree budget. Counter failures stop the run. Windows uses owned process
tree termination; POSIX uses an owned process group plus tracked descendants.
The watchdog itself needs real validation on each host before its bounds are
described as established platform evidence.

Dependency metadata checks archive identities, notices and declared Rust version
requirements. A missing `rust-version` is recorded as undeclared; it is not proof
of compatibility with the pinned compiler. Successful native compilation and
execution remain separate requirements. Audit findings and retained optional UI
warnings must be reported with their active feature scope, without ignore flags.

The runner builds the selected targets together, requires terminal Cargo JSON
success, finds each exact test executable, and records executable and CLI/LSP
hashes. It runs targets serially, batching their pending exact cases with
`--test-threads=3`. Only passes
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

Dispatch the existing registered CI workflow with `gh workflow run ci.yml --ref <branch> -f supporting_host_timing=true` to invoke its reusable `Supporting-host UDP timing` helper. This mode measures only `net_udp_large` on Linux, where the 60000-byte datagram is supported. Platform acceptance jobs provide correctness evidence and do not measure performance. This separate workflow retains the existing native, hand-written Rust and Node-after-tsc controls, three processes with 31 samples after eight warmups, and the quiet-machine guard. It validates all nine process rows before ingesting the one actual result and uploads that exact JSON for root verification and ingestion. The caller and helper have separate concurrency groups, both distinct from ordinary CI, so this mode does not cancel live platform jobs. The final status is supporting-host timing, not full or platform acceptance. A supporting-host result does not claim delivery or timing support on hosts that reject this datagram size.

For the reviewed repair of run `42460717406` only, dispatch existing `ci.yml` with
`platform_acceptance=true` and `platform_prior_run=42460717406`. The job downloads
its own host artifact and validates the exact results SHA and original checkpoint
`469e8bb32a86771eb3317cd3232554c6a4192752`. Freshly built, validated binaries list
all61 targets. Failed, incomplete, new and previously unreached cases execute;
the shared55, relocated exports, pure/genuine Rayon and async IO/worker controls
repeat on every host. Other passes retain their OLD source/consumer identities.
The same single55-minute budget includes retrieval, build and cleanup. Results
say `scope: focused-repair`, `repairComplete` and `complete: false`; this does not
claim a full current-source platform run or permit missing scopes to disappear.
Unset the input for unchanged full-scope behavior. No tests or timing controls are
removed, and budget exhaustion remains incomplete acceptance.
