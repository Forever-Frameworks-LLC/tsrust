# Tsrust handoff

This is the single handoff note (owner direction, 2026-10-30): keep it current
and do not create separate handoff files. Older handoffs and the long
`agent-progress.md` history were deleted; they remain in Git history.

## Start here

Repository: `/Users/seancotter/Desktop/dev/learning/tsrust`.
Branch: `codex/optimize-native-hot-paths` (staging PR #8; see promotion status below).
Read `AGENTS.md`, `docs/development-workflow.md` and `docs/agent-goal.md`, then
this note. Inspect current source and `git status` before acting.

## Current state — 2026-10-30 (optimization loop; read first)

Active owner direction: optimize toward "making Rust as easy to write as
TypeScript": JS/TS-familiar API, Rust semantics and accuracy underneath, generated
code at hand-written-Rust parity.

Owner milestone correction, 2026-11-02: treat the remaining large performance gaps
as one optimization milestone. Continue profiling/optimizing until only small
refinements remain; do not run broad/full regressions between optimization
clusters. Run affected suites and quiet matched A/B checks for each coherent
change. At the final milestone, finish broad regressions, refresh affected public
performance metrics, then promote/push to main. While CI/CD runs, reprioritize
next steps with the owner, including possible UI-framework work. UI revival is a
proposal for that discussion, not a change to current backend scope.
The docs restructuring in the next section remains open and is not the current
optimization focus.

Owner steering, 2026-11-03: HTTP/object ergonomics supersede optimization until this section is delivered. The hello expression `c.json({greeting:clone(c.env.greeting)})` must work; routes, middleware, authorization and client generation must stay in source, without API settings in tsrust.config.json. Later requests require hover on literal keys plus useful next() hover/signature help.

**Final schema/ownership gap audit — profiled; no product change retained; parent `cbc2aaad`.**
Previous goal turn made progress: accepted schema-array storage in 67479689,
110 checks, three native histories, public selection 108 runs, paired installation
and cleanup (metadata cbc2aaad). Current tree began clean; no live jobs inherited.

Fresh owned-string-array reversal allocation profile: original 100,000 iterations,
checksum 571742071. Native/Rust 6/5 setup allocations, requested/peak 147/27
bytes, input 11, three identical samples, zero reallocations/retained/cleanup.
Native CPU sample (3 seconds of an 8-second direct-call loop) completes 4,788
batches; 1,953 samples in inlined loop work, 278 memcmp and 45 memcmp stubs.
Watchdog 11.882 s / 267,104 KiB. No per-reversal allocation problem.

Generated-only four-way probe retains original five identities, 8 warmups /
3×31 samples / two reverse-order rounds, three warm rotated compiles, same
source/executable paths and argv0, atomic swaps and unsafe forbidden. Runtime
parent 1.810291/1.768333, fixed [String;5] 1.639541/1.639208, Vec
1.645209/1.636250, cached repeated indexed read 1.748750/1.744416 ms.
Compile medians 277.289/295.474/270.037/273.871 ms; max load 2.324219,
watchdog 9.316 s / 309,984 KiB. Fixed storage's ~6.6% compile increase
rejects that prototype; caching gains little. Vec removes the carrier branch but
would require new local-storage specialization and safe fallback proofs; defer
that ~8% residual rather than changing ordinary small-array ownership/storage.
Follow-up forced-inline reverse/read/all variants are inconsistent: parent
1.976542/1.777458, reverse 1.779750/1.796875, reads 1.784000/1.793291,
all 1.773875/1.805667. No product change, ingestion or timing claim from these
probes. Cached Node/Rust execution references remain unchanged.

Source-retaining schema split profile passes 22.242 s / 337,206 KiB.
The first custom CPU driver assumed function0 and failed to compile; repaired it
with the existing adapter for the fallible generated signature. Original case
bodies/assertions remain intact, independent checksums 4*n+7=8007 / n+7=2007.
Three identical allocation samples/engine/case, no reallocations/retention/cleanup:
success native/Rust 22,000/22,000 blocks, requested 3,318,000/4,358,000,
peak 1,523/2,051; missing-field 14,000/10,000, requested 1,962,000/2,010,000,
peak 813/893; input 52. Eight-second native loops complete 6,577 success /
19,581 failure batches, sampled 3 seconds each. Success samples show repeated
RawValue/source cursor scans plus final retained-value parsing; failure allocation
and Issues::push dominate. mach_absolute_time is under allocator free, not a new
language clock check. This reproduces the earlier successful allocation parity;
the remaining error mismatch includes richer public diagnostics.

Two new generated/source-only alternatives were measured before implementation:

- Callback-free retained visitor reuses the existing checked JSON Value Seed,
  preserves source fallback, canonical extra-key ordering and exact token scan.
  Original five identities, original timing/compile protocol and unsafe ban pass.
  Parent 1.658042→1.513917 / 1.664375→1.540333 ms, compile
  711.488→787.898 ms (+10.7%). Blocks 36,000 unchanged, requested
  5,280,000→6,762,000, peak 1,523→1,635, checksum10007, cleanup0.
  Watchdog 11.397 s / 362,896 KiB, max load1.846191. Rejected: compile
  and temporary-memory regressions outweigh the modest runtime gain.
- Reuse JSON.parse plus the existing owned schema validator: diagnostic source
  preserves all valid-value/missing-field checks, while removing only the public
  JSON-versus-validation wrapper check. This is a hypothesis probe, not exact
  API-preserving A/B acceptance. Original five checksum identities and unsafe ban
  pass; parent 1.670667→1.302542 / 1.665584→1.301125 ms, compile
  714.608→661.773. Blocks 36,000→38,000, requested
  5,280,000→5,988,000, peak1,523→1,699, checksum10007, cleanup0.
  Watchdog9.883 s /349,450 KiB, maxload3.443359. No product change:
  eagerly owning declared keys adds allocations and memory. Do not route
  safeParseJson through a different public error API to obtain these numbers.

These diagnostic probes are not ingested or published; Node/Rust execution
references and all SQLite histories remain unchanged. No compiler source changed,
so affected native/compiler suites are not rerun merely for profiling. Existing
boxed labels remain accepted; static/Cow/tagged paths, forced inlining and capped
reservations remain rejected. Avoid widening storage specializations or adding
arenas/RC for these residuals. The measured source-decoder tradeoffs may inform a
future retained-data decoder feature, but require exact error, callback, duplicate,
Unicode/integer, recursion and ownership parity plus actual-emission controls.

All current profile/probe jobs are terminal, including repaired profile97234 and
DOM probe39914. Rejected-outcome checkpoint `247e11f7`; pruned 63 closed
scratch/profile/probe artifacts (16957614 bytes). Shared Cargo/tests/consumers remain.
Next: audit the stale callable-array-transfer signal against current emission;
selected cached native/Rust once_transfer 1.828541/1.485042 ms,
transfer_pipeline 1.654417/1.351500, mutable_pipeline 1.329167/1.046416.
These are ranking leads, not new execution measurements. Start with
benchmarks/runtime/workloads/callable_array_once_transfer.tsr and its parity
reference, allocation counts and a bounded CPU sample; preserve original oracles.
finish the remaining large-gap assessment before final broad regressions,
affected performance/HTTP refresh and approved v0.1.1 promotion. No new estimate
or acceptance milestone is implied by these diagnostic measurements. Goal active.

**Plain schema-array output storage — accepted locally, measured and installed; parent `8cc9278a`.**
Previous goal turn made progress: hypot accepted in bdc29b8c/8cc9278a, 24
checks, one native history, public selection 107 runs, paired installation and
cleanup. Owner then asked for an estimate; estimate was 2–4 working days,
not a changed deadline or reduced goal. Focus this pass on remaining genuine
schema/ownership gaps; leave small math/Unicode refinements alone.

Split owned/JSON array success and two-error workloads, preserving all original
assertions and independent checksum 4007 for 2,000 iterations. Three allocation
samples per engine/case agree, no reallocations or retained/cleanup bytes:
owned success native/Rust 4,000/2,000 allocations, 192,000/32,000 requested,
96/16 peak; owned failure 14,000/12,000, 1,504,000/640,000 requested, 712/304
peak. JSON success 6,000/4,000, 448,000/288,000 requested, 224/144 peak; JSON
failure 16,000/10,000, 1,760,000/576,000 requested, 712/288 peak. Input bytes
11 owned / 5 JSON. Native owned CPU samples (3 seconds of 8-second batches)
complete 80,781 success / 24,121 failure batches. Success spends time in
validate_array plus malloc/free; failure is allocation-heavy, including the
internal-to-public diagnostic bridge. mach_absolute_time is beneath allocator
free, not a new language clock/deadline. Profile 23.171 s / 289,088 KiB.

The concrete success defect: plain owned/JSON validation builds Vec output and
then copies it into final NumericArray/Array storage. Inlined final-array loops
remove one allocation per successful call and improve runtime, but native compile
cost rises ~10–15 ms; do not select that form. Shared target-carrier prototype
reuses the validation loop with an empty final carrier and its push function:
owned parent .486958→.377833 / .463000→.373500 ms, compile 382.515→373.260 ms;
JSON .669875→.597500 / .605917→.579125, compile 509.101→494.846 ms. Both
retain original 5 identities, 8 warmups / 3×31 samples / two reverse rounds,
three warm rotated compiles, same paths/argv, atomic swaps and unsafe ban.
Owned/JSON target probes 9.369 s / 305,434 KiB and 11.399 s / 284,870 KiB;
maximum loads 2.047852/1.937012 < 4. Full owned blocks 18,000→16,000,
requested 1,696,000→1,632,000; JSON 22,000→20,000, 2,208,000→2,144,000.
Peak 712, zero reallocations/retained/cleanup and checksum8007 unchanged.
Diagnostic prototypes are not ingested; Node/Rust execution timings remain cached.

Implemented runtime/plan.rs validate_array_into with generic carrier and standard
FnMut push; existing Vec-returning validate_array is a compatibility wrapper.
Both plain owned and JSON emitters pass their final carrier directly. All elements
are still validated after rejection; partial output is dropped before the next
callback, and later successful outputs are discarded. No new trait, dependency,
API, limit, ownership rule or generated unsafe. Pipeline/trial rollback storage
is separate and unchanged. Added a non-Clone/non-Default carrier unit test; existing
partial-drop-order test remains required. New schema_array_storage O0/O3 witness
asserts both lowered calls, zero allocation for small numeric output, pointer-
preserving owned strings across inline/heap boundaries, complete cleanup and all
plan module units under unsafe forbidden (allocator instrumentation is separate).

Selected joint compiler/LSP no-run passed 209.336 s / 2,665,682 KiB.
The initial storage witness had harness defects: standalone checking cannot resolve
std:schema imports, and owned-only schema emission does not contain JSON lowering.
Fixed both with separately formatted, project-checked owned/JSON sources. A raw
unsafe substring assertion also matched a JSON runtime documentation comment;
removed that assertion. Actual benchmark compiles still forbid unsafe code.
The allocation witness compiles independently emitted pure numeric/string carrier
support plus the actual plan module, avoiding unnecessary Serde linkage. All
plan module units still run at O0/O3 with unsafe forbidden. No product failure
was found in these harness repairs. Same selected joint no-run relink passes
(3.708 s / 337,200 KiB on first repair; final relink 3.121 s / 261,824 KiB; log remains in
/tmp/tsrust-schema-array-build.{jsonl,log}).

All 110 affected checks pass, none ignored, serial binaries / 3 threads /
5 GB–30 min watchdog. Storage 1 (2.064 s / 135,968 KiB), decode 10
(30.162 s / 1,086,016 KiB), parse 5 (2.097 s / 661,648 KiB), pipelines
20 (49.270 s / 1,047,600 KiB), collections 5 (8.874 s / 942,768 KiB),
unions 13 (56.394 s / 1,143,312 KiB), LSP schema_editing 17 (.523 s),
project intelligence schema_ 39 (.513 s, 142 unrelated filtered). LSP runs
finish too quickly for meaningful sampled memory. Orchestrator 99587 is terminal.

Actual-emission A/B against 8cc9278a passes original five identities,
8 warmups / 3×31 samples / two reverse rounds, three warm rotated compiles,
same source/executable paths and argv0, atomic swaps and unsafe forbidden.
Owned .433125→.400125 / .432125→.407000 ms; compile 388.633→378.161 ms.
JSON .670834→.602125 / .661125→.608000; compile 515.932→505.248 ms.
Owned/JSON watches 6.755 s / 351,952 KiB and 6.746 s / 315,888 KiB;
maximum loads 2.726074/2.847656 < 4. Three allocation samples/version agree
with the prototype: owned 18,000→16,000 allocations, JSON 22,000→20,000;
requested bytes both -64,000, peak 712 and zero reallocations/retention/cleanup.

Exact int-to-number array first control is noisy: .374292→.340750 /
.360417→.405792 ms. Preserved that result, then reran the same protocol:
.411334→.326625 / .354917→.319875, compile 356.079→346.838 ms;
6.205 s / 350,144 KiB, max load 2.680176. Adjacent collection control
1.837459→1.822000 / 1.822167→1.817625, compile 1084.981→1077.613;
12.761 s / 378,896 KiB, max load 2.640625. No material regression.
Fresh emitted-call inventory confirms only owned_array, json_array and
number_array_int use the changed lowering; export/recursive/json_nested/pipeline
have zero calls. Cached Node/Rust execution timings remain unchanged.

Canonical release refresh passes 303.516 s / 2,189,996 KiB, timestamp
2026-11-04T12:53:16.135Z. Quiet guard waited through load 6.80/5.73/4.61/
4.58; accepted maximum 3.997559 < 4, no override. Each workload retains 93
original samples and all five independent correctness identities. Independently
recomputed medians/mean rates: owned .366792 ms / 9,016,993.558 ops/s;
JSON .582959 / 6,483,837.318; int-to-number .313042 / 12,514,810.688.
Debug/release emissions and canonical source/emitted SHA256 match for all three.
`npm run store:ingest` adds exactly three native histories; independent full-row
comparison proves every previous entry and all external reference histories
unchanged, SQLite integrity OK. Public selection now 108 accepted runs / 933
comparisons. Both performance-page tests and 67-page site build pass
(4.149 s / 1,102,060 KiB). No before/after or date columns added.
Whole-project formatter passes: 0 files, 0 snippets, 0 fingerprints changed.

Validated paired release consumers atomically installed and SHA256-verified:
CLI a5ea32c8a8dfbfd50df036b8473957e8b55da01323c9df991c1ab0bda6769e8b;
LSP f02b2ed5ad604d30f3ce7708ded1fae20d93e7d8f1726918ce521f40a613cd30.
Installed CLI checks unchanged hello/src/main.tsr; owner source/config hashes
unchanged, editor processes untouched. All current jobs are terminal. Source checkpoint `67479689` contains implementation, tests, three histories and
public selection. Pruned 113 closed scratch/raw/native artifacts (24845568 bytes);
shared Cargo libraries, current test binaries and paired consumers remain.
Next: fresh audit of the remaining error-bridge/schema and stale ownership gaps.
Fresh selected-history audit still shows source-retaining schema 1.653458 /
Rust 1.212250 ms, collections 1.691500 / 1.308709, and owned-string reverse
1.973667 / 1.634917. Treat these as profiling leads, not newly measured claims.
The reverse workload measures repeated content classification and checksum modulo
in addition to reversal; isolate those costs before changing array storage.
Full regressions/CI/main remain deferred to the large-gap milestone; goal active.
The previous timeline-only reply changed no authoritative state; this continuation
repairs the harness and completes accepted implementation and measurements.

**Math.hypot — accepted locally and installed; parent `d5caa070`.**
Source checkpoint `bdc29b8c` contains the accepted runtime/test, SQLite history and
performance selection. After independent ingestion verification and commit, pruned
64 closed scratch/raw/profile artifacts (16950952 bytes). Shared Cargo libraries,
current math/spread/editor tests, immutable MPFR references and paired consumers
remain reusable. All jobs are terminal. Next: fresh schema/ownership gap audit
from current code and the ranking below.

Previous goal turn made progress: extrema accepted in 52a767af/d5caa070, 23
checks, two native histories, public selection 106 runs, paired installation and
cleanup. This section began with a clean tree and no live jobs.

Fresh CPU profiling passes 11.348 s / 258,512 KiB: 3-second sample of 8-second,
131,072-iteration batches, each checked against the immutable MPFR accumulator
interval; 6,302 batches. Samples: hypot 1,172, hypot stub 281, main 705. First
prototype skips zero-to-first hypot for finite values, preserving NaN payloads:
1.204708→1.131417 / 1.206667→1.149083 ms, compile 206.417→206.544 ms.
Single-pass infinity handling gives little further gain but compile regressions:
parent 1.260209/1.206625, single 1.191791/1.176083, seeded single
1.185833/1.137958 ms; compile 211.655→239.439/241.810 ms. Rejected.

Selected deferred correction plus seeded fold: parent 1.204167→1.002500 /
1.204125→.995750 ms (1.201×/1.209×), compile 206.377→205.346 ms. Deferral
alone is smaller, 1.151750/1.149667 ms, compile 206.471 ms. All variants pass
76,683 raw-bit edge/random vectors at O0/O3 against the original reducer and all
40 immutable MPFR points. Five original workload identities, 8 warmups / 3×31
samples / two reverse rounds, three warm rotated compiles, same paths/argv,
atomic swaps and unsafe ban pass. All three allocation samples/version have
zero allocation/reallocation/requested/retained/peak/input/cleanup counts; actual
bits 411782b65954281c. Original artifacts remain unchanged. Watchdogs: seeded
7.763 s / 309,904 KiB; single-pass 10.367 s / 261,136 KiB; deferred 10.349 s /
308,160 KiB. Maximum loads 2.282715/1.913086/2.213867 < 4. These diagnostic
prototypes were not ingested; Node/Rust execution references remain cached.

Do not simply delete infinity correction. Pinned Rust 1.96.1 std source calls
libc hypot on Unix/Windows and documents unspecified platform precision.
[Gnulib's primary portability manual](https://www.gnu.org/software/gnulib/manual/gnulib.pdf)
records mixed NaN/infinity discrepancies on mingw/MSVC 14. Instead compute the
ordered fold, then scan the original values only when its result is NaN; actual
infinity inputs still produce positive infinity. A cloned work iterator preserves
the original including its first value. Finite first values use abs; NaN first
values keep 0.hypot(value) for the existing quieting/payload behavior. Empty,
signed zero, overflow/underflow and remaining operation order stay unchanged.

Implemented only runtime/math.rs hypot helper; ordinary and spread callers traced.
No dependency, API, limit, ownership or generated unsafe change. Added one
math_approx O0/O3 witness comparing the original ordered standard fold through
slice and segmented-iterator adapters, for host math plus an emulated mixed
NaN/infinity backend discrepancy: 613,464 exact-bit comparisons. Existing eager
operand/disposal/classification/arity/spread and MPFR checks remain required.
Joint compiler/LSP selected no-run passes 208.839 s / 2,642,066 KiB; no --lib
or broad unit preparation. All 24 affected checks pass, none ignored, serial
binaries / 3 threads / 5 GB–30 min watchdog: math_approx 6 (2.601 s / 175,680 KiB),
math_exact 5 (2.600 s / 357,600 KiB), root_builtin_spread 5 (2.654 s /
467,248 KiB), compiler iterable_call_spread 3 (2.133 s / 276,464 KiB), LSP
iterable_call_spread 3 (.523 s), intelligence math 2 (.514 s). LSP runs finish
too quickly for meaningful sampled memory. Host/emulated backend witness passes
613,464 exact-bit comparisons across O0/O3 and both adapters.

Actual-emission A/B against d5caa070 passes 1.206542→1.002250 /
1.203333→1.006959 ms (1.204×/1.195×), compile median 210.796→206.903 ms.
All original workload identities have identical raw bits and pass immutable MPFR
case intervals; both helpers pass all 40 MPFR points and O0/O3 raw-bit edges.
Original protocol retained: 131,072 iterations, 5 identities, 8 warmups / 3×31
samples / two reverse rounds, 3 warm rotated compiles, same paths/argv, atomic
swaps and unsafe ban. Watchdog 8.292 s / 323,344 KiB, maximum load 3.936035 < 4.
Allocation samples are unchanged at zero; original reference artifacts verify
unchanged. No accuracy/rounding/ownership change.

Adjacent Math.max control: .839083→.841291 / .838708→.838708 ms, compile
211.753→213.149 ms, maximum load 2.682129 < 4. Runtime watch 4.671 s /
266,000 KiB. Corrected a control compile-order typo and repeated only the warm
compiles (4.153 s / 338,784 KiB); preserved completed runtime evidence. Control
emissions differ only in unused hypot body. Interfaces, path normalization and
Unicode normalization emit byte-identically. No unresolved material regression.
Cached Node/Rust execution references remain unchanged.

Canonical native-only run 2026-11-04T11:54:33.518Z passes 219.237 s /
2,188,524 KiB including paired release build, native MPFR/all-point probes and
93 samples. Maximum load 2.681152 < 4; no override. Median .999459 ms /
125,797,087.627 mean operations/s, original 131,072 operations/sample. All warmup
and sample bits remain 411782b65954281c. Final debug/release emissions and
canonical source/emitted hashes verify. npm run store:ingest adds exactly one
native history; independently recomputed median/mean rate/operations/count/date
match. All earlier histories and all Node/Rust references are unchanged; SQLite
integrity OK. Performance selection: 107 accepted runs / 933 comparisons. Two
page tests and 67-page build pass (3.697 s / 1,184,654 KiB). Whole-project
formatter changes 0 files/snippets/fingerprints; git diff --check passes.

Paired release consumers atomically installed and SHA256-verified:
CLI f3d9703225f0b7192d6e0a5625bfd11cf5b890bd92b07ee0001301c1ac6fe62a;
LSP 274106b34260e532cc2c91f43302c5670d5ef639870e53ad860e8c55df13c22c.
Installed compiler checks unchanged hello/src/main.tsr; owner source/config hashes
unchanged, editor processes untouched. All current jobs are terminal. The remaining
hypot difference versus saved Rust .930416 ms is .069043 ms / 7.42%; this large
gap is now a small residual. Do not reopen the rejected explicit-loop variants.
No full regressions, CI, push or publication; local performance page updated.

Next audit the remaining CPU/ownership/schema gaps against current output:
source-retaining schema 1.653458 / Rust 1.212250 ms, schema collections
1.691500 / 1.308709, schema JSON array .664542 / .472625, owned array
.404959 / .250625. Profile split success/error work before implementation; retain
rejected static/Cow/path-tag/arena hypotheses below unless fresh evidence changes
the tradeoff. Check stale owned-string-array reverse and callable-array-transfer
signals before treating their old gaps as current. Generated strings already use
std::borrow::Cow<'static,str> and valid UTF-8 storage (runtime/strings.rs); do not
infer an obsolete UTF-16 enum/layout defect from old history. Preserve the object
allocation plan. Full regressions, affected HTTP metric refresh and approved
v0.1.1 main promotion remain deferred until the large gaps reach small refinements.

**Math.min/Math.max — accepted locally and installed; parent `a22f7466`.**
Source checkpoint `52a767af` contains the accepted runtime/test, SQLite histories
and public selection. After independent ingestion verification and commit, pruned
67 closed scratch/raw/profile artifacts (17270299 bytes). Shared Cargo libraries,
current math/spread/editor tests and paired consumers remain reusable. All jobs
are terminal. The next executable action is fresh Math.hypot profiling.

Fresh CPU profiling passes 19.541 s / 302,096 KiB: 3-second samples of 8-second,
500,000-iteration batches, checked against independent exact-bit oracles.
Max/min complete 7,914/7,894 batches; computation is fully inlined, with no
allocation or I/O. Generated-only intrinsic reductions improve max
1.038666→.840917 / 1.027916→.838459 ms and min
1.014542→.839417 / 1.011375→.838833 ms. Seeding from the first element adds
little benefit and more code; keep the smaller intrinsic-only change. Prototype
warm compile medians: max 210.548→212.593 ms, min 211.253→210.733 ms.
Both probes pass the original protocol and O0/O3 equivalence against the original
reducer on 76,683 edge/random vectors with both extrema. Max/min watchdogs:
8.246 s / 289,737 KiB and 6.694 s / 271,456 KiB; maximum loads 1.926270/2.065430.
Prototype results were not ingested.

The shared runtime/math.rs reducer now uses standard f64::max/min after the
existing signed-zero branch. Infinity initialization, early NaN rejection, empty
arity and eager operand staging remain unchanged. Ordinary and spread callers
share this fix. No helper, dependency, API, limit, ownership or generated unsafe
change. The existing math_exact O0/O3 witness now checks an independent
NaN-filtered total_cmp reference through slice and segmented-iterator adapters:
76,683 vectors, both extrema, both adapters and optimization levels total
613,464 exact-bit comparisons. Includes subnormals, finite bounds, infinities,
signaling/quiet NaNs, signed zero and lengths 0–8. Existing effect-order,
empty/spread, disposal and type checks pass.

Joint compiler/LSP no-run passes 479.815 s / 3,611,715 KiB. All 23 affected
checks pass, none ignored, serial binaries / 3 threads / 5 GB–30 min watchdog:
math_exact 5 (2.592 s / 190,592 KiB), math_approx 5 (2.087 s / 233,760 KiB),
root_builtin_spread 5 (2.633 s / 490,144 KiB), compiler iterable_call_spread 3
(2.093 s / 353,232 KiB), LSP iterable_call_spread 3 (.516 s), intelligence math
cases 2 (.515 s). LSP cases finish too quickly for meaningful sampled memory.
Library unit binaries were built but not run. Future routine no-run builds should
omit --lib unless affected unit filters will run; preparing unused broad unit
binaries adds substantial build time. Keep both package selections and one target.

Actual-emission A/B against a22f7466 passes. Max: 1.017917→.839792 /
1.020958→.838250 ms (1.212×/1.218×), compile median 210.294→211.188 ms.
Min: 1.015208→.841875 / 1.013750→.838666 ms (1.206×/1.209×), compile
212.438→212.118 ms. Original five independent checksum/bit identities, O0/O3
edge comparisons, 8 warmups / 3×31 samples / two reverse rounds, three warm
rotated compiles, same paths/argv, atomic swaps and unsafe ban pass. Max/min
watchdogs: 4.673 s / 280,944 KiB and 6.748 s / 312,688 KiB; maximum loads
2.722168/2.387695 < 4. Three allocation samples per version are identical:
all allocation/reallocation/requested/retained/peak/input/cleanup counts zero.
Max checksum 521614782, exact n:414d19aea0000000. No material compile regression.

Math.round control varies in opposite directions: .510917→.499958 /
.508916→.541833 ms; compile 209.842→209.069 ms, watch 4.662 s / 301,600 KiB,
maximum load 1.876465. Complete O3 assembly at the same source/output path is
byte-identical, SHA256 a8e1e49d2d9a7fb099898b66f40151a125280659c6079b187963fa42de6cfcae.
Math.round/Math.hypot emissions differ only in the unused extrema body;
interfaces/path normalization emit byte-identically. Assembly/emission proof:
2.079 s / 173,104 KiB. Control variation is resolved; no unresolved regression.
Cached Node/Rust execution references were not repeated.

Canonical native-only run 2026-11-04T11:24:40.069Z passes 215.734 s /
2,245,755 KiB including paired release build, independent correctness cases and
93 samples per workload. Maximum load 2.441406 < 4; no override. Min median
.896708 ms / 563,936,902.067 mean ops/s; max .896875 ms / 560,489,595.425.
Each sample retains the original 500,000 operations. Final debug/release emissions
match and canonical source/emitted hashes verify. npm run store:ingest adds exactly
two native histories; independent medians, mean rates, operations, counts and dates
match. Every earlier history and all Node/Rust references remain unchanged;
SQLite integrity OK. Public selection: 106 accepted runs / 933 comparisons.
Two page tests pass; 67-page website build 3.647 s / 1,210,846 KiB. Whole-project
formatter changes 0 files/snippets/fingerprints; git diff --check passes.

Paired release consumers atomically installed and SHA256-verified:
CLI 957eafec3b00250e62f8a1359e0dc9c107e1cdedc5ba353fc144404cddb07077;
LSP d93f8b1c5e3a37fba10cd8cb002e13b610afa8923053ba67303dcb8b25e2d97a.
Installed CLI checks unchanged hello/src/main.tsr; source/config hashes unchanged,
editor processes untouched. All current jobs are terminal. Local performance page
updated; no full regressions, CI, push or publication. Goal remains active.

Next profile Math.hypot (saved native 1.294958 / Rust .930416 ms, .364542 excess).
Inspect the initial zero-to-first hypot operation versus a seeded reduction,
preserving infinity-over-NaN precedence, empty/single inputs, signed zero,
subnormals, overflow/underflow and rounding. Do not change it without fresh
profile/edge proof and matched A/B. Source-retaining schema remains .441208 ms
over saved Rust; rejected alternatives below remain rejected. Full regressions,
affected HTTP metric refresh and approved v0.1.1 promotion stay deferred until
large gaps reach small refinements. Do not infer milestone completion from this
extrema checkpoint.

**Normalization composition bound — accepted locally and installed; parent46c09058.**
Source checkpoint `e823b0f4` contains the accepted runtime/test,SQLite history and performance selection. After independent ingestion verification and commit,pruned92closed scratch/raw/profile artifacts (35204606bytes). Shared Cargo libraries,current numeric/normalization test binaries,paired consumers and exact schema/Nix args retained. All section jobs terminal. Next executable action:profile Math.min/Math.max against this checkpoint before any implementation;preserve special values,empty arity,spread behavior and eager argument evaluation.

Previous goal turn made progress:accepted typed JSON/safe close,ingested5native histories,updated local performance selection104runs/933comparisons,installed paired consumers,verified hello and next/key hovers,committed06ac9edd/46c09058. No jobs remained live. Current source remains unchanged.

Fresh direct-work schema CPU samples on46c09058 pass28.911s/315,856KiB:3s samples of8s loops,success/missing/record checksums8007/2007/4007,original2000iterations,unsafe forbidden. Success still spends time in retained dynamic decoding,RawValue scanning and allocator calls;missing/record remain allocation-heavy. Tested a generated-only trusted-cursor retention decode using the existing strict Seed<false> visitor without redundant end/integer rescans. It preserves private RawValue property semantics;direct serde_json::Value cannot be substituted because its reserved wrapper key is observable (existing witness). Rejected:no runtime gain1.693625→1.710208/1.659375→1.671292ms;compile711.078→719.247ms. Three identical samples/version remain36,000blocks/5,280,000requested/1,523peak/0reallocations,retained and cleanup0,checksum10007. Original5identities/8warmups/3×31samples/two rotatedrounds/3warmcompiles/samepaths+argv/atomic swaps pass;watch9.373s/353,680KiB. No benchmark ingestion or product edits. Do not repeat this hypothesis.

Normalization CPU profile passes15.158s/266,400KiB:3s sample of8s direct2048-iteration batches,3791batches,output-length identity2048,no observer/instrumentation;889top-of-stack normalizer samples plus string-extension/allocation/free/growth costs. Generated-only composition-bound probe passes8.302s/338,304KiB:parent1.852333→1.796334/1.960833→1.801833ms,compile371.864→370.701ms. Bounded placement comparison passes13.047s/311,968KiB:after-Hangul1.862250→1.791625/1.874875→1.789041ms (~4%);before-Hangul1.796250/1.788875 adds no benefit. Compile medians parent384.012/after378.608/before390.928ms;early placement not selected. All3versions share30,721allocations/5,742reallocations/1,384,011requested/489,539retained/489,581peak/input19/cleanup0/checksum332162546,three identical samples. Original2048iterations/5identities/8warmups/3×31samples/two rotatedrounds/3warmcompiles/samepaths+argv/atomic swaps/unsafe ban pass,maxload3.885254<4;Node/Rust execution references remain cached. Diagnostic prototypes are not ingested.

Implemented only two-line guard after existing Hangul cases in runtime/normalization.rs:table composition cannot match a right scalar belowU+0300. No ownership/API/default/dependency/unsafe/table-data change. Existing O0/O3 normalization-pages harness now asserts all961tablepairs satisfy the bound and compose exactly,tests boundary/nonmatches,and all399leading/vowel plus10,773tailHangul combinations. Existing official399,300normalization and1,101,784identity outputs,2,224,128decompositions,allocation/capacity and editor cases remain required. Selected joint compiler/LSP no-run build passes523.158s/3,615,331KiB (includes new --lib feature context and dependency rebuild);unit libraries built but not run. All14affectedchecks pass,none ignored,serial3threads/5GB30min:compiler normalization12 (4.715s/455,792KiB),editor normalization2 (1.043s/41,936KiB). All conformance/decomposition/identity/capacity/ownership/new composition and Hangul checks remain green. Actual-emission A/B against46c09058 oldrelease/currentdebug passes1.852208→1.809792/1.854292→1.800708ms (1.023×/1.030×);compile medians375.854→369.872ms with overlapping372.159–377.541/367.578–382.847ranges. Three allocation samples/version exactly match the prototype;no memory/ownership change. Original2048iterations/5oracleidentities/8warmups/3×31samples/two rotatedrounds/3warmcompiles/samepaths+argv/atomic swaps/unsafe ban,maxload2.210938<4,watch6.723s/378,560KiB. Actual gain is2–3%;do not describe it as the larger prototype percentage. Interfaces,pathnormalize andradix controls emit byte-identically;no controltiming needed. No unresolved regression.

Canonical native-only refresh2026-11-04T10:57:53.442Z passes219.394s/2,227,644KiB including pairedrelease build;all independent correctnesscases and93samples,maxload3.088867<4,nooverride. Median1.904416ms/1,091,507.466meanops,original2048operations/sample. Finaldebug/release emissions equal and canonical source/emittedSHA match. npm run store:ingest adds exactly1native history;independent median/meanrate/ops/count/date recomputation and all priorhistoryhash/count comparisons pass,all Node/Rust references unchanged,SQLite integrity OK. Public selection105accepted runs/933comparisons;two page tests pass;67-page site build4.168s/1,161,454KiB. Wholeformatter0files/snippets/fingerprints,git diff --check passes. Remaining normalization overhead versus savedRust1.636ms is.268416ms/16.41%;not parity.

Paired release consumers atomically installed and SHA256-verified:CLId132d1384395d5ec91b5a984640b52475273c6893e8e7252f738cf9d1edbd09e;LSP274106b34260e532cc2c91f43302c5670d5ef639870e53ad860e8c55df13c22c. Installed CLI checks unchanged hello/src/main.tsr;owner source/config hashes unchanged,editor processes untouched. All current build/test/profile/timing/site/install jobs terminal. No full regressions/CI/push/publication;local performance page updated,live deployment deferred. Next profileMath.min/Math.max using fresh actual output and independent NaN/signedzero/arity/evaluation-order witnesses. Broader SQLite ranking audit finds Math.max native1.109791/Rust.742916ms (.366875excess,1.494×),Math.min1.085125/.742959 (.342166,1.461×),Math.hypot1.294958/.930416 (.364542,1.392×). Public selected native/Node rows are still2026-10-29,not fresh. Current extrema reducer starts at infinity and branches per operand;hand-written reference uses NaN/signed-zero-correct pair reductions. This is a new profiling lead,not a gain claim. Profile extrema after this accepted normalization checkpoint;do not restrict the final audit to the old schema-only ranking. Full regressions/main promotion remain deferred to the owner’s large-gap milestone.

Owner next() hover follow-up reverified against the installed LSP directly over stdio: next() includes Task<void>, downstream behavior, ownership and single-use guidance; greeting:clone(c.env.greeting) includes greeting:string. Watchdog0.521s/6,192KiB,exit0. Repeated after installing the final pair below: both still pass. No editor process control or owner source changes. Actual editor rendering remains unverified.

**Safe consuming close and typed JSON — accepted locally and installed; JSON source checkpoint8072665c.**
Final JSONL canonical run2026-11-04T10:35:10.536Z passes,339.683s/2,185,116KiB including paired release build and quiet wait;max timed load3.459473<4,no override. Typed20.422292ms/97,850.829meanops,borrowed20.353166ms/97,993.985meanops,93samples each. JSONL matched A/B below establishes no runtime regression and12,000 fewer allocations per batch. Do not claim compile speedup: safe close adds a small required dependency metadata cost.

Independently verified all five workload source hashes,final debug/release emission equality,and canonical emitted hashes. Original pure JSON3 emissions remain identical;reuse their original93samples at2026-11-04T09:19:59.562Z (literal.214459ms/7,825,603.715meanops;typed.248500/7,306,789.065;JSONC.522709/3,671,456.950). Filtered only these3 execution workloads into the accepted subset;obsolete old JSONL rows were never imported. npm run store:ingest adds exactly5native histories across2acceptedtimestamps. Independent median/meanops/operations/samplecount/date recomputation and unchanged prior histories pass;all Node/Rust histories unchanged;SQLite integrity OK. Performance selection104 accepted runs/933 comparisons;two page tests pass;67-page site build4.164s/1,212,782KiB. Whole-project formatter0files/snippets/fingerprints;git diff --check passes.

Paired final release consumers atomically installed and SHA256-verified: CLI55408449629fbd6af3b880b8d97bd90bae78f3a82726273a7ee1eaef2e853d7c;LSPd93f8b1c5e3a37fba10cd8cb002e13b610afa8923053ba67303dcb8b25e2d97a. Installed CLI checks unchanged hello/src/main.tsr;owner source/config hashes unchanged. Installed LSP next()/literal-key hover protocol checks pass. All current build/test/profile/timing/site/install jobs terminal. No full regressions/CI/push/publication. Local performance page updated;live deployment remains deferred to the large-gap milestone.

Source checkpoint `06ac9edd` contains the accepted close repair,SQLite histories and performance selection. Pruned90 closed raw/profile/log artifacts (15960312bytes) after independent ingestion verification and commit. Shared Cargo target,current consumers and exact serde/Nix argument files remain reusable.

Next profile source-retaining schema (1.653458/Rust1.212250ms,.441208excess),schema collections(1.691500/1.308709,.382791),normalization(.340042excess),then smallerJSONarray(.191917),ownedarray(.154334),callspread(.153792),objectretain(.108500),radix(.081042) and literalJSON(.062875). Retain rejected schema CPU/map/retained visitor hypotheses below;do not repeat without new evidence. Preserve object/string allocation plan and Rust default semantics. Broad regressions and HTTP affected-metric refresh remain pending the final milestone.

The JSON optimization was staged in8072665c and accepted in06ac9edd (pure JSON45checks and IO acceptance/ingestion/install completed above). Found pre-existing generated std:fs unsafe FFI through the JSONL -Funsafe_code control. Preserve single-close/error semantics while removing both production FFI and embedded test FFI from generated modules.

Implemented fs/runtime/fs.rs close_fd via safe nix::unistd::close→std::io::Error;into_raw_fd transfer/no-retry/error mapping unchanged. native/runtime.rs files_module strips each trailing cfg(test) module (resource/fs/buffered_fs),and separate marks file support as dependency-backed. Optional files feature and pinned nix=0.31.2 added to backend/native dependency manifests;runtime_build/package_export select it only when file module exists. Root targetUnix dev dependency supports backend witnesses;test support rustc resolves its installed Cargo artifact like existing Tokio. Benchmark dependency preparation now detects plain files as well as JSON and selects exact nix artifact only when needed;extended real offline/link test. Existing std_fs witness asserts emitted code contains no unsafe and uses dependency-backed runtime;new portable file test relocates,vendors Nix,uses an empty Cargo home,adds forbid(unsafe_code) to generated application and runs O0/O3 with shared Cargo target.

Pinned0.31.3 was initially tried but conflicts with existing exactlibc0.2.183 (requires>=0.2.186). Chose0.31.2 (requires>=0.2.181) and verified its downloaded primary source safe close<Fd:IntoRawFd>. Fetch adds exactly1new package to each root/backend/native lock;no existing package versions change. Root cargo fetch completes,backend/native lock updates offline complete. No unsafe second wrapper,no error suppression,no dependency upgrade beyond required capability. Nix is optional/default-features=false in runtime packs;FS remains supported on macOS/Linux as before (no Windows FS acceptance claim). Prior schema alternatives remain rejected;typedJSON implementation unchanged.

Joint no-run build passes268.476s/2,612,642KiB (new Nix/libc feature context rebuild included),toolsession45543terminal0. Normal offline setup completes15.579s/147,792KiB;new verified portable pack291032a97fc5afc77ca4df52538a5e56aad7769646c79847a67ae33aa8b3bc2f in sharedCargo cache. No project config/source changes. First std_fs witness failed because added assertion called single-file check on imports;repaired to project load/check_graph only,relink3.630s/378,880KiB. Backend tests had already passed and were preserved.

All35affected checks now pass,none ignored,serial3threads/5GB30min:resource lifecycle3 (3.620s/269,504KiB,includes O0/O3 actual FD release),buffered backend1 (1.045s),std_fs3 (37.241s/381,328KiB),buffered files2 (7.765s/360,128KiB),runtime separation1 (8.879s/227,888KiB),new offline relocated file package1 (34.714s/377,536KiB,forbid unsafe in generated app,O0/O3,emptyCargoHome),dependency bundle2 (24.337s/422,880KiB),portable sources2 (19.659s/151,424KiB),JSONL6 (25.410s/1,070,496KiB),borrowedJSONL5 (16.606s/1,124,928KiB),JSONLwriter5 (17.072s/1,070,432KiB),borrowedwriter4 (8.778s/1,075,360KiB). Unrelated HTTP/TLS/SQLite package cases not rerun. Prior45pureJSON/editor checks unchanged. DebugCLI SHA matches original canonical pureJSON3 emitted bodies exactly;reuse their93samples/timestamp rather than retiming. Real offline benchmarkdependency Node test passed5/5 as recorded below,including file/noJSON Nix-only artifact link with-Funsafe_code. All otherjobs terminal. Root/backend/native lockfiles add exactlyNix0.31.2 and no version upgrades.

Benchmark dependency integration passes5/5,none skipped,92.173s/408,784KiB (real offline JSON and Nix-only file export/link case). Exact serde/serde_json/nix arguments captured from Cargo artifact messages in/tmp/tsrust-safe-close-native-args.json;no guessed cache hashes. Initial preparer substring check mistook the raw-value documentation comment “No unsafe cast” for code;no product failure. Candidate compilation enforces the actual Rust unsafe lint and passes.

Matched JSONL controls are terminal0. Historical baseline emitted by verified installed oldCLI210bd28d...reproduces old FFI;only it compiles without-F. Newdebug candidate MUST/does compile with-Funsafe_code,with exact newNix dependency args. Original2000iterations/5oracleidentities/8warmups/3×31samples/two rotated rounds/3warm compiles/same paths+argv/atomic swaps. Typed parent20.467959→20.348042/20.592541→20.357250ms (1.006×/1.012×);borrowed21.133916→20.863041/21.076458→20.776208 (1.013×/1.014×). No runtime regression. Warm compile typed444.854→451.837ms (+6.983ms,~1.57%,overlapping441.660–453.964/448.024–456.289ranges);borrowed452.637→455.234 (+2.597ms,~.57%,overlapping450.114–458.468/453.051–455.663). Do not claim compiler speedup;small extra metadata cost accompanies the required safe-close capability,not a reason to restore unsafe or suppress errors. Three identical allocation samples each:20,000→8,000blocks,16,980,000→16,916,000requested bytes,0reallocations/retained/cleanup,checksum168007;fileinput fixture/argv fixed. Maxload3.508<4,typedwatch47.180s/320,032KiB includesquietwait,borrowed16.035s/319,808KiB.

Final canonical refresh,ingestion,paired installation,performance selection and cleanup are complete above;all old raw result files and section logs were pruned after independent verification. Cached Node/Rust execution references were not repeated.

**Typed JSON keys/literal storage — 45 checks/pure-JSON A/B pass; JSONL unsafe-close defect must be fixed before final acceptance; parentd4d355ef.**
CPU profiling of original retained-schema success,missing-field and record-JSON cases is terminal0 (28.922s/378,128KiB):3s samples of8s direct-work loops,original2000iterations/seed7,independent8007/2007/4007checksums,no result observer or allocator instrumentation,unsafe forbidden. Initial wrapper failed before profiling because it assumed function0;reused assertABI to select the checked workload correctly. Retained success stacks show dynamic retained-value decoding,source RawValue scans and allocation/free;missing/record work is allocation-heavy. mach_absolute_time samples are beneath macOS allocator free,not a new Tsrust clock/deadline check. Native semantics/default limits remain unchanged.

Two schema candidates are not selected. Generated-only record output Vec→final Map directly removes2,000blocks/256,000requested bytes (62,000→60,000;5,722,000→5,466,000,peak649,checksum16007,cleanup0). Full collections runtime1.710000→1.680792/1.712125→1.684750ms (~1.7%);compile median1054.063→1082.023ms with wide1050–1155candidate variation. Too small a runtime benefit with unresolved compile cost to keep now;no product edit. Generated-only callback-free retained-object direct Serde visitor preserves sorted extras and full diagnostic fallback:runtime1.677542→1.504375/1.674166→1.489750 (~1.12×),but compile723.295→777.590ms,requested5,280,000→6,762,000,peak1523→1635,blocks36,000 unchanged. Extra speculative work on missing fields regresses memory/compile cost;reject. Existing retained/collection gaps remain open. Original5identities/8warmups/3×31samples/two rotated rounds/3warmcompiles/samepaths+argv+atomic swaps pass;no Node/Rust execution retiming or ingestion. Scripts/results /tmp/tsrust-schema-{cpu,map,retain-direct}* are closed diagnostics,not implementation.

Profile next ranked native_json_literal_union (2000iterations):native/Rust10,000/2,000allocations,0reallocations,52,000/64,000requested bytes,peak11/32,input47,checksum84007,cleanup0;three identical samples each,watchdog3.626s/302,032KiB. Native allocates for three property names per record (6,000blocks) plus literal payload and String() copy (4,000);Rust uses enum tags and a Vec. Reuse existing source::Text serde adapter for transient keys/literal matching,and store successfully matched literal payloads as existing static program literals rather than source borrows.

Four-way generated-only original-protocol probe passes:parent/keys/literal/both runtime.465333/.308416/.350333/.285584ms and reverse round.393416/.305750/.326000/.251666;compile medians468.600/476.420/470.661/468.681ms. Combined1.629×/1.563×,flatcompile;allocations10,000/4,000/6,000/0,requested52,000/20,000/32,000/0,all0reallocations/retained/cleanup,checksum84007. Maxload2.143<4,unsafe forbidden for production-like A/B. Keys-only removes6,000blocks;literals remove4,000. Select both as one coherent typed-decoding section,not the rejected schema alternatives. The cached Rust reference.151584ms remains faster;required integer-token accuracy/unknown-input validation stays in place.

Implementation changes emit/json_typed.rs ordinary record-key visitor to existing source::Text;emit/json_literals.rs uses Text for string-literal matching and returns String::Borrowed of the exact declared static literal (single,union,nullable/optional cases retain their existing discriminants). Numeric/bool literals unchanged. Public strings remain independent of input;escaped key/value scratch storage is not retained. Added json_literal_decoding.rs O0/O3 CLI witness for escaped keys/literals,Unicode+NUL,single/union/array/tuple values,source replacement/drop,valid and invalid duplicates,unknown keys and invalid surrogate keys. Existing JSONC/JSONL/default-limit/error tests remain required. No API/default/signature/dependency/unsafe change.

Selected joint compiler/LSP no-run build passes213.125s/2,581,186KiB. All45affected checks pass,none ignored,serial3threads/5GB30min:json_literal_decoding3 (18.626s/734,656KiB,includes O0/O3 witness);json_typed8 (28.996s/1,189,648KiB);json_parse_plain3 (7.252s/723,232KiB);json_views5 (27.378s/1,076,464KiB);jsonl6 (21.806s/1,079,088KiB);jsonl_borrowed5 (14.513s/1,088,464KiB);json_namespace9 (54.371s/1,094,320KiB);json_default_limits2 (12.947s/753,040KiB);json_dependency_selection1 (.522s);LSP JSON namespace/typed decoding3 (.516/.522s). No tests failed.

Actual compiler output A/B againstd4d355ef passes,original protocol/five oracle identities/unsafe ban/same paths+argv+atomic swaps. Parent→current ms:literal.455792→.249083/.398167→.264084 (1.830×/1.508×),compile471.199→468.478;ordinary typed.385125→.294125/.368375→.271250 (1.309×/1.358×),compile472.289→472.507;JSONC.673209→.565250/.685167→.543250 (1.191×/1.261×),compile483.451→482.178;escaped literal/key.534792→.404459/.499750→.386209 (1.322×/1.294×),compile501.212→490.838;Unicode ordinary record.373167→.259666/.364500→.262417 (1.437×/1.389×),compile462.175→461.142. Initial Unicode diagnostic oracle wrongly used JS UTF-16 length;parent/current both returned the documented UTF-8 byte count. Corrected only the diagnostic oracle to Buffer.byteLength;rerun passes. No product bug or semantic change. Max accepted load2.496<4. Watchdogs literal7.761s/319,872KiB,typed5.695s/308,192KiB,JSONC6.221s/341,424KiB,escaped6.232s/374,848KiB,Unicode6.215s/370,016KiB. Three identical allocation samples/version:literal10,000→0/52,000→0requested bytes/peak11→0;typed8,000→2,000/42,000→10,000/11→5;JSONC10,000→4,000/160,000→128,000/70→64;escaped12,000→6,000/68,000→34,000/19→17;Unicode8,000→2,000/44,000→12,000/12→6. Reallocation/retained/cleanup0 throughout. Diagnostic input variants preserve original iterations/observation/seed identities and are not new public measurements.

Pure JSON A/B has no unresolved regression. Confirmed emitted changes for native_json_literal_union,native_json_typed,native_jsonc_typed,native_jsonl_typed,native_jsonl_borrowed. Canonical five-workload native-only run is terminal0,443.084s/2,184,348KiB,including quiet wait;run2026-11-04T09:19:59.562Z,maxacceptedload3.270508<4,93samples each. Medians:literal.214459ms,typed.248500,JSONC.522709,JSONLtyped20.229916,JSONLborrowed20.252750. Raw benchmark file benchmarks/runtime/results/2026-11-04T09-19-59-562Z.json is NOT INGESTED/SELECTED YET. Cached Node/Rust execution references unchanged. Preserve completed pure JSON measurements unless emissions change.

Additional matched JSONL control attempted under unsafe_code forbidden,using the verified previous-checkpoint installedCLI210bd28df4c816a118f4be7968df1bc935be018fcbdb1e29011dd41a99231b9d as old emitter (releaseCLI is now newer from canonical build). It fails before any timing:existing emitted std:fs close_fd contains unsafe extern C plus unsafe native_close call in src/stdlib/fs/runtime/fs.rs154–158. Current typed-key/literal change adds no unsafe,but this existing defect violates owner rule and must be fixed. Do not remove the candidate unsafe ban or weaken close-error handling. /tmp/tsrust-json-literal-io-probe.mjs uses nativeFileInput fixture helper and originaloracle/sampling protocol;its first run is terminal1,2.067s/235,920KiB. Only historical baseline may need its existing FFI to reproduce prior behavior;new candidate must compile with -F unsafe_code.

Rust1.96.1 source is installed under ~/.rustup/toolchains/1.96.1-aarch64-apple-darwin/lib/rustlib/src/rust/library. std::fs::File has no safe consuming error-reporting close;Drop ignores errors,so do not substitute drop/sync_all and lose late close failures. Installed rustix1.1.5 io::try_close is unsafe RawFd and does not solve generated unsafe. Nix0.31.3 primary source verifies safe unistd::close<Fd:IntoRawFd> returning Result (https://docs.rs/crate/nix/0.31.3/source/src/unistd.rs,close function). It is not currently installed or locked. Smallest behavior-preserving candidate is target-Unix/default-features=false pinned nix for the necessary safe OS close capability,then close_fd uses nix::unistd::close(descriptor).map_err(std::io::Error::from),retaining into_raw_fd ownership transfer and no retry. This is a dependency for missing safe OS capability,not a convenience helper;standard library cannot meet both no-generated-unsafe and existing close-error semantics. No FS/dependency implementation yet. Before risky FS edits checkpoint current typed-JSON work and preserve partial acceptance;then update native dependency/package/export/bundle selection and backend tests coherently if selecting that repair. No owner question pending;do not silently weaken errors or add a second unsafe wrapper. Existing FS runtime supports macOS/Linux only;do not claim Windows FS acceptance.

Current tree changes only emit/json_typed.rs,emit/json_literals.rs,new json_literal_decoding.rs witness and this note. The installed CLI/LSP are still previous accepted radix binaries;target debug/release contain validated JSON optimization. NO source checkpoint/ingestion/public selection/install for this section yet. Current /tmp/tsrust-json-literal-* includes profile,alltestlogs,actual A/B files,pending IO script and pre-ingestion history hashes /tmp/tsrust-json-literal-store-before.json. All jobs terminal;no live build/test/benchmark processes. Need resolve FS defect,run affected FS/JSONL suites and matched old/candidate IO (candidate unsafe banned),retain unaffected45/pure-JSONAB evidence. New safeFS emissions will require rerunning only affected JSONL canonical rows;reuse unchanged pureJSON samples. Then verify exact histories/medians/mean rates/counts/source hashes with store:ingest,update public page,formatter/install/checkpoint/cleanup. Typed HTTP parsing may also benefit;retain its old measured rows until final affected HTTP refresh. Full regressions/CI/promotion deferred until large gaps reach nits.

**Radix formatting — accepted locally, measured and installed; parent98c9de2a.**
Original2048iteration number_to_string_radix native/Rust allocation profile:25,583/14,335 blocks,1,718/1,286 reallocations,849,981/430,074 requested bytes,361,725/387,050 retained output bytes,361,749/387,058 peak;input19,checksum410002210,cleanup0,three identical samples. Watchdog3.116s/271,008KiB. Native retains less output but spends on temporary buffers;do not assume all Rust allocation differences are removable. Split five original conversions with independently filtered complete-output oracles,three identical samples/engine/case,cleanup0 (5.191s/269,664KiB):decimal native/Rust2,047/4,097 blocks,9/0reallocations;binary5,885/4,097,1,718/1,286;hex5,885/4,097,9/0;base365,885/6,143,9/0;octal5,885/4,097,9/0. Decimal is already allocation-efficient;nondecimal temporary whole/fraction buffers are the target.

Generated-only integer stack buffer removes8,184blocks/1,709reallocations;runtime.788792→.665083/.782084→.660000ms (1.186×/1.185×),compile314.747→319.486ms. Adding fraction stack buffer removes all15,352temporaryblocks;three-way runtime parent.787792/.795209,whole.653083/.651083,both.610250/.609000;compile314.303/321.719/317.276. Final three-way probe adds exact u64 digit arithmetic only through2^53,preserving existing f64 large-value fallback:parent.811833/.804833,buffers.614250/.612625,fast.473792/.472333ms (1.713×/1.704×versus parent);compile314.068/316.734/316.705ms. Buffers/fast allocation count10,231,reallocations9 (outer result growth only),requested607,005,retained361,725,peak361,725,zero cleanup. Original8warmups/3×31samples/two rotated rounds,3warm rotated compiles,same paths/argv/atomic swaps,all five independent complete-output identities,unsafe forbidden in production-like probes. Max final load1.989<4;final probe terminal0. Cached Node/Rust execution timings unchanged;these probes are not ingested.

Before implementation,unchanged versus candidate O0/O3 compares584,465complete outputs each:allradices2..36,all binary exponents−1074..1023 and adjacent bit patterns with both signs,4,096deterministic random f64 bit patterns,signed zero,subnormal/normal boundaries,2^53 boundaries,maximum finite and special values. Both pass6.249s/139,280KiB;unsafe_code forbidden. The digit capacities derive from f64::MAX_EXP (1024whole) and MANTISSA_DIGITS−MIN_EXP (1074fraction);no caller quota/default change. Each fractional step at least doubles delta from the minimum subnormal;whole digits cannot exceed the binary exponent range.

Implemented only runtime/strings.rs tsr_number_to_string:ASCII digit arrays,existing rounding/carry and large-value fallback,exact-range integer division. Added number_to_string.rs O0/O3 witness:all binary powers/signs,known maximum-finite binary digits,integer round trips in every nondecimal radix,nonfinite/signed-zero spelling,extreme radix storage/alphabet. Existing400reference random cases andradix fixture remain. No API/signature/docs/dependency/unsafe change;compiler caller sites inspected including ordinary method,templates and retained UI decimal-only callers (unchanged decimal branch). Affected joint no-run build passes210.996s/2,578,402KiB and refreshes both debug consumers. All22checks pass,none ignored,serial3threads/5GB30min:compiler number_to_string4 (3.118s/491,136KiB),number_formatting5 (7.243s/385,200KiB),numbers7 (3.176s/562,112KiB),number_locale_string3 (1.045s/291,664KiB);LSP number_to_string/number_formatting_discovery/integer_formatting_discovery3. Existing reference fixtures,range errors and documentation remain valid.

Actual compiler output A/B versus98c9de2a passes:radix.779708→.455291/.783459→.460333ms (1.713×/1.702×),compile314.528→316.910ms;precision-default.081667→.078125/.078333→.075500,compile347.207→348.272;template-ASCII.249083→.250792/.245583→.243458,compile390.049→392.224. Initial fixed-default.083625→.087125/.079584→.081667 showed a few microseconds slower;one bounded7compile confirmation resolves it:.086708→.086541/.083542→.083333,compile341.941→342.805ms (+.25%,overlapping339.523–343.925/338.926–346.290ranges). Do not restart broad passes for that resolved noise. Same original protocol/five independent exact output identities/paths/argv/atomic swaps/unsafe ban;max initial accepted load2.574<4. Primary allocations25,583→10,231 (-60%),reallocations1,718→9,requested849,981→607,005;retained361,725/cleanup0 unchanged;control allocation counts unchanged. No unresolved material regression. Canonical native-only primary-workload release/measurement passes222.395s/2,201,804KiB;run2026-11-04T08:31:07.145Z,93samples,all independent correctness cases pass,max accepted load2.854004<4,no override. Median.508625ms/4,106,624.975mean ops/s;this public run improves over saved.807541ms,while matched A/B above demonstrates1.70–1.71×. The saved hand-written Rust reference is.427583ms:remaining.081042ms/18.95% overhead,not yet parity,now lower priority than larger absolute gaps. `npm run store:ingest` adds exactly1native history;independent93sample median/mean rate/operations/timestamp recomputation matches,all other histories unchanged,SQLite integrity OK. Unchanged decimal/precision/template controls retain previous public histories;Node/Rust execution timings were not repeated. Final debug/release emissions match on all4A/B workloads,including canonical measured SHA256. Public selection102accepted runs/933comparisons;both page tests and67-page build pass3.657s/1,158,398KiB. No before/after table.

Paired release consumers installed atomically and SHA256-verified:CLI210bd28df4c816a118f4be7968df1bc935be018fcbdb1e29011dd41a99231b9d;LSP913e70c8ba8c1860dbc98c784630350d53810ca990d9e21e635986a2974cb619. Installed CLI checks unchanged hello source;owner source/config hashes unchanged,editor processes untouched. Whole-project formatter passes0files/snippets/fingerprints;source checkpointfa51189d is accepted. No other implementation/API/default/ownership/dependency/unsafe change. All profiling/build/test/measurement/site jobs terminal. After verified ingestion and commit,pruned239closed scratch/raw/native and obsolete schema/project test artifacts (174,478,046bytes). Shared Cargo libraries,current numeric tests,paired CLI/LSP binaries and cached schema dependency args retained. Prior schema/project test binaries will need relinking for future affected or milestone runs.

Next:profile remaining retained-schema success/missing-field CPU work and collection record-JSON decoding before another implementation. Rank fresh excess over cached Rust:source retain.441208ms;collections.382791;normalization.340042;literal JSON union.252749;JSON-array.191917;owned-array.154334;call spread.153792;object-retain.108500;radix.081042. Preserve the earlier error-allocation and ordinary-container results;do not reopen rejected static/Cow path variants or capped reservations without a new measured hypothesis. No need to rerun unchanged Node/Rust references. Remaining object lifecycle plans and default-limit invariants stay in force. Full regressions/CI/main promotion remain deferred until all large gaps reach nits;the performance page is updated locally,not deployed. After the milestone,promote approvedv0.1.1 then reprioritize with owner while CI/CD runs.

**Retained-schema error paths — accepted locally, measured and installed; parent `f9dc567c`.**
Source-retaining schema remains1.678792/native versus1.212250/saved Rust (.466542ms excess). Split the original success and missing-field cases,keeping all original native/Rust checks and independent checksums. Three identical samples/case,cleanup0:success native/Rust both22,000 allocations;requested3,318,000/4,358,000 bytes,peak1,523/2,051,checksum8007,input52. Missing-field native14,000/Rust10,000 allocations;requested2,026,000/2,010,000,peak845/893,checksum2007,input52. No reallocations/retained bytes. Split profile4.664s/311,792KiB. Extra allocations are on error construction,not successful retained payloads.

Generated-only four-way static-name/Cow path experiment is rejected. Parent1.683125/1.692458ms,static1.675833/1.645833,forced inline error bridge1.712583/1.665208,both1.642125/1.634750. Compile medians728.896/736.628/732.916/733.502ms. Static names remove2,000 blocks but enlarge path enum storage:requested5,344,000→5,462,000,peak1,523→1,555. Forced inlining alone has no consistent benefit or allocation change. Do not introduce extra segment variants,Cow path labels,interning/tag encodings or forced inlining from this probe. Original protocol/five identities pass,max load2.728,watchdog19.748s/349,120KiB;diagnostic only,not ingested.

Smaller standard-library alternative:store immutable internal OwnedSegment::Field labels as Box<str>,keep their existing owned-copy behavior,and convert the box into the public owned String without copying. No incoming Segment change or API/default/ownership/unsafe/dependency change. Four-way parent/boxed/small initial path reservation/both source-retain runtime1.753542/1.721500,1.669875/1.627750,1.702584/1.679500,1.664500/1.686375ms. Compile medians727.508/727.444/729.171/723.154ms. All36,000 allocation counts unchanged;requested5,344,000/5,280,000/5,200,000/5,184,000,peak1,523,input52,checksum10007,cleanup0,three identical samples. Box-only runtime1.050×/1.058× with flat compile is simplest and steadier;extra reservation tuning adds no consistent runtime benefit and is not selected. Original2000 iterations/seed7/input/8warmups/3×31samples/two rotated rounds,five identities,same paths/argv/atomic swaps,unsafe_code forbidden,max load2.436,watchdog19.733s/331,312KiB.

Collection/index-path control after lazy-buffer checkpoint:parent1.817083/1.748416ms,boxed1.744250/1.756583,capped1.738125/1.764333,both1.766458/1.787166. Compile medians1050.534/1058.565/1054.663/1051.179ms. All62,000 blocks unchanged;requested5,850,000/5,722,000/5,562,000/5,530,000,peak649,checksum16007,input7,cleanup0,three identical samples. Box control second round~0.47% slower within observed variation;native compile~8ms higher needs actual-compiler confirmation before acceptance. Generated-only probe at that stage;not a public measurement. Control26.240s/382,048KiB,max load2.124;all five identities/original protocol pass. Cached Node/Rust execution references remain unchanged.

Implemented only boxed labels:runtime/issues.rs OwnedSegment Field(String)→Field(Box<str>),append_relative name.as_str→name.as_ref,and truncated field copy→boxed owned slice;compiler/emit.rs error bridge converts name.into() into the existing public String::Owned. Reuse schema_trial_storage.rs’s O0/O3 allocator/unit harness for Unicode/byte truncation,dynamic input drop/rebase and zero-allocation pointer-preserving Box<str>→String conversion;existing issues/union/trial unit modules must still pass. Affected schema error,collection,pipeline,union/editor fixtures and actual-compiler A/B pass below;canonical measurements are accepted. Do not select capacity tuning merely for fewer requested bytes if controls do not improve.

Affected joint compiler/LSP no-run build passes212.042s/2,612,066KiB. All92 selected checks pass,none ignored,serial binaries/3threads under5GB30min watchdog:storage1 (2.588s/153,440KiB,includes O0/O3 normal allocator plus issues/union/trial unit runs);decode10 (30.560s/1,145,920KiB);diagnostic options11 (21.790s/997,056KiB);super-refine5 (7.754s/724,336KiB);parse5 (2.084s/507,520KiB);pipelines20 (49.693s/1,066,736KiB);collections5 (8.813s/1,051,232KiB);unions13 (56.957s/1,079,376KiB);schema_editing17 and project pipeline/error/direct-union/next-hover5. The next/object-key protocol witness passes again;actual editor rendering remains unverified. No tests failed. Debug CLI link enables the actual-emission matched A/B below.

Actual-emission A/B passes against parent f9dc567c (old verified release CLI versus current debug CLI;debug build204.396s/2,610,226KiB). Original protocol/five independent oracle identities,3warm rotated native compiles,8warmups/3×31samples/two reverse-order rounds,same paths/argv,atomic swaps,unsafe_code forbidden. Runtime parent→boxed milliseconds:source-retain1.802167→1.733917/1.753791→1.752375 (1.039×/1.001×);collections1.864750→1.780625/1.779375→1.787458 (1.047×/0.995×,second -0.45% within variation);object-retain.845792→.816709/.837542→.822041 (1.036×/1.019×);JSON-array.688333→.654708/.678875→.658792 (1.051×/1.030×);owned-array.444000→.427000/.437000→.434125 (1.040×/1.007×). Compile medians respectively730.423→724.070,1078.463→1065.014,624.739→620.735,529.418→521.021,391.944→393.284ms (last +.34%,small within variance). No unresolved material regression;do not claim uniform gain. Max accepted load3.578<4;collections waited45s for quiet load rather than overriding. Watchdog source10.462s/370,624KiB,collections57.797s/375,184KiB,object7.782s/363,104KiB,JSON-array6.720s/304,640KiB,owned-array5.171s/350,720KiB. Three identical allocation samples/version:counts unchanged,requested bytes source5,344,000→5,280,000;collections5,850,000→5,722,000;object4,248,000→4,184,000;JSON-array2,336,000→2,208,000;owned-array1,824,000→1,696,000. Peak bytes source1523/collections649 unchanged,object1026→994,arrays776→712;zero reallocations/retained/cleanup bytes. Whole-project formatter passes0files/snippets/fingerprints. Canonical native-only five-workload refresh passes below;Node/Rust references unchanged.

Canonical native-only release/measurement passes397.391s/2,242,556KiB,including a required quiet-machine wait after preparation. Run2026-11-04T08:05:46.758Z,max accepted load3.860840<4,no override;93samples/workload,all independent correctness cases pass. Medians/mean ops per second:source-retain1.653458ms/2,413,218.598;collections1.691500/5,884,829.553;object-retain.737375/5,361,819.534;JSON-array.664542/5,907,731.084;owned-array.404959/8,402,905.657. Historical JSON-array median.655958 is slightly lower than this fresh run;matched A/B above improves both rounds,so do not infer a regression from unmatched historical medians. `npm run store:ingest` adds exactly5native histories;independently recomputed93sample medians/mean rates/operations/timestamp match,all other histories (including Node/Rust controls) unchanged,SQLite integrity OK. Final debug/release emitted SHA256 matches each canonical measured source. Public selection now101accepted runs/933comparisons;two performance-page tests and67-page build pass (3.635s/1,191,966KiB). No before/after table added.

Paired release consumers atomically installed and SHA256-verified:CLI0ab72eb7f3b1394f4ce578d9cabc466af381cd2b9511b04e8deefaebb69a2c37;LSPbd9ed62bc3d810478821ffb7ca5e3c27cc9f0f9afe72c1183c7067ed13bdde60. Installed CLI checks unchanged hello source;source/config hashes unchanged,editor processes untouched. Whole-project formatter passes0files/snippets/fingerprints. Product changes remain only boxed internal labels,bridge conversion and the reused witness;no API/default/ownership/unsafe/dependency change. Source checkpoint052fc838 is accepted. All schema profiling/build/test/measurement jobs are terminal;pruned100closed scratch/raw/native artifacts (21,304,626bytes) after ingestion verification and commit. Shared Cargo target,cached schema dependency args and active radix profile remain.

Continue closing large gaps rather than declaring the family complete. Fresh remaining excess over saved Rust:source-retain1.653458−1.212250=.441208ms;collections1.691500−1.308709=.382791;radix formatting.807541−.427583=.379958;normalization1.976042−1.636000=.340042;literal JSON union.404333−.151584=.252749. Ordinary Map/Set/index/string-join gaps already closed above;do not reopen stale rows. Next bounded profile:number_to_string_radix (original2048iterations) against the existing independent Rust reference,allocation counts/retained output/temporary buffers before a generated-only probe. Source review finds fraction String,integer Vec<char>,then final String in tsr_number_to_string;Rust reference uses standard binary/octal/hex formatting and explicit base36 conversion over the matched exact-eighth corpus. Do not specialize correctness to that corpus:retain allradix2..36/special/f64/extreme rounding semantics and defaults. Prepared /tmp/tsrust-radix-profile.mjs is next;profile/probe a shared-runtime improvement only if evidence supports it. Preserve prior numeric/object lifecycle plans. Full suites/CI/promotion remain deferred until all large gaps reach nits.

**Lazy schema rollback storage — accepted locally and installed; source parent `057a624e`.**
The accepted refresh ranks schema collections1.816334/native versus1.308709/saved Rust (.507625ms excess),and source-retaining schema1.678792/1.212250 (.466542ms). Initial matched allocation profile is terminal0 (5.321s/389,504KiB),three identical samples per engine/workload,original iterations/input,independent checksum/zero-cleanup checks. Collections:native66,000 allocations/0 reallocations/6,042,000 requested bytes/0 retained/649 peak;Rust48,000/0/2,422,000/0/709. Both checksum16007,input7,cleanup0. Source retain:native36,000/0/5,344,000/0/1,523;Rust32,000/0/6,368,000/0/2,304. Both checksum10007,input52,cleanup0. No engine execution timing repeated;cached Node/Rust references unchanged. Instrumentation is external,not generated unsafe.

Source comparison:hand-written collection validators consume HashMap/HashSet inputs and move strings;Tsrust safeParse retains its checked ownership contract. Preserve input reuse/independent outputs;do not weaken ownership to remove copies or assume the entire difference is removable. Native source-retain already requests fewer bytes and has a lower peak than Rust,so its gap may be validation/scanning work rather than allocation alone. Split five operations with original native/Rust checks and independently known checksums;all three counter samples/case agree,cleanup0. At original2000 iterations,allocations/requested bytes/peak native versus Rust:record JSON10,000/1,554,000/649 versus4,000/282,000/141;plain Map4,000/384,000/192 versus6,000/562,000/281;plain Set4,000/320,000/160 versus6,000/442,000/221;Map collision24,000/2,004,000/552 versus16,000/632,000/314;Set collision24,000/1,780,000/552 versus16,000/504,000/250. No reallocations or retained bytes. Split profile6.734s/320,816KiB. Ordinary Map/Set validation already allocates less than Rust;focus transformed collision paths (+16,000 blocks) and record JSON (+6,000),not a generic container rewrite.

Bounded3s CPU sample over an8s direct-work loop is terminal0 (11.894s/395,472KiB). No result observer/allocation instrumentation in sampled loop;compiled with unsafe_code forbidden. Startup dyld samples are present;use workload stacks only. Workload stacks show trial::sequence,transform callbacks,to_lowercase,malloc/free and record JSON deserialization. The sequence helper unconditionally reserves restores Vec before it knows whether callbacks commit. Public error messages already use static borrowed code text when no custom message;do not add Cow/message rewrites based on the earlier hypothesis.

Generated-only lazy-restoration probe is terminal0 (14.260s/386,176KiB),original2000/seed7/input/8warmups/3×31samples/two rotated rounds,five checksums/identities,same paths/argv/atomic swaps,max load3.294,unsafe_code forbidden for production-like code. Reserve output as before;start restores with Vec::new and reserve_exact(input_len) only on first Prepared/noncommitted restoration. Parent1.794125→1.723208ms/1.809959→1.733959ms (1.041×/1.044×);warm native compile1056.673→1056.200ms with overlapping1052.743–1083.319/1055.045–1059.737ranges. Three identical allocation samples66,000→62,000 blocks,6,042,000→5,850,000 requested bytes (-192,000);reallocations0,retained0,peak649,input7,checksum16007,cleanup0 unchanged. Prototype only;not ingested or product code.

Implementation now changes only runtime/trial.rs:reserve output as before,start restores empty,and reserve its original full input length on the first Prepared/noncommitted value. Added schema_trial_storage.rs:normal O0/O3 allocator/drop witnesses for committed,prepared/restored,rejected-first,empty and zero-sized states at counts0/1/4/5/17;noncloneable values exercise partial rejection,commitment,aborted callbacks and panic cleanup. The same source also compiles/runs the existing issues/union/trial unit modules at O0/O3 with3testthreads,using the exact crate module paths;unit mode forbids unsafe and excludes external allocation instrumentation. All three sequence emitter consumers inspected (collection_trials,pipeline_array_trials,Set branch of collection_trials);no signature,API,docs,defaults or ownership-contract change. Joint selected no-run build passes208.460s/2,635,538KiB;witness-only relink2.071s/394,512KiB. All60 affected checks pass,none ignored,serial3threads/5GB30min:storage1 (2.082s/137,200KiB),compiler unions13 (57.467s/1,080,640KiB),pipelines20 (49.735s/1,001,072KiB),collections5 (8.817s/1,019,600KiB),editor schema_editing17 (.523s) and project pipeline/default/error/direct-union4. Storage witness also executes existing issues/union/trial unit modules at O0/O3. Coherent implementation and semantic acceptance are complete;performance acceptance remains pending. Continue collection/record JSON profiling after acceptance;4% prototype does not close the family gap. Actual-compiler A/B collections1.862167→1.754500ms/1.800875→1.800250ms;warm native compile1055.577→1070.260ms. Allocation savings match prototype,including62,000 blocks/5,850,000 requested bytes. First runtime round wins,second is flat;compile cost requires confirmation. Diagnostic readonly object/array control has a real sequence call and unchanged34,000allocations/4,836,000requested bytes;runtime1.391250→1.394167/1.387000→1.386083ms,compile599.675→596.601ms. Readonly union-fallback control requires preserved stripped extra fields and has a real sequence call;unchanged44,000allocations/2,000reallocations/6,868,000requested bytes,runtime2.202833→2.156875/2.164417→2.148375ms,compile775.908→761.364ms. Native pipeline control differs only in unused sequence body;unchanged20,000allocations/1,916,000requested bytes,runtime .700667→.686667/.673041→.687583ms,compile682.458→683.884ms. Five independent identities and original2×3×31sample protocols pass,same paths/argv/atomic swaps,unsafe_code forbidden. Do not accept based on allocation counts alone.

Seven-warm-compile confirmation is terminal0 (22.628s/410,560KiB),max load2.104. Runtime1.920584→1.825333ms/1.848667→1.841375ms (1.052×/1.004×). Compile medians1057.860→1059.663ms (+1.803ms,~0.17%) with broad overlapping1044.771–1072.832/1032.222–1070.779ranges;the first~15ms cost is not stable. All allocation samples remain exact. Accept the small,variable positive runtime signal and deterministic4,000-block/192,000-byte savings without claiming a uniform4% speedup or compiler speedup. Controls show no material regression. No source/API/default/ownership/unsafe change beyond the lazy buffer. Canonical native-only run2026-11-04T06:55:22.082Z passes all independent cases and93samples:1.737083ms/5,498,517.971mean workload operations/s (10,000 operations/sample). Watchdog313.753s/2,185,468KiB includes release build and quiet waits;max accepted load3.733<4,no override. npm run store:ingest adds exactly one native history40→41;independently recomputed median,mean rate,operation count,sample count/date and unchanged Node/Rust histories match;SQLite integrity OK. Median improves from1.816334ms,but mean rate is essentially flat versus5,504,515.922;do not claim uniform throughput improvement. Saved Rust1.308709ms remains unchanged;native is now1.327× that reference,so the large gap is not closed.

Final release/debug emissions match for collections,readonly and readonly-rollback diagnostics,native pipeline,owned array and source-retaining schema. Canonical emitted hash matches stdout.trim. Performance selection100 accepted runs/933comparisons;two page tests and67-page build pass (3.639s/909,708KiB). Paired release consumers atomically installed and SHA256-verified:CLI f97a9ad02adce3173ac64f70a6873bb5d053a1c5e3df89802e7a0ae83e3028fb;LSP5693f18e10a8e3be0cd09ec76c976e09a1c5223cee7a326cfaacff127a4fe70b. Installed CLI checks unchanged hello/src/main.tsr (.521s;sampled80KiB too short for memory claims);owner source/config/editor processes untouched. All test/build/profile/A/B/site/install jobs terminal. Whole-project formatter passes with0 files/snippets/fingerprints changed;source checkpoint `239b5d14` contains the accepted runtime/witness,SQLite measurement and public selection. Source parent057a624e (metadata only over a1dde20f).

Pruned94 closed scratch/raw/native artifacts (24229538bytes) after source checkpoint and independent ingestion verification. Cached /tmp/tsrust-schema-native-args.json and shared target stay reusable. Next profile source-retaining schema1.678792/1.212250ms (.466542ms excess),then remaining collection/record JSON overhead1.737083/1.308709 (.428374ms),radix .807541/.427583 (.379958ms),normalization1.976042/1.636000 (.340042ms) and literal JSON union .404333/.151584 (.252749ms). Plain Map/Set validation already allocates less than Rust;focus remaining record JSON and transformed-collision costs. No arena/RC or ownership-contract weakening. Do not reopen closed Array.from/Set entries/int division/Map-string gaps from stale rows. Preserve numeric/object lifecycle plan;full suites/CI/promotion deferred until all large gaps reach nits.

**String-array standard join — accepted locally and installed; parent `e7f30c6a`.**
Product source:emit/mod.rs,runtime/mod.rs,new runtime/string_array_join.rs,tests/array_join.rs. Regular string arrays use Rust slice join over existing heap Cow strings;inline arrays lend at most four str values through stack slots. Fixed literal-string arrays call their existing borrowed slice’s standard join directly. No staged heap vector,element copies,new dependency,unsafe,quota,forced inlining or ownership/API/default change. Mixed/nested/numeric joins retain their original single-pass generic writer. Public signatures/behavior/docs/examples are unchanged. Rust’s own checked join capacity/allocation semantics apply.

Profile-first native allocation samples reveal43,008 allocations/12,297 reallocations/5,877,325 requested/911,341 retained/912,030 peak bytes;independent hand-written Rust allocation profile59,393/6,144/3,182,592/673,792/674,147. Both checksum638609391,input19,cleanup0;three identical samples/version. Native allocates fewer blocks but requests more bytes and grows buffers twice as often. Rust allocation profiling is not fresh execution timing. Four-way inline spread snapshot/Set collection probe saves6,144/2,048 allocations but only~4%/~2–3% runtime;combined also~4%,with inline snapshots costing~90–98ms compilation. Rejected;do not reopen without a compile-cost hypothesis. Original protocol,five full identities,max load2.412,watchdog21.710s/304,560KiB.

Output-growth probes isolate string join (12,297→6,153 reallocations) from String.raw (no consistent runtime gain). Inline exact-sizing Option chains improve~11–17% runtime but cost~11–16ms native compile;checked-fold version still costs~10ms on standalone join. Shared checked helper leaves~5ms cost;saturating arithmetic leaves~8ms;none retained. Standard slice join solves that cost:generated-only standalone parent/inline-fold/shared-std/never-std compile medians304.025/317.730/298.354/297.474ms;runtime parent .221000/.211000,shared std .132958/.130250. Call spread compile684.370/695.342/674.657/677.485ms;parent1.608834/1.594792,shared std1.418000/1.405500. Default helper inlining chosen;no forced hint. Original8warmups/3×31samples/two rotated rounds,five exact identities,same paths/argv/atomic swaps,unsafe_code forbidden,max loads2.482/2.256. Diagnostic timings not ingested. Initial intermediate build stopped deliberately20.244s/2,292,258KiB to repair runtime-helper selection;no checks claimed from that stopped build. Final std helper has its own narrow selector;obsolete direct-append selectors removed.

Final joint selected no-run build passes211.418s/2,602,210KiB;additional collection-test links reuse it in2.067s/253,680KiB. All30 affected checks pass,none ignored,serial3threads/5GB30min:compiler iterable3 (2.589s/365,200KiB),collection2 (1.035s/180,640KiB),call spread13 (4.718s/495,008KiB),join4 (2.092s/471,712KiB);editor iterable3 (.524s),collection2 (.525s),join1 (1.566s/180,528KiB),call spread2 (1.589s/197,968KiB). Strengthened O0/O3 witness proves exact capacity,one allocation for nonempty output,zero for empty output across counts0/1/4/5/256,empty/Unicode/NUL elements and separators;full cleanup and independent output. Existing literal storage,evaluation,ownership,strict types,unwind/native errors and documented examples pass. Preserve these completed checks.

Actual final std-helper A/B passes call spread1.611125→1.430792ms/1.633834→1.431667 (1.126×/1.141×),warm native compile685.413→680.559ms;string join .213667→.130250/.208667→.131208 (1.640×/1.590×),compile308.613→295.578ms;Array.from collections1.202000→1.122666/1.181625→1.105458 (1.071×/1.069×),compile562.978→554.613ms (both broad overlapping ranges). Call spread allocation/reallocation/requested/retained/peak counters43,008/6,153/5,699,149/847,853/848,573;string join5,466/8/297,873/97,257/97,544 (parent5,466/2,464/373,380/121,200/121,476). Collections43,009/3,019/2,835,907/631,123/632,001,input19,checksum335226169. Three identical samples/version;checksum/input unchanged and cleanup0. All three original2048-iteration/seed7/input/8warmups/3×31sample/two-rotated-round protocols,five full identities,same paths/argv/atomic swaps and generated unsafe_code forbid pass. Max loads2.804/2.804/2.660;watchdogs9.836s/304,464KiB,4.661s/332,128KiB,8.305s/293,728KiB. Seven controls emit byte-identically to parent:integer/boolean/nested/number/nullish joins,interfaces,Array.from Map strings. No material regression;Node/Rust execution references cached unchanged.

Canonical native-only release/measurement is terminal0,run2026-11-04T05:48:53.194Z,watchdog477.647s/2,255,900KiB,including required quiet wait after preparation load>4;max accepted timing load3.806<4,no override. It measures the three affected workloads plus nine selected remaining-gap candidates,not all933:array_from_map_string,native_schema_collections,native_schema_json_source_retain,native_schema_json_object_retain,set_entries,native_json_literal_union,number_to_string_radix,int_divide,scoped_record_composition. The full selected list and results are recorded here and in SQLite;closed raw inputs were pruned after checkpoint. Independent oracles/source guards and all93samples pass. Do not rerun completed timings unless final emissions change. All earlier build/test/profile/A/B jobs terminal. Confirmed helper-selection edge defect before checkpoint:standalone console text 'tsr_string_array_join(' emits helper without Array declaration;native rustc E0425/E0433. Selector now also requires existing owned_support,which actual calls establish. New standalone O0/O3 regression checks text output and absence of helper declaration. Guard-focused joint build passes209.407s/2,584,418KiB;compiler array_join5 passes3.118s/474,912KiB and editor join intelligence1 passes1.556s/174,608KiB. Earlier30 affected checks preserved;31 distinct accepted checks total. All12 final-debug emitted hashes match canonical exactly,so measured native code is unchanged. Actual changed-workload A/B compiles with unsafe_code forbidden.

Run2026-11-04T05:48:53.194Z is ingested with npm run store:ingest:exactly12native histories added (call spread12→13,string join5→6,collections10→11),all other engines/history counts unchanged. Independent median/mean rate/operation count/93samples/date recomputation matches every entry;SQLite integrity OK. Native milliseconds:call spread1.426042,string join .131583,collections1.064041,Map strings .820875,schema collections1.816334,source-retaining schema1.678792,object-retaining schema .806125,Set entries .576667,literal JSON union .404333,radix .807541,int division4.101208,scoped record composition20.279584. Public performance selection99 accepted runs/933 comparisons;two page tests pass. Native/Rust ratios for refreshed closed gaps:collections .763×,Map strings .514×,Set entries .871×,int division .999×,string join .889×. Call spread1.121× is a smaller residual. These refreshes include prior shared gains;do not attribute every refreshed improvement to this join change.

Guard-fixed release build passes216.909s/2,122,402KiB. Final release/debug emissions match for all12 measured workloads and six additional controls;all12 canonical emitted hashes match stdout.trim. Seven controls still match the parent (Map strings is included among the12). No timing rerun needed. Site build passes67pages (3.636s/1,141,582KiB);both performance-page tests pass,selection99 accepted runs/933 comparisons. Whole-project formatter passes:0 files,0 documentation snippets and0 fingerprints changed.

Paired final release consumers atomically installed and SHA256-verified: CLI dbcbcbb8376c4847c5b6bf7dcac2b48a174e0bb5488acf1d0f97112714a519b2; LSP 444fc42e723bffea7b05dc12c5ab16d0e04facbb215574d7856941a9edbabeee. Installed CLI checks unchanged hello/src/main.tsr (watchdog.521s;sampled80KiB too short to characterize memory). Owner source/config/editor processes untouched. All current build/test/profile/timing/site/install jobs terminal. Source checkpoint `c6c8fc74` contains the accepted emitter/runtime/tests,SQLite histories and public selection. Pruned183 closed scratch/raw/native artifacts (47895443bytes) after commit and independent verification. Shared target and serde args stay reusable.

Next profile schema collections1.816334/Rust1.308709ms (.507625ms excess) or source-retaining schema1.678792/1.212250 (.466542ms),then radix .807541/.427583 (.379958ms) and literal JSON union .404333/.151584 (.252749ms). Normalization1.976042/1.636000 (.340042ms) is a smaller existing signal. Scoped record composition20.279584/18.420917 is~10.1% slower and needs protocol/CPU/allocation inspection before calling it a large removable gap;do not rank I/O by absolute differences alone. Do not reopen closed Set entries/Array.from/Map-string/int division gaps from old rows. Preserve numeric/object lifecycle plan below. Full regressions/CI/push/publication still deferred to the owner’s large-gap milestone.

**Basename byte scanning — accepted locally and installed; parent `2fda2d8b`.**
Generated-only four-way profiling isolates reverse byte scanning and a proved short-suffix UTF-16 length bound. Parent .884084/.851000ms, bytes .687625/.673875, bound .677958/.689042, combined .478541/.486500 (1.847×/1.749×). Warm compile medians315.873→313.981ms. Three allocation samples per version unchanged:20,001 allocations/12 reallocations,1,702,672 requested/916,432 retained/peak bytes,checksum932196939,cleanup0. Original20,000 iterations/seed7/8warmups/3×31samples/two rotated rounds,five checksum/exact identities,same paths/argv/atomic swaps,max load2.442,watchdog10.880s/273,696KiB. Prototype only; not ingested. Cached Node/Rust execution references remain unchanged.

Runtime implementation reuses the existing ASCII separator helper and preserves drive/suffix state,including whole-path equality,whole-component retention,partial suffixes and trailing separators. UTF-8 uses at most three bytes per UTF-16 unit; suffix bytes<=path bytes/3 proves the original length condition,otherwise exact unit counting remains. No ownership/API/default/dependency/unsafe change. Expanded Node correctness corpus and native O0/O3 all-scalar boundary/allocation witnesses pass. Final accepted measurement and cleanup are recorded below;prototype timings were not ingested.

All24 affected checks pass,none ignored,serial/3threads/5GB30min: native_path8 (3.618s/157,648KiB),allocation4 (.523s),relative6 (2.600s/360,320KiB),editor optional-native1 plus five native-alias/factory/help cases. Batched no-run build211.935s/2,638,162KiB. New O0/O3 witness exercises4,448,256 path/suffix/platform cases per level,including every valid scalar,separator scalars and complete component suffixes. Expanded Node reference checks UTF-8 width boundaries,NUL,whole/partial/long suffixes and both platform conventions. Owned-result allocation remains exactly one for nonempty output,zero for empty.

Actual-compiler quiet matched A/B passes .865125→.493834ms/.876875→.500000ms (1.752×/1.754×),warm compile median314.555→309.350ms. All three allocation samples match the prototype and parent exactly. Five checksum/exact identities,original8warmups/3×31samples/two rotated rounds,same paths/argv/atomic swaps,unsafe_code forbidden,max load3.166,watchdog5.197s/326,592KiB. Adjacent controls differ only in unused basename body; interfaces emit byte-identically. Parent→current runtime milliseconds: dirname .375125→.367792/.361375→.367083;extname .422083→.419792/.432334→.418292;normalize1.407917→1.420292/1.371208→1.372083;interfaces .133875→.134875/.132833→.132792. Warm compile medians304.425→304.811/304.410→306.317/338.865→336.811/255.246→252.780ms. No material control regression; small variations are not attributed to unused support. Controls13.935s/287,760KiB,max load3.207. Cached Node/Rust timing references retained.

Final release/canonical run2026-11-04T05:09:55.741Z passes independent correctness cases and93samples: .483458ms/40,124,320.596mean workload operations/s (20,000 operations/sample). Watchdog219.045s/2,238,796KiB includes release build;max load2.524<4,no override. npm run store:ingest adds exactly one history (native4→5);independent recomputation of median,mean rate,sample count,operations,timestamp and unchanged Node/Rust histories passes;SQLite integrity OK. Performance selection98 accepted runs/933 comparisons;two page tests and67-page build pass (3.642s/971,371KiB). Whole-project formatter0 changed files/snippets/fingerprints. All five final debug/release emissions match;canonical emitted SHA matches stdout.trim. No generated unsafe.

Paired release consumers atomically installed and SHA256-verified: CLI832311d3f2baeaad49c9e4005738e6ed5ad030363fb9f948497f3543edb86314,LSP4b629fe35413d3ce04b581eae5212b7864528ea5031e59a8b2b383f9c64824af. Installed CLI checks unchanged hello/src/main.tsr;owner source/config/editor processes untouched. All current build/test/profile/timing/site jobs are terminal. Basename is now1.122× saved Rust .431000ms (.052458ms excess),closing this large gap. Next profile iterable_call_spread1.703666/1.272250ms (.431416ms excess);normalization1.976042/1.636000ms is the next smaller residual. Preserve object lifecycle plans and completed acceptance evidence;full regressions/CI/push/publication remain deferred to the large-gap milestone. Source checkpoint `5987e6c6` contains the accepted runtime/tests,SQLite measurement and public selection. Pruned52 closed scratch/raw/native artifacts (9076585bytes) after commit and independent ingestion verification. Shared target/current consumers and serde args remain reusable.

**Exact normalization output reservation — accepted locally and installed; parent `3a27e685`.**
At profiling start, selected normalization2.231167/native versus1.636000/saved Rust has .595167ms excess. Matched generated-only allocation profile on the accepted compiler reproduces30,721 allocations/15,349 reallocations,1,565,566 requested bytes,567,094 retained/567,136 peak,input19,checksum332162546,cleanup0. Direct manual output loops reserve either input bytes or exact UTF-8 bytes and improve runtime~11–13%,but cost~13ms warm native compile; not retained. Outlined helpers also cost9–16ms and are rejected. Standard String::extend avoids most of that compile cost. Input-byte reservation saves growth but may overreserve composed strings; exact reservation is preferred.

Chosen generated-only form borrows the packed scalar slice through a reusable character iterator,counts bytes with Rust char::len_utf8,reserves once and extends from the same conversion. It adds no helper,dependency,limit,unsafe,copy/RC/arena or ownership/API change. Prototype2.045708→1.849333ms/2.068917→1.847417ms (1.106×/1.120×),warm compile medians366.533→368.010ms with overlapping362.611–367.655/365.341–369.790ms ranges. Three identical allocation samples each: allocations30,721 unchanged,reallocations15,349→5,742 (-9,607),requested1,565,566→1,384,011 (-181,555),retained567,094→489,539 (-77,555),peak567,136→489,581,input/cleanup unchanged. Original2048/seed7/8warmups/3×31samples/two rotated rounds,five complete checksum/exact identities,same paths/argv/atomic swaps,max load1.795,watchdog10.869s/357,280KiB. Standard char sizing and a threshold-count loop were compared; retained form uses the standard character method and borrowed iteration. Prototype timings are not ingested; Node/Rust timing baselines remain cached.

Implemented four-line output replacement in runtime/normalization.rs. New native O0/O3 witness covers UTF-8 width boundaries,noncharacters,max scalar,empty/ASCII,inert and long composed/decomposed/compatibility/leading-mark strings,exact owned-output capacity,no growth for inert single scalars and complete cleanup. Existing official399,300 normalized-output checks now also assert exact output capacity; all prior1,101,784 identities,expanded decomposition/page/ownership/order cases remain required. No Unicode data or public signature change.

All14 affected checks pass,none ignored,serial/3threads/5GB30min. Selected compiler/LSP no-run build207.713s/2,649,746KiB. Compiler string_normalize12 pass4.679s/505,760KiB; LSP intelligence string_normalize2 pass1.030s/18,000KiB. Official399,300 expected outputs also have exact capacity; prior1,101,784 identities,2,224,128 scalar/mode decompositions at O0/O3,page lookups,ownership,order,forms and catalog examples remain green. New O0/O3 boundary/long-string witness confirms exact owned capacity,no growth for inert scalar boundaries,and complete cleanup.

Actual-compiler quiet matched A/B passes2.080375→1.828209ms/2.067375→1.816000ms (1.138×/1.138×). Three allocation samples per side match the borrowed prototype exactly:30,721 allocations unchanged,15,349→5,742 reallocations,1,565,566→1,384,011 requested bytes,567,094→489,539 retained,567,136→489,581 peak,input19,checksum332162546,cleanup0. Warm native compile median364.646→369.263ms (+4.617ms),within overlapping parent360.997–370.358/current369.257–369.542ms ranges; do not claim compile improvement. Unlike rejected13ms manual loops/outlined forms,this standard form has no demonstrated material compile regression. Original2048/seed7/8warmups/3×31samples/two rotated rounds,same paths/argv/atomic swaps,five complete checksum/exact identities,max load2.214,watchdog7.250s/318,368KiB. Interfaces,mixed numeric conversion,basename and set algebra emit byte-identically; Node/Rust execution baselines remain cached. Generated code compiles with unsafe_code forbidden; allocation instrumentation is separate.

Final release/canonical run2026-11-04T04:48:32.599Z passes all independent correctness cases,93 samples:1.976042ms/1,044,723.296mean workload operations/s (2048 original operations/sample). Watchdog218.982s/2,193,068KiB includes release build; max timing load2.633<4,no busy override. npm run store:ingest adds exactly one history (native15→16). Independently recomputed median,mean rate,operation count,sample count/timestamp match; SQLite integrity OK. Node/Rust timing references remain cached. Performance selection97 accepted runs/933 comparisons; two page tests and67-page site build pass (3.643s/1,213,919KiB). Whole-project formatter passes0 files/snippets/fingerprints changed. All five final debug/release emissions match; canonical source SHA matches after stdout.trim; four unrelated controls match parent byte-for-byte. No generated unsafe.

Atomically installed paired release consumers and SHA256-verified: CLI971c1c91b747e65fdf490caf1068dc0d55a25aab86c5dfd91e8dcf9caeb5e724,LSPd6cb5f73175b2756b61a6a3f9195f17657f2507b2d9b64201d17042a9f22bef5. Installed CLI checks unchanged hello/src/main.tsr (0.521s); owner source/config and editor processes untouched. All current profile/build/test/timing/site jobs are terminal. Source checkpoint `e6291a29` contains accepted runtime/tests,SQLite row and public performance selection. Pruned54 closed scratch/raw files (18913332bytes) after checkpointing and independent verification. Profile basename.877500/native versus.431000/saved Rust next (.446500ms excess),then iterable call spread1.703666/1.272250ms (.431416ms). Normalization now1.976042/1.636000ms (.340042ms excess,1.208×) is a smaller residual; do not claim exact parity or milestone completion. Preserve the numeric/object lifecycle plan below. Do not reopen closed key collection,Map/Set snapshots or decimal parsing from stale rows. Full regressions,CI,push/publication remain deferred until all large gaps reach nits. Shared target/current consumers and /tmp/tsrust-schema-native-args.json stay reusable.

**Object.keys direct collection — accepted locally and installed; parent `fc7e87f0`.**
At profiling start, selected object_keys_order1.796417/native versus1.086375/saved Rust has .710042ms excess. Direct-work3s CPU sample shows malloc/free,reallocation,memmove,array push and integer formatting; profile6.723s/249,408KiB. Three allocation samples each agree: native55,085 allocations/16,393 reallocations/3,368,548 requested bytes/542,788 retained/543,469 peak; unchanged hand-written Rust55,297 allocations/9 reallocations/1,668,000 requested/513,024 retained/513,564 peak. Both checksum102068690,input19,cleanup0. Rust allocation counters were profiled,not its cached execution timing. This is key/vector/string lifecycle overhead; allocation count alone would conceal it and does not justify an arena or record-storage redesign.

Generated-only four-way probe isolates direct collection and cached single-digit key formatting. Parent1.690500/1.674167ms; direct collection1.392667/1.386541; digits1.348958/1.360083; combined1.067625/1.058458ms (1.583×/1.582×). Warm compile medians325.611→324.613ms combined (stream319.599,digits331.029). Combined allocations55,085→28,461 (-26,624),reallocations16,393→14,345 (-2,048),requested3,368,548→1,873,508 (-1,495,040),retained542,788 unchanged,peak543,469→543,222,cleanup0. Three identical allocation samples per variant; original2048/seed7/8warmups/3×31samples/two rotated rounds,same paths/argv/atomic swaps,five complete checksum/exact identities,load max2.240,watchdog13.996s/306,432KiB. These are prototype timings,not ingested compiler measurements.

Implemented standard FromIterator for existing private Array<T>: lower bound>4 collects directly to Vec/Heap,otherwise existing inline storage/push handles both exact and unknown-length iterators. Object.keys now collects directly to its native result,removing staged Vec plus copy/growth. Numeric keys0..9 reuse existing tsr_int_text; its i64 cast occurs only in that proven small range,while other indices retain full usize.to_string behavior. Record constructor order,tuple/array/string UTF-16 keys,empty/primitive behavior,evaluation,ownership and mutations are unchanged; no public API,default limit,new dependency,arena,RC or generated unsafe. New witnesses cover record/tuple/string inline/heap boundaries0/1/4/5/8/9/10/11/17/33/129,zero staged allocations,zero key-array reallocations,independent string/key mutation,unknown iterator hints and exact Drop cleanup at O0/O3.

All44 affected checks pass,none ignored,serial/3threads/5GB30min. Selected batch build212.501s/2,625,314KiB. Native37: object_keys8 (2.596s/107,456KiB),array_from_collections2 (1.553s/166,064KiB),array_join4 (2.079s/459,600KiB),arrays12 (3.111s/434,608KiB),interfaces7 (1.557s/483,936KiB),object_values_entries4 (1.040s/390,800KiB). Editor7: array_from_collections2 (0.524s),interfaces3 (0.520s),Object.keys intelligence2 (1.057s/10,144KiB). New O0/O3 witnesses prove small record/tuple/empty results need no heap storage,large records one allocation,exact numeric key arrays never reallocate,all boundary key strings/mutations are independent,and non-Copy iterator values drop once for exact/unknown hints. Existing receiver evaluation/order/clone/UTF-16/primitive/generic/scoped behavior stays green.

Actual-compiler quiet matched A/B passes1.689958→1.070833ms/1.692375→1.076833ms (1.578×/1.572×); warm native compile median324.613→322.168ms improves. Allocation/reallocation/requested/retained/peak/cleanup counts match the combined prototype exactly in three samples. Four controls differ only in unused standard collector support; independent removal of that exact support yields parent byte-for-byte. Runtime controls: interfaces.132083→.132542/.133334→.135417ms; small nullish packs.041417→.041625/.041459→.041292; join strings.219667→.214084/.216625→.209958; values/entries.796750→.783958/.787958→.785875. Warm compile medians253.296→250.289/279.719→280.679/303.002→303.346/374.035→376.940ms. No meaningful control regression; do not attribute small control shifts to unused support. Same paths/argv/atomic swaps,original8warmups/3×31samples/two rotated rounds,five complete checksum/exact identities per workload,max load1.691,watchdog23.306s/293,344KiB. Generated code compiles with unsafe_code forbidden; allocator instrumentation is separate. Node/Rust execution baselines remain cached.

Final release/canonical run2026-11-04T04:19:30.045Z passes all independent correctness cases,93 samples:1.139667ms/1,815,343.865mean workload operations/s (2048 original scenario operations/sample). Watchdog217.302s/2,235,515KiB includes release build; max timing load2.117<4,no busy override. npm run store:ingest adds exactly one history (native5→6). Independently recomputed median,mean rate,operation count,sample count/timestamp match; SQLite integrity OK. Saved Rust1.086375ms remains unchanged; current native is1.049× that reference,so this large gap is closed without claiming a new Rust timing. Performance selection96 accepted runs/933 comparisons; two page tests and67-page site build pass (3.611s/1,160,670KiB). Whole-project formatter passes0 files/snippets/fingerprints changed. All five final debug/release emissions match; canonical source SHA matches after the runner's stdout.trim,and all four controls differ from parent only by unused collector support. No generated unsafe.

Paired release consumers installed atomically and SHA256-verified: CLIa2f003d7d7684938dfcbdb55f10331bfa4d70e666e62f5951cf746e9c92b3510,LSPaf77a663cc6997f80b301bf36b1f98c0ed0f993b277bc0d0a62fec1d36b54b6f. Installed CLI checks unchanged hello/src/main.tsr (0.515s); owner source/config and editor processes untouched. All current profile/build/test/timing/site jobs are terminal. Source checkpoint `738a97b8` contains accepted emitter/runtime/tests,SQLite row and public performance selection. Pruned40 closed scratch/raw files (12781033bytes),including the generated dSYM,after checkpointing and independent verification. Re-rank residual normalization2.231167/1.636000ms (.595167ms excess),basename.877500/.431000ms (.446500ms) and iterable call spread1.703666/1.272250ms (.431416ms). Normalization output still has15,349 reallocations per workload; compare output construction/reservation against the matched Rust before another change. Earlier exact UTF-8 sizing was rejected for compile cost on an older normalizer; keep that evidence and require fresh runtime/compile/allocation checks before revisiting it. Preserve the numeric/object lifecycle plan below; do not reopen closed key collection,Map/Set snapshots or decimal parsing from stale rows. Full regressions,CI,push/publication remain deferred until all large gaps reach nits. Shared target/current consumers and /tmp/tsrust-schema-native-args.json remain reusable.

**Expanded normalization tables — accepted locally and installed; parent `82438bb1`.**
A3s direct-work CPU sample excludes untimed result observers and identifies normalizer work plus allocation/free/growth costs (1,089 top-of-stack normalizer samples,watchdog5.688s/278,752KiB). Parent allocation profile is36,454 allocations/15,349 reallocations,1,657,294 requested bytes,567,094 retained/567,136 peak,input19,checksum332162546,cleanup0. Pinned UnicodeData analysis finds16 canonical mappings with distinct compatibility expansions. Generated-only pre-expanded/packed-class probe improves2.573042→2.088000ms/2.604625→2.072208ms and370.996→364.303ms compile median before implementation. No prototype timing is ingested.

Runtime/generator/tables/README and native witnesses now expand chains offline,cache leaf combining classes,and remove the pending Vec. Main rows keep the same keys/page offsets; kinds0/1 retain canonical/compatibility-only semantics,2+ index the16 alternative compatibility ranges. Composition pairs still use original raw mappings. Packed words8,739→9,221 add2,120 read-only bytes. No public API,Unicode version,ownership,dependency,input limit or generated unsafe change. Generator validates pinned counts,scalar/range/kind widths and class-zero Hangul Jamo. Generation is idempotent; class/composition/page tables,assigned ranges and pinned sources match parent byte-for-byte.

All13 affected checks pass,none ignored,serial/3threads/5GB30min. Batched selected compiler/LSP no-run build208.930s/2,632,066KiB; compiler string_normalize11 pass4.681s/411,744KiB; LSP intelligence filtered string_normalize2 pass1.031s/96,720KiB. Existing official399,300 normalization relationships/1,101,784 identity checks remain green. New independent raw-UnicodeData reference checks2,224,128 scalar/mode decompositions at each of O0/O3,packed classes and all alternatives. Nontrivial canonical/compatibility/Hangul calls use only scalar/output allocations and clean up completely. Existing2,228,226 page/full lookup checks,receiver/form order,invalid forms,owned/temporary/optional values and independent outputs pass. Preserve these results; no broad/full suites ran.

Final actual-compiler quiet A/B passes2.579750→2.068250ms/2.583167→2.035042ms (1.247×/1.269×); three warm rotated compile medians373.058→360.422ms improve. Three allocation samples per side agree:36,454→30,721 allocations (-5,733),1,657,294→1,565,566 requested bytes (-91,728); reallocations/retained output/peak/input unchanged and cleanup0. Original2048/seed7/8warmups/3×31samples/two rotated rounds,five complete checksum/exact identities,same paths/argv/atomic swaps,max load2.637,watchdog6.750s/311,248KiB. Generated code compiles with unsafe_code forbidden; allocation instrumentation is separate. Interfaces,mixed numeric conversion,path basename,set algebra emit byte-identically to parent. Node/Rust baselines remain cached.

Final release/canonical run2026-11-04T03:48:46.914Z passes all independent correctness cases,93 samples:2.231167ms/921,326.211mean workload operations/s (2048 original workload operations/sample). Watchdog217.503s/2,234,284KiB includes release build; max timing load2.800<4,no busy override. npm run store:ingest adds exactly one history (native14→15). Independently recomputed median,mean rate,operation count,sample count/timestamp match; SQLite integrity OK. Node/Rust reference timings remain cached. Public performance selection now95 accepted runs/933 comparisons; two page tests and67-page site build pass (3.642s/1,214,446KiB). Whole-project formatter passes0 files/snippets/fingerprints changed. All five final debug/release emissions match; changed normalized-source SHA matches canonical after the runner's stdout.trim; all four unrelated controls match parent byte-for-byte. No generated unsafe.

Atomically installed paired release consumers and SHA256-verified: CLIab3e926abc99068b85a1bd2f7d8df9a1e7caf5782edb442aeb61500a94454886,LSP6222dc62dcc47f660b761258e88f530a4023c79e798d9587f1b2a9c099eb9d7c. Installed CLI checks unchanged hello/src/main.tsr (0.520s); owner source/config and editor processes untouched. All current profile/build/test/timing/site jobs are terminal. Source checkpoint `0ca813fa` contains accepted runtime/tables/generator/tests,SQLite row and public performance selection. Pruned37 closed scratch/raw files (12018637bytes),including the generated dSYM,after checkpointing and independent verification. Largest listed remaining absolute gap is object_keys_order1.796417/1.086375ms (.710042ms excess); profile its complete construction/key-order/string lifecycle against matched Rust next. Normalization2.231167/1.636000ms still has .595167ms excess (1.364×),then basename.877500/.431000ms and iterable call spread1.703666/1.272250ms. Do not claim normalization parity or milestone completion. Preserve the numeric/object lifecycle plan below; do not reopen closed Map/Set snapshots or decimal parsing from stale rows. Full regressions,CI,push/publication remain deferred until all large gaps reach nits. Shared target/current consumers and /tmp/tsrust-schema-native-args.json stay reusable.

**Decimal trimming optimization — accepted locally and installed; parent `4e8f4038`.**
Before that decimal section, comparable native/Rust milliseconds were decimal5.592875/4.178834 (1.414041ms excess),normalization2.754167/1.636000 (1.118167ms),object keys1.796417/1.086375 (.710042ms),basename.877500/.431000 (.446500ms),iterable call spread1.703666/1.272250 (.431416ms). Recursive parsing1.772375/1.548083 and export1.298916/1.127833 are smaller residuals. Cached Node/Rust controls stay unchanged.

A3s CPU sample of current decimal parsing found743 trim_matches samples among2,171 application-stack samples (~34%); Rust's actual f64 parsing is the other major cost. Profile5.698s/269,936KiB. Generated-only ASCII trim probe improves decimal5.285667→3.830083ms/5.245292→3.842292ms. Runtime/numeric_string.rs now uses std str::trim_ascii,then the existing conversion-whitespace predicate only when a remaining boundary is non-ASCII or vertical tab. Rust's ASCII trim excludes vertical tab; preserve it explicitly. A safe slice pattern avoids Option closures and reduces their initial~9ms warm native compile cost. No grammar/default/ownership/API change,new dependency,allocation or generated unsafe. All callers of whole-string Number conversion (including unary plus,unions,enums) retain the same shared helper.

Extended the existing numeric_string_conversion O0/O3 witness over all1,112,064 valid Unicode scalars at left/right/both boundaries (3,336,192 cases per optimization level),plus405 mixed ASCII/Unicode/vertical-tab empty/negative-zero/decimal/radix/Infinity cases. Grammar,rounding,special spellings and input retention remain checked. Initial Rust fixture incorrectly used static Borrowed storage for a dynamic formatted string; repaired to existing Owned storage before acceptance. Final batched selected compiler/LSP build208.286s/2,628,066KiB. All17 affected checks pass,none ignored,one binary at a time/3threads/5GB30min: conversion7 (4.134s/456,400KiB),numbers7 (3.676s/482,704KiB),unary2 (1.043s/61,872KiB),focused numeric editor1 (1.043s/27,024KiB). Preserve these passes; no full suites ran.

Actual-compiler matched A/B under stricter quiet load<2 passes all five boundary/full checksum/exact identities,same source/output paths/argv/atomic swaps,8warmups/3×31samples/two rotated rounds. Decimal5.243125→3.824917ms/5.265542→3.812167ms (1.371×/1.381×),mixed.185458→.177875/.190375→.171250ms. Unary.381666→.377125/.364000→.381500ms remains variable; byte-identical number_from_int control varies.349166→.320459/.336291→.328209ms,so these small shifts do not establish a source regression or gain. An earlier load<4 run also showed isolated control variation; stricter repetition resolves the noise concern,not a new measurement protocol. Warm native compile medians: decimal285.003→288.465ms (+3.462ms/~1.2%),mixed380.118→382.376,unary370.653→374.091,identical int205.953→207.816ms. Record the small compile cost; do not claim compile improvement. Exact slice-source prototype had overlapping compile ranges285.180–289.733/287.181–288.367ms. Runtime gain is stable; no meaningful control regression demonstrated. Max load1.984,watchdog90.305s/331,888KiB includes quiet waits. Prototype/rejected/diagnostic timings are not ingested.

Final release/canonical run2026-11-04T03:25:13.466Z passes all independent correctness cases,93 samples per workload: decimal4.008875ms/124,819,674.417mean ops/s,mixed.173000ms/11,539,790.494ops/s,unary.405333ms/5,063,591.094ops/s. Watchdog219.533s/2,236,284KiB includes release build; max timing load2.698<4,no busy override. npm run store:ingest adds exactly three histories (native counts2→3/9→10/5→6). Independently recomputed sample medians,mean rates,operation counts,count/timestamp match and SQLite integrity is OK. Node/Rust references remain cached. Current decimal is now near/slightly below saved Rust4.178834ms; do not claim a new Rust measurement. Performance page selects94 accepted runs/933 comparisons; two page tests and67-page build pass (3.642s/1,217,487KiB). Whole-project formatter passes0 files/snippets/fingerprints changed. All four final debug/release emissions match; changed workload hashes also match canonical input after the runner's documented stdout.trim normalization. Unchanged int control matches parent byte-for-byte. No generated unsafe.

Paired release consumers installed atomically and SHA256 verified: CLI124d4f477bcb2aa4a6680f332bfade5b42c0f116cc41e0873972c3a93958c802,LSPd2d14d084bf03bcf3c2dac26376fda6f51bfc8610876004d047cbc5bd52e06cb. Installed CLI checks unchanged hello/src/main.tsr (0.521s); owner source/config and editor processes untouched. All current build/test/timing/site jobs are terminal. Source checkpoint `2058b40d` contains accepted runtime/tests,SQLite data and public performance selection. Pruned48 closed scratch/raw files (11807476bytes),including the generated dSYM,after checkpointing and independent verification. Resume profiling normalization2.754167/1.636000ms (1.118167ms excess),then object keys1.796417/1.086375ms and basename.877500/.431000ms; preserve the numeric/object lifecycle plan below. Do not reopen closed Map/Set snapshot or decimal gaps from stale rows. Full regressions,CI,push/publication remain deferred until remaining large gaps reach nits. Shared target/current consumers and /tmp/tsrust-schema-native-args.json remain reusable.

**Recursive JSON finalization — accepted locally and installed; parent `fd35647c`.**
Profile 2026-11-04T01:53:04.572Z identified80,000 allocations/8,344,000 requested bytes for2,000 recursive parse iterations. A generated-only prototype supported dropping rollback storage at a final, non-branching recursive JSON entry. Implementation in emit/schema/{mod.rs,pipelines.rs,pipeline_json.rs} emits direct finalizers using existing leaf validation, missing-field trials, named-output conversion and issue helpers. Graph eligibility rejects wrappers/callbacks/unions/collections/retaining objects; their reversible path is preserved. No API change, dependency, new source limit or generated unsafe.

New tests/schema_recursive_direct.rs covers named outputs, exact integers, missing fields, error order, stripped keys, recursive array/scalar roots, nested definitions, both parsing APIs and reversible callback/union fallback. Validation exposed an existing console_value emission loop through recursive array/union types, reproduced in parent release and current debug. It expanded console helpers indefinitely; the diagnostic sample reported13.1GB physical footprint despite low RSS. emit/mod.rs now creates recursive union helpers at runtime, like record helpers, and uses existing logical tsr_data_view conversion for nested owned union payloads. Initial helper validation found wrapper/view mismatches; repaired at the same shared boundary. The local /tmp/tsrust-watch.py now caps aggregate max(RSS,macOS physical footprint) per process as well as30min; a touched32MiB allocation increased measured footprint33,603,608bytes and passed the assert check. Earlier RSS-only peaks are not physical-memory characterizations.

All264 affected checks pass, none ignored, one binary at a time/3threads under5GB30min. Compiler schema-filtered library101; native integration124 (new3,console14,collections5,decode10,diagnostic11,enums10,export9,literals9,parse5,pipelines20,recursive10,super-refine5,unions13); project-intelligence schema39. LSP library schema filter selects0 tests; no LSP-library coverage claim. Preserve these passes. Final batched build488.151s/1,754,112KiB (RSS-only watcher); new fixtures9.324s/1,079,808KiB with corrected watcher. Remaining binaries pass with per-binary peak at most1,117,792KiB; export61.150s,unions56.448s,pipelines49.741s are the longest runs. Failed exploratory runs/builds remain diagnosed, not accepted evidence.

Actual-compiler quiet matched A/B passes: recursive2.633083→1.941625ms and2.622333→1.877208ms (1.356×/1.397×). Three warm rotated compile medians743.602→560.260ms improve. Three allocation samples per side agree:80,000→50,000 allocations (37.5% fewer),8,344,000→7,224,000 requested bytes,peak1,898→1,832bytes,zero reallocations/retained heap/cleanup residue. Export control1.301375→1.302125ms and1.292667→1.296917ms,compile419.045→415.799ms,allocations38,000 unchanged. Its emitted difference is console logical-view conversion/formatting; no meaningful regression. Object-key and normalization consumers emit byte-identically. Original same-path/argv/atomic-swap protocol,8warmups/3×31samples/two rotated rounds and five complete checksum/exact identities pass. Watchdog16.157s/380,192KiB,max load1.840. Node/Rust baselines remain cached.

Release/canonical native-only run2026-11-04T02:56:54.993Z passes all independent result cases and93 samples:1.772375ms/2,131,428.860mean operations/s (two parses per iteration). Watchdog298.084s/2,235,388KiB includes release build and quiet wait from6.81load; max timed load3.750<4,no busy override. npm run store:ingest adds exactly one history (25→26 for recursive/native). Independently recomputed median,mean rate,timestamp,count and4000operations/sample match SQLite; integrity is OK. Allocation A/B above uses actual compiler output,not the prototype. Performance page selects93 accepted runs/933 comparisons; two page tests and67-page site build pass (3.640s/1,222,062KiB). Whole-project formatter passes with0 files/snippets/fingerprints changed. All four final debug/release emissions match. The only unsafe word in recursive/export output is an existing documentation comment,not generated unsafe code.

Atomically installed paired release consumers and SHA256-verified: CLI a335e59f300799ab42436b92a8e02ca97ab1a93b9150c5a7f1dd48f662a79dcf; LSP d00d595261fc9f14f0830118e95a1c394b31413141cd1c98da36ac0f931e403e. Installed CLI checks unchanged hello/src/main.tsr (0.512s); owner source/config and editor processes untouched. All current build/test/timing/site jobs are terminal. Source checkpoint `bed80104` contains the accepted implementation,regression witnesses,SQLite row and public performance selection. Pruned43 closed section scratch/raw files (12032295bytes) after independent verification and checkpointing. Next profile the remaining ranked object/path/normalization/decimal gaps against this compiler. Full regressions,CI,push and publication remain deferred until large gaps are closed. Keep the shared Cargo target and /tmp/tsrust-schema-native-args.json for subsequent profiling.

**HTTP/object/editor section — accepted locally and installed, source `4d97c4d2`.**
Object inference checkpoint `aecaade3` introduced anonymous fixed-layout objects and shape interning; its installed compiler checks the unchanged hello project. This section adds generic object argument inference without evaluating arguments or weakening ownership; source-derived Router contracts, config-free TypeScript/Rust clients and OpenAPI; a source-first starter/tutorial; literal-key hover/definitions and native ContextNext help, including renamed parameters and zero-argument signature help. Existing configured RPC remains compatible. Contextually typed literals (Router state included) use checked expression-result facts. Inferred async Hono middleware can return a response, fall through after next(), use bare return or return-await-next. A private erased result conversion preserves ordinary/explicit return rules and nested closure scopes. Generated fallback reachability uses the existing generated_continuations marker at a unique empty closing span. No new dependency or generated unsafe. Native int64 contracts/Rust models preserve precision; TypeScript rejects unsafe integer values rather than rounding successfully.

45 affected compiler/API cases pass across resumed groups: inferred objects3, JSON encoding3, generic expression flow4, generic built-ins2, API contracts11, API projects4, existing Rust SDK2, Router clients3, scaffold4, HTTP context9. Preserve these results. The final changed HTTP witness passes63.239s/609,152KiB; other8 HTTP cases previously passed69.519s/946,320KiB. Router3 pass2.590s/369,696KiB, scaffold4 pass0.525s/1,632KiB. SDK checks compile real TypeScript5.9.3 declarations and exercise generated fetch/Reqwest calls; Rust round-trips9007199254740993 exactly. Nearest ordinary project blocks parent RPC discovery. Native middleware checks cover unauthorized/authorized/unfinalized responses, before/after headers, nested closures and continuation ownership. Unfinalized middleware correctly returns500, not404 (fixture repaired).

208 affected editor cases pass: all180 project_intelligence cases, the inferred-object completion row across4 typing modes, and27 editor_consistency cases. Object/key/definition witness1.050s, ContextNext/renamed-next hover and signature witness0.525s; remaining178 cases1.570s/41,920KiB; typing row3.730s/17,136KiB; consistency13.645s/130,608KiB. Constructor-key diagnosis found literals in checked_method_results, not only checked_views; use both exact-span tables. Field definitions retain compiler source locations. LSP build199.473s/1,377,664KiB; final LSP-only repair10.397s/520,576KiB.

Whole-project formatter passes with0 files/snippets/fingerprints changed. Website builds67 pages (4.229s/998,224KiB). Required offline compiler/LSP --all-targets check passes (116.270s/1,403,024KiB), with one pre-existing lib-test dead_code warning for generated-only JSON decode_plain (actual emitter and json_parse_plain callers verified; not removed/suppressed). No performance measurements changed; Node/Rust timing controls were not rerun. No full regressions, CI, push or publication.

Release build of both consumers passes223.118s/1,304,336KiB under the5GB/30min watchdog. Atomically installed ~/.cargo/bin/tsrust and tsrust-lsp; SHA256 matches release outputs: CLI587278a5cd1befe97e6875984ab7f53c518f9154a148dffd53493f619eb57cf8, LSP84b4b4e6f04bf0bb0a6e23d1be1976da7c2973987962d0f429cf8e8f565ae797. Installed CLI checks unchanged hello/src/main.tsr. Config-free `tsrust api generate --out <temporary directory>` from hello emits one GET / route with required greeting:string, TypeScript declarations and OpenAPI. User source/config were not edited. Editor shows zero problems; status-bar restart changed LSP PID40538→11851. Actual hover rendering remains unverified; focused protocol tests above pass. Do not further control the owner editor/processes (agent-goal.md).

Next: ordinary JSON output quotas,native optional arguments,default JSON builder quotas and trusted-root serialization are fixed below. Finish acceptance of the adjacent default-limit section,then resume recursive schema/object/path optimization from the refreshed ranking. Full regressions/promotion stay deferred until large gaps are closed. Preserve deferred broad-regression evidence.

**Ordinary JSON output quotas — accepted locally; parent `6b4a0552`.**
The parent CLI reproducibly rejected an omitted-limit17MiB string with output-limit (5.746s/793,920KiB). Global JSON.stringify/tryStringify now preserve omitted/undefined byte budgets instead of injecting16MiB; checked optional-int emission maps absence directly to native usize::MAX, without an i64 or arbitrary platform quota. std:json.stringify matches for omitted budgets. Typed/Dynamic/raw output depth uses checked native arithmetic without the64-container quota. Explicit caller budgets,negative/zero errors,UTF-8/escaped-byte accounting,error offsets,undefined/nonfinite rejection,borrow semantics and evaluation order remain. No new dependency or generated unsafe. Native17MiB/256-level records,explicit budgets and order witnesses pass; docs/hover/help now describe actual default behavior. Input parsing and JSONL byte defaults are unchanged in this section.

All129 affected cases pass,none ignored: compiler/native35 (encoding4,namespace9,raw13,owned writer5,borrowed writer4),LSP29 (intelligence4,project_intelligence25),compiler JSON library units65 (other1243 deliberately filtered). Execute one binary at a time/3threads under5GB30min with absolute Cargo target and incremental disabled. Encoding11.405s/995,296KiB; namespace8 passed54.918s/1,077,360KiB plus repaired imported-contract case8.296s/375,888KiB; raw28.581s/1,079,184KiB; owned writer15.042s/1,089,632KiB; borrowed writer7.795s/1,079,360KiB; library1.056s (too short for meaningful sampled peak). Initial omitted-limit fixture exposed the shared native optional-argument gap: native bindings reject both explicit undefined and int|undefined forwarding. The fixture now branches to an omitted native call for absence; no claim that native bindings accept explicit undefined. Global adapters do accept it. Fresh final-catalog editor rerun4+25 passes0.521s/0.525s. No unresolved test failure. Initial batched build220.971s/1,115,936KiB; compiler fixture/library rebuild483.414s/1,697,280KiB; namespace repair build1.553s/380,800KiB; final LSP build200.400s/1,557,600KiB. Avoid this repeated feature-context rebuild: include both packages and needed --lib plus selected integration targets in the initial no-run batch; finish catalog wording before building.

This is a required correctness fix, with no speedup claim. Actual-compiler quiet parent/current A/B uses identical paths/argv,atomic swaps,8warmups/3×31samples/two rotated rounds and five exact boundary/full result identities per workload. Parent→current milliseconds: export1.554500→1.561375/1.559792→1.568333; namespace.599959→.597625/.597291→.587291; typed encoding.274417→.283292/.264792→.264958; owned writer58.804959→58.731958/58.185000→57.829750; raw encoding.299500→.271625/.255042→.256500. No consistent material slowdown; three warm rotated compile medians respectively417.120→415.750,377.017→387.122,306.089→311.189,368.186→365.135,313.213→309.683ms. A/B watchdog47.747s/330,480KiB,max load2.967. A generated-only delegation of Dynamic serialization to serde_json::Value was rejected: inconsistent runtime1.547333→1.489875/1.532917→1.528750ms and compile410.691→428.178ms (+4.26%); existing custom serializer retained. Rejected probe data were not ingested.

Release build of both consumers passes224.157s/1,641,840KiB. Canonical native-only run2026-11-03T21:37:18.130Z passes all independent result oracles/93samples per workload: export1.565875ms/2,452,192.498mean ops/s (two exports/iteration),namespace.588459ms/3,421,237.911,typed encoding.263042ms/7,721,086.612,owned writer61.827834ms/31,167.299,raw encoding.252917ms/7,732,909.406. Watchdog199.031s/377,616KiB includes preparation and an enforced quiet wait after load rose above4; accepted sample max load3.817,no busy override. npm run store:ingest adds exactly5 histories; medians,mean rates,sample counts,timestamp and history-count increments independently match; SQLite integrity OK. Existing Node/Rust timing references reused,workloads/protocols unchanged. All five final debug/release emissions match. Public performance page selects88 accepted runs/933 comparisons,two page tests pass,67-page build passes4.147s/856,432KiB. Whole-project formatter passes with0 files/snippets/fingerprints changed.

Installed validated release consumers atomically without controlling the editor process. SHA256 CLI445473ae43eacc01a5f079d7324ce44b3a5615b6f650e34a23c283e7cfd07038,LSP8d0f86fecd6b416a51655688378c83edf7d6532087a8ecc1df89dd4f77a914fe. Installed CLI checks unchanged hello/src/main.tsr; installed17MiB witness returns17825794 encoded bytes (5.208s/951,008KiB). No owner source/config changes. No full regressions,CI,push or publication. All section jobs are terminal;45 closed temporary/raw files (7385102bytes) pruned after verified ingestion. Shared target,current native consumers,serde arguments and deferred milestone evidence retained.

**Native optional arguments — accepted locally; parent `1d45e0cd`.**
Final adversarial check found that native callback adaptation accepted advertised int|string suffixes. Shared native function-shape checking now requires every advertised input type to be accepted by the native operation,including required parameters; three foreign int/null callback witnesses added. Guard no-run build passes501.075s/1,619,600KiB. All244 affected cases rerun and pass: new witnesses2 (8.295s/363,344KiB),project native13 (6.797s/440,864KiB),compiler optional22 (8.415s/473,840KiB),facts3 (0.526s),editor4 (1.572s/209,440KiB),compiler library native_200 (48.924s/579,184KiB). Preserve the other85 passes below. Public diagnostic now reports Expected string|undefined,found int|string|undefined (0.522s/5,600KiB). All five guarded-debug and final-release consumer emissions match the already measured version's SHA256 exactly; canonical measurements and A/B evidence below remain valid. Formatter again reports0 changes.

The parent basename(path,undefined) reproduces Expected string,found undefined (watchdog0.517s). Shared native checking now uses existing optional-union argument/borrow adaptation; spreads use the same contract. Native function-value shapes retain optional types/omission metadata and specialize to contextual required callback contracts,with cache identity including checked parameter types/omission. Shared emission lowers checked optional inputs to the existing Rust Option ABI in direct calls and forwarding bodies; absent inputs become None without dropping their evaluation,while present payloads retain registry access modes. No implicit payload clones,new dependency,configuration or generated unsafe. Imported JSON forwarding no longer needs a manual absence branch; std:json.stringify(value,undefined) is documented and tested,including17MiB output. Function reference/hover/help remain consistent.

All329 affected cases pass,none ignored: compiler/native125,compiler library200 filtered native_ tests (1108 others deferred),editor4. Integration counts: new optional-native witnesses2,project native13,encoding4,namespace9,construction3,raw13,owned writer5,borrowed writer4,JSONL6,borrowed JSONL5,full-file1,buffered-file2,bytes26,path7,ordinary optional parameters22,parameter facts3. Existing required native callbacks,owned record fields,capture/conditional return boundaries,reassignment/move errors and variadic paths remain accepted. New witness executes direct undefined,forwarded optional values,readonly strings/limit records,scalar fill,multiple optional slots,callback contracts,record fields,conditional calls,dynamic optional suffix spread,once/left-to-right evaluation and unchanged owner reads. Negative type/move cases preserve errors. Two initial positive-fixture mistakes were repaired: an unknown-length path array could put undefined in the required path slot; tuple call-spread is unsupported. Final witness keeps the required path explicit and spreads an undefined[] suffix; no new tuple-spread feature was introduced. The passed negative witness was retained. All executions are serial binaries/3threads/absolute shared Cargo target/incremental off under5GB30min. Positive witness9.877s/361,584KiB; project13=6.767s/457,632KiB; namespace9=54.501s/1,087,904KiB; bytes26=48.833s/639,664KiB; compiler library200=48.893s/538,272KiB; editor4=1.573s/184,816KiB. LSP library native_ filter selected0 of20; no additional library-pass claim. Batched compiler/LSP/library+integration no-run build passes502.077s/1,685,280KiB; two fixture-only rebuilds3.632s/246,560KiB and3.117s/263,184KiB preserve the joint package feature context instead of rebuilding core compiler dependencies.

Required correctness fix,no speedup claim. Quiet actual-compiler parent/current A/B (same paths/argv,atomic swaps,8warmups/3×31samples/two rotated rounds,all five exact boundary/full identities) passes. Changed consumers: basename.882250→.872291/.866333→.868583ms,namespace.582792→.582333/.606958→.582916ms. Three warm rotated compile medians317.251→320.382ms and373.944→368.626ms. Typed encoding,raw encoding and owned writer emit byte-identically to the parent; their timing differences calibrate noise,not a source regression or gain. In particular the identical writer varied60.787666→61.844666/57.306000→62.463833ms; do not attribute this I/O variation to the optional-argument fix. A/B50.480s/331,360KiB,max load2.818. Only the two changed native consumers were refreshed canonically; unchanged native controls and all Node/Rust timing references reused.

Initial release CLI/LSP build passes226.292s/1,305,584KiB; final callback-guarded pair passes227.867s/1,556,864KiB. Native-only canonical run2026-11-03T22:24:06.713Z passes independent oracles/93samples each: basename.877500ms/21,627,300.244mean ops/s,namespace.551000ms/3,619,980.069. Watchdog97.950s/335,744KiB includes an enforced quiet wait,max accepted load3.433,no override. npm run store:ingest adds exactly2 histories; independently recomputed medians/mean rates/counts/date/history increments match,SQLite integrity OK. All five final debug/release emissions match. Public performance page selects89 accepted runs/933 comparisons,two page tests pass,and67-page site build passes4.145s/930,784KiB. Whole-project formatter passes with0 files/snippets/fingerprints changed.

Final guarded release pair installed atomically,without controlling the editor. SHA256 CLI1a35088f33172c9fb239fd962e2b7f52b91019aab649f1f9edb231c0e0e17a37,LSP3f8a8cc4e129bec154ec058d9e19c7f8b653256084a0643b29f413f46f19759c. Installed CLI checks unchanged hello/src/main.tsr (0.518s; short sampled peak not meaningful); installed basename(path,undefined) prints report.txt (2.076s/192,448KiB). Installed invalid callback returns a source type diagnostic (0.526s/6,304KiB). No owner source/config changes,full regressions,CI,push or publication. All section jobs are terminal;62 closed temporary/raw files (5483115bytes) pruned after verified ingestion and emission identity checks. Shared target,current native consumers,serde arguments and deferred milestone evidence retained.

**Default JSON construction — accepted locally,source `d16991ea`; parent `5e64a70d`.**
Omitted/undefined fromArray/fromEntries limits now use plain Value::Array or the shared iterator-fed build_object_plain. Removes descendant budget walks and the temporary converted entry Vec on the default object path. Explicit budgets retain root preflight and descendant checks,including counting replaced entries; the legacy hard maxDepth64 rejection is removed. Shared ordered Map::insert preserves sorted keys,last duplicate wins and moved child/key storage. No parsing,implicit child clones,receiver mutation,new dependency,configuration or generated unsafe. Invalid-limit message now says nonnegative/supported by the operation instead of falsely imposing depth64 on every JSON API. Global input-parser limits are unchanged in this section. Catalog/hover/help/reference and existing benchmark notes agree; workload sources/protocols remain unchanged.

Profile-first on unchanged native_json_containers(n2000,seed7,input5bytes): three identical baseline samples,checksum68007,18,000 allocations/deallocations,0reallocations,2,026,000 requested/0retained/885peak bytes,zero cleanup residue (21.830s/338,496KiB). Generated-only validation bypass .782708→.704541/.760333→.699750ms,unchanged allocations. Bypass plus direct entry insertion .643500/.617458ms (1.216×/1.231×),16,000allocations/1,690,000requested/838peak bytes. Five exact boundary/full identities,8warmups/3×31samples/two rotated rounds and three deterministic allocation samples/version;warm compile medians412.569→409.929→386.161ms for parent/walk/direct;load3.076,watchdog9.378s/301,632KiB. Diagnostic prototypes were not ingested or described as accepted product timings.

All132 affected cases pass,none ignored: compiler/native40 (containers5,standalone backend17,scalar construction3,namespace9,encoding4,native optional2),editor25 project_intelligence JSON cases,compiler library JSON67 (other1243 deferred). New public witness accepts256 levels and1,000,001 entries with no default quota,exact signed integer9007199254740993,omitted/undefined limits,explicit depth65 and exact node/depth boundaries,duplicate-node accounting,unchanged limit records and negative-limit errors. Existing consumption-on-success/error,borrowed-child rejection,privacy/error-code/offset and catalog examples remain accepted. Runtime units verify ownership pointers,Unicode last-key-wins and requested depth257. Native witness15.602s/975,440KiB;construction3=33.153s/712,752KiB;namespace9=54.420s/1,115,376KiB;encoding4=10.398s/896,768KiB;native optional2=7.268s/361,536KiB;standalone17=0.524s,editor25=0.539s,library67=1.052s (short runs have no meaningful sampled peak). Batched no-run build490.897s/1,588,656KiB. All binaries serial/3threads,absolute shared Cargo target,CARGO_INCREMENTAL=0,5GB30min watchdog.

Actual compiler parent/current A/B accepts the production change:containers.792959→.678125/.783083→.677042ms (1.169×/1.157×),three warm rotated compile median420.902→401.006ms. Same paths/argv,atomic replacements,original8warmups/3×31samples/two rotated rounds,all five exact boundary/full identities. Controls: schema export1.562709→1.538916/1.555375→1.533916ms,namespace.596916→.616958/.591916→.592834ms,scalar construction.395125→.390875/.435084→.385583ms; no consistent material control regression. Their compile medians418.922→415.308,384.028→379.045,314.481→312.751ms. Watchdog18.753s/332,592KiB,max load2.442. All three actual allocation samples match the direct prototype exactly:2,000 allocations and336,000 requested bytes removed,0 retained/reallocations/cleanup residue,peak885→838,input5 unchanged. Keep the measured1.16–1.17× product improvement distinct from the larger generated-only prototype gain.

Release CLI/LSP build222.037s/1,535,184KiB. Canonical native-only run2026-11-03T23:26:35.645Z passes independent result oracles and93 samples: .716917ms/2,636,570.426mean ops/s. Watchdog67.322s/374,896KiB includes enforced quiet waits,max accepted load3.896,no override. npm run store:ingest adds exactly1 history (native10→11); independently recomputed median/mean rate/count/date match,SQLite integrity OK. Final release allocation profile2026-11-03T23:26:56.979Z confirms the same16,000/1,690,000/838 counters in three identical samples (21.339s/391,360KiB). Existing Node/Rust controls reused; no fresh reference timing. All four final debug/release emissions match. Public page selects90 accepted runs/933 comparisons,two page tests pass,67-page build4.176s/1,026,288KiB. Whole-project formatter0 changed files/snippets/fingerprints. Exact new learn/data construction example executes with installed CLI and prints {"items":[42]} (8.820s/311,616KiB).

Installed validated release consumers atomically without controlling the editor. SHA256 CLIf396bfcba014466cbf9a0d628bd408bdea6f50de9cd27cb3ce7e0ebdde0a1d2a,LSP830e50e8120cd73c9bf6cfe7b824a145cc914268e41fb358d17ca8e2c9cddd81. Installed CLI checks unchanged hello/src/main.tsr (0.523s; short sampled peak not meaningful). No owner source/config changes,full regressions,CI,push or publication. All section jobs are terminal;41 closed temporary/raw files (10945729bytes) pruned after verified ingestion/profiling. Shared target,current native consumers,serde arguments and deferred milestone evidence retained.

**Trusted-root JSON serialization — accepted locally,source `50931eb2`; parent `695fe26d`.**
Profile-first generated-only probes isolated the default byte writer/checked UTF-8 overhead from container allocation. Selected a concrete typed::encode_value_unbounded(&serde_json::Value) helper using safe serde_json::to_string,called only for root JsonValue/JsonView when the converted byte budget is usize::MAX. Existing bounded Dynamic path,ordinary typed-record serializers and JSONL remain. Opaque values already contain finite numbers/valid UTF-8; no broad infallibility assumption,unsafe,new dependency or public behavior change. Constructor inlining,alternate Dynamic helper and forced helper inlining add no consistent benefit and are not retained. Earlier runtime-wide bypass added ~10% namespace Rust compile cost and was rejected.

All158 affected checks pass,none ignored,serial binaries/3threads/absolute shared Cargo target/incremental off under5GB30min: compiler JSON library68 (1243 other cases deferred),native58 (construction3,containers5,encoding5,namespace9,raw13,views5,borrowed writer4,owned writer5,export9),editor32 JSON-filtered project_intelligence cases. Batched no-run build490.053s/1,647,920KiB; pre-existing generated-only decode_plain dead_code warning remains. New witnesses compare every small byte budget against the existing native serializer,including exact codes/messages/offsets,Unicode/escapes,signed/unsigned integer limits,negative zero,borrowed views,17MiB output and once/left-to-right evaluation. Existing nonfinite/undefined and borrow invalidation/escape rejections pass. Native encoding15.065s,namespace55.403s,views28.025s,export63.261s. Whole-project formatter changes0 files/snippets/fingerprints.

Actual-compiler quiet parent/current A/B passes all five exact boundary/full identities per workload,8warmups/3×31samples/two rotated rounds,fixed paths/argv and atomic swaps. Parent→current milliseconds: containers .681792→.602917/.668375→.596333;scalar .395792→.314292/.401667→.313917;namespace .597667→.497958/.612417→.506333;export1.577875→1.306125/1.559542→1.316667. Warm compile medians398.808→396.350,317.577→329.432,383.363→384.353,428.807→424.851ms. Initial typed control was noisy (.278708→.313750/.272792→.275125). Seven-sample compile/two-round runtime confirmation: scalar314.123→321.712ms,.398042→.317792/.396958→.316125;typed control307.429→306.874ms,.270708→.272250/.304166→.271500. Final helper-variant comparison shows scalar322.165→320.993ms,namespace386.301→377.249ms,typed309.437→315.287ms;small compile differences are not consistent across repeats. No stable control slowdown. Do not claim compile speedup. Container allocations remain16,000/1,690,000requested/838peak bytes,zero retained/reallocation/cleanup residue,three identical samples/version. Main A/B19.742s/377,360KiB;confirmation11.374s/342,320KiB. Cached unchanged Node/Rust timing references reused;no diagnostic prototype ingested.

Canonical native-only release/measurement passes295.246s/1,576,576KiB. Run2026-11-04T00:25:14.093Z passes independent correctness cases and93samples/workload: containers .602333ms/3,317,416.568mean ops/s,scalar .310667ms/6,425,144.015,namespace .497375ms/4,000,611.502,export1.298916ms/3,022,163.476 (two exports/iteration). Maximum accepted load3.310,limit4,no override. npm run store:ingest adds exactly4 histories; separately computed median,mean rate,count,date and history increments match;SQLite integrity OK. All five final debug/release emissions match. Performance page selects91 accepted runs/933 comparisons;both page tests and67-page build pass (3.666s/956,704KiB). Final release whole-project formatter changes0 files/snippets/fingerprints.

Validated release consumers installed atomically,without controlling the editor. SHA256 CLI7bf156c7a9bae41f8f523af58b1210e752d9deb46fef8a1e9d29c63383ece5df,LSPd9645ce88598e058ce6c3c2ad858075c860ad50252c84cf560a5a3a8b55badc2. Installed CLI checks unchanged hello/src/main.tsr (0.521s;sampled32KiB too short to characterize memory). No owner source/config changes,full regressions,CI,push or publication. All section jobs are terminal;70 closed temporary/raw files (17483255bytes) pruned after verified ingestion/evidence recording. Shared target,current native consumers,serde arguments and deferred milestone evidence remain.
Owner hover follow-up verified against the installed release LSP directly over stdio,without touching the editor: next() returns the full ContextNext behavior/example/ownership documentation and greeting:clone(c.env.greeting) returns greeting:string. Watchdog0.522s/1,632KiB sampled,exit0. Installed LSP SHA256 matches release830e50e8120cd73c9bf6cfe7b824a145cc914268e41fb358d17ca8e2c9cddd81. Existing regression covers renamed proceed() and zero-argument signature help. Actual owner-editor hover rendering remains unverified; no additional code change needed.

**Adjacent JSON default-limit section — accepted locally,source `eb6f826b`; compiler baseline `94685609`; inherited-definition source `561c8048`.**
Native/typed JSONC now preserves omitted/undefined limits through the adapter;unbounded Limits::default uses usize::MAX budgets,not i64 sentinels. Native JSONC/JSONL parsing selects ordinary strict Serde defaults;typed JSONC and raw/typed JSONL validation keeps the existing full-document checked visitor with unbounded numeric budgets when quotas are absent. The strict validator retains Serde's own recursion guard;RawValue extraction alone is deliberately not substituted because it accepts lone surrogate escapes. Explicit limits retain exact budgets/errors;the arbitrary maxDepth64 validity restriction is removed. JSONL owned/borrowed output defaults to no byte quota. Buffered capacities remain positive with8192 default,but no extra1MiB ceiling;shared fs validation checks native size/allocation. New create_buffered_writer reserves before fs::create so failed allocation cannot truncate existing data. Data-module validation uses the same default policy. No new dependency,generated unsafe,source/config change to hello,or public method/signature.

New native fixtures pass17MiB JSONC/raw and owned/borrowed JSONL input,1,000,001 entries,100-level data,explicit65-level budgets,exact bytes/output,negative limits,strict malformed Unicode/JSON/overflow,once/left-to-right argument evaluation,capacities1,048,577,and huge failed allocation without truncation. Both65MiB owned/borrowed outputs compare exactly against Rust-side expected bytes. Boundary2 pass14.539s/775,408KiB. Existing obsolete65-depth/1MiB ceiling rejection witnesses now test actual negative/zero values. Catalog parameter help/hover,learn/data and focused editor assertions updated. Data-module validation follows the same defaults. Whole-project formatter passes0 files/snippets/fingerprints.

Initial batched build was deliberately stopped after192.978s/1,551,136KiB (terminal-15) when inspection found the shared file API64MiB write-call ceiling. No checks from that build were claimed. JSONL Output now splits an already staged record into MAX_WRITE_BYTES chunks,keeping the file API and JSONL prefix-on-I/O-error behavior. invalid_limits is generated-only after removal of the64-level guard and is annotated with that exact reason,matching parse_plain. Completed compiler/LSP/library+17integration no-run build passes492.342s/1,703,888KiB;only the pre-existing generated-only decode_plain library-test warning remains.

All373 affected checks pass,none ignored,serial binaries/3threads/absolute shared Cargo target/incremental off/per-binary5GB30min watchdog. Compiler library79 (JSON69,buffered6,data-module4),native89 (new boundaries2,data imports5,JSON4,container backend17,encoding5,namespace9,plain parse3,raw13,typed8,JSONL6,borrowed JSONL5,borrowed writer4,owned writer5,buffered backend1,buffered bindings2),editor205 (LSP library20,intelligence JSON4,project_intelligence181). JSON catalog CLI examples289.775s/1,077,968KiB;namespace54.935s;raw29.027s;typed28.518s;JSONL22.348s. Preserve these completed checks;do not restart the full pass.

The LSP library initially found a real inherited-field definition bug (19passed/1failed): checked imported generic fields carry an extends-clause diagnostic span,and apply_project_checked_facts overwrote the already resolved Base field definition with it. Shared project_generic_intelligence.rs now retains existing foreign definitions and uses checked locations only as fallback. Parser syntax/mod.rs deliberately respans external inherited fields for diagnostics;do not change that behavior. Existing inherited-help regression is unchanged and now passes all header/checked/generic/mutable/owned cases. LSP-only rebuild with the same joint package feature context13.483s/600,176KiB;library20pass1.065s,JSON intelligence4pass.550s,project181pass2.112s. No compiler/native rerun needed after this LSP-only repair.

Initial actual-compiler quiet A/B is terminal0 (109.811s/319,040KiB). Parent→first implementation milliseconds: JSONC .649583→.642125/.648667→.647750;typed JSONL21.660125→21.492000/21.625709→21.554708;owned writer61.206167→60.219875/61.729958→60.611417;borrowed reader21.887708→21.854125/21.855416→21.862583;borrowed writer60.184791→68.510417/60.656708→62.675333;raw .272417→.270833/.287334→.260167;typed control .304417→.298459/.264958→.282958. JSONC Rust compile464.547→479.726ms,raw312.588→323.304ms. Five-sample compile/two-round runtime confirmation (87.066s/354,960KiB) shows JSONC458.005→482.950ms,raw307.449→317.804ms;borrowed writer60.218625→60.772375/59.924625→61.146458. Owned/borrowed writer slow signals are not consistent across repeats,not a speedup claim.

Generated-only smaller variant compares first implementation vs shared JSONC decoder delegation plus retaining only the existing checked validation visitor (no additional false specialization). JSONC compile488.275→477.042ms,runtime .649375→.632833/.638667→.616542;raw compile313.456→308.834ms (parent309.033),runtime noisy (.308292/.252500→.281083/.260000). This removes unnecessary generic implementation work while keeping unlimited numeric budgets and the strict recursion guard. The same probe's small-record writer fast path adds no consistent benefit and is rejected;current chunk loop stays. Probe114.843s/352,432KiB. One earlier generated-only probe stopped on a missing writer-runtime replacement before timing (.515s/880KiB);no product data imported. No probe timings are ingested.

Only the two measured JSON changes are implemented in typed.rs:decode_jsonc_plain delegates to existing decode_jsonc with Limits::default(),and validate_floats reuses validate_document::<true> with unbounded limits instead of compiling another visitor. All373 earlier affected passes remain recorded,but final changed-code validation is pending. Final joint compiler/LSP/library+17integration no-run build passes493.268s/1,841,552KiB. Targeted final304 checks pass,none ignored:JSON library69,boundary2,typed8,raw13,JSONL6,borrowed JSONL5,LSP library20,project_intelligence181. Preserve69 unaffected earlier passes for373 accepted section cases total;the289s unchanged catalog sweep was not repeated. New boundary14.508s;raw29.009s;typed28.609s;JSONL21.834s;borrowed14.539s;LSP library.556s/project2.101s. No unresolved test failure.

Final actual-compiler quiet A/B passes105.668s/370,496KiB,max observed load3.023/no override,all5exact boundary/full identities/workload,original8warmups/3×31samples/two rotated rounds,fixed paths/argv/atomic swaps. Parent→final ms:JSONC .724708→.638292/.672083→.650875;typed JSONL21.620833→21.606833/21.682833→21.883708;owned writer59.518417→59.506000/62.650583→59.123083;borrowed reader22.056917→22.002083/21.972292→21.910042;borrowed writer59.298375→59.570208/60.392875→60.131125;raw .264208→.261125/.265125→.259416;typed control .298125→.284084/.273916→.279416. Rust compile medians respectively460.505→479.993,462.090→456.931,369.219→357.587,478.543→465.528,370.760→371.275,313.807→312.231,310.467→308.404ms. No stable material runtime slowdown across the repeats;JSONC compile costs ~19.5ms/4.2% more than the old implicit-budget adapter. Keep that measured cost explicit;no compile-parity or general speedup claim. Required correctness fix. A later small refinement can unify the two typed JSONC intrinsic visitor emissions while preserving absence;do not delay larger-gap optimization for this compile nit.

Canonical native-only release/measurement passes438.064s/1,747,024KiB,including an enforced quiet wait after native preparation load exceeded4. Run2026-11-04T01:41:24.214Z passes independent original result oracles/93samples per workload:JSONC .689375ms/2,649,873.165mean ops/s;typed JSONL21.431333ms/93,616.759;owned writer59.884000ms/32,636.798;borrowed reader21.577292ms/93,063.463;borrowed writer60.548167ms/32,409.318;raw .252917ms/7,959,561.246. Accepted max load3.746,no busy override. npm run store:ingest adds exactly6 histories from1file;independently recomputed medians/mean rates/counts/date/history increments match and SQLite integrity OK. Cached unchanged Node/Rust references reused;no fresh reference timing. All eight final debug/release emissions match. Performance page selects92 accepted runs/933comparisons;both page tests and67-page build pass (4.178s/1,060,592KiB).

Final allocation profile2026-11-04T01:44:22.460Z passes109.411s/351,360KiB. All six workloads have three identical samples exactly matching the baseline counters below,including allocations/deallocations/requested/peak/input bytes and zero reallocations/retained/cleanup residue. No allocation reduction or regression claimed. Profile data are not imported as execution timing.

Validated CLI/LSP consumers installed atomically without controlling the editor. SHA256 CLI373502d1e27131c05033fa8dee912cdd6fb27ec9802e0a3563c9ce10c61501a0,LSP0dd75e5aac7a84d5ec82961441ddb5bf10d2fdafcd04080ae843b1c9033e1837. Installed CLI checks unchanged hello/src/main.tsr (.521s;short sampled peak32KiB is not a memory characterization);no owner source/config changes. Actual owner-editor hover rendering remains unverified;no restart/process control. All jobs are terminal. Final whole-project formatter passes with0 files/snippets/fingerprints changed. 81 closed scratch/raw files (12918062bytes) pruned after verified ingestion/evidence recording. Current native consumers/shared Cargo target/serde arguments/deferred broad-regression evidence remain. Next profile recursive-schema trial/input/result allocations on this accepted source,then resume the object/path/normalization/decimal ranking. Full regressions/CI/push/publication remain deferred.

**Adjacent JSON default-limit baseline — source `50931eb2`.**
Public parent reproducer returns jsonc-depth:depth-limit and raw-depth:depth-limit for100nested arrays;jsonc-bytes:byte-limit and raw-bytes:byte-limit for a17MiB JSON string;default JSONL writing returns output-limit. Watchdog9.344s/325,904KiB. Standalone Rust using the exact pinned Serde artifacts accepts all five corresponding strict-JSON/input/output cases and a1,048,577-byte buffer. Value accepts127levels and rejects128/256/1024;RawValue accepts all four. RawValue also accepts a lone surrogate escape,so it cannot replace current strict Unicode validation alone. These are correctness/boundary observations,not performance timings. Rust control0.521s/1,440KiB sampled (too short for peak characterization). Buffer capacities8192 already match Rust's default;the explicit1MiB ceiling is also duplicated in fs/runtime/buffered_fs.rs,so a JSON-bridge-only change would leave it broken.

Production allocation baseline2026-11-04T00:32:02.768Z passes106.442s/378,240KiB. All six workloads use2000iterations/seed7,three identical samples each,0reallocations/retained bytes/cleanup residue. JSONC typed:checksum84007,10,000allocations/160,000requested/70peak/input59. Owned and borrowed JSONL typed each:checksum168007,20,000allocations/16,980,000requested/8,459peak/input86. Owned and borrowed JSONL writer each:checksum2007,4,000allocations/16,640,000requested/8,320peak/input86. Raw encoding:checksum100007,2,000allocations/256,000requested/128peak/input38. No profile data were ingested as execution timings.

Next implement this coherent default-limit section across native and typed paths before checks: namespace.rs currently injects a default JsonLimits record into typed JSONC;emit/json_typed.rs receives it and calls decode_jsonc. Runtime parser/normalizer,typed validation,raw borrow,JSONL Lines::new and owned/borrowed bridges all retain64 guards. Preserve caller limits,strict Unicode,exact integers,original JSONC offsets,JSONL error/terminal/resource semantics and no-truncation option validation. Match the relevant underlying Rust default instead of replacing64 with128 everywhere. Expand existing fixtures/catalog/reference/examples consistently;batch compiler/LSP/library no-run once,run affected binaries serially,then quiet A/B/current references and canonical changed-workload refresh. All earlier audit/profile jobs are terminal. 9 temporary audit/raw profile files (20263270bytes) pruned after verified evidence recording;current native consumers/shared target/serde args/deferred full milestone evidence remain.

**Next required section — remaining large gaps.**
Trusted-root serialization is accepted. Current containers .602333ms versus cached hand-written Rust .443750ms (~1.36×,.158583ms excess),Node .436750ms. Schema export1.298916/1.127833ms is now ~1.15× Rust,.171083ms excess. Do not reopen writer/constructor inlining probes without a new measured hypothesis. Default serialization has no allocation reduction;the accepted improvement removes writer/visitor overhead. Preserve exact typed undefined/nonfinite errors,explicit budget offsets,ownership and native integer accuracy.
The JSONC/raw/JSONL default-limit section and inherited-field definition failure are accepted above. Explicit parser budgets no longer have a64 ceiling,while strict validation retains Serde's own recursion guard. RawValue extraction is deliberately not used as a substitute for strict Unicode validation. JSONL stages arbitrarily sized records and chunks file writes at the existing64MiB per-operation file API boundary;65MiB owned/borrowed witnesses pass. No arbitrary buffer-capacity ceiling remains;positive/native-size/fallible-allocation checks stay. Resume recursive schema trial/input/result-copy profiling and the existing object/path/normalization/decimal ranking;malformed JSON must still throw. The ~20ms JSONC Rust compile cost is a later small refinement,not a reason to repeat these runtime probes. Full regressions/promotion remain deferred until all large gaps reach nits.

**Direct schema-export field assembly — accepted locally, parent `de0769e0`.**
Implements the measured single-field factory with direct Map::insert; four runtime and both literal/enum emitter callers now pass name/value directly. Removes only temporary field Vec/collect work; JSON Schema keywords,order,ownership,precision,error paths and source API remain unchanged. Existing8 export cases cover bounds/presence/strict fields,recursive refs,union/intersection,unrepresentable rules,nested errors and executable catalog docs. New native fixture covers repeated exports of Unicode/exact-int64/boolean/null literals and nominal string enums, exercising both emitter callers; source is formatted/token/idempotence checked by exported(). Null remains type:null, matching the existing exporter.

Complete affected section is implemented. Batched compiler/LSP no-run build passes216.501s/1,393,888KiB. All10 affected cases pass,none ignored,executed one binary at a time/3threads under5GB30min: compiler schema_export9 (63.307s/1,077,472KiB),LSP project_intelligence1 filtered export completion/hover across native/chained plans (0.523s/32KiB sampled; short run). Public literal/enum/repeated export regression passes; existing error/callback/recursive/presence/precision/catalog examples pass. Final actual-compiler quiet A/B/allocation check passes:1.797417→1.553167ms/1.807625→1.527458ms (1.157×/1.183×). Three warm rotated compile medians448.756→412.655ms improve. Original protocol/five complete result cases and full-document assertions pass; load max2.227,watchdog6.778s/327,968KiB. Recursive schema,object key order,normalization controls emit byte-identically. Allocation samples match the prototype exactly:12,000 allocations/624,000 requested bytes removed; zero retained heap,reallocations and cleanup residue; peak1,596/input102 unchanged. Final native-only release build/measurement passes244.443s/1,526,176KiB. Canonical run2026-11-03T20:29:09.456Z passes all independent result cases,93 samples:1.571625ms/2,540,458.354mean operations/s (two exports per iteration),max timing load3.138,no busy override. npm run store:ingest adds exactly1 history; median,mean rate,count/timestamp independently match and SQLite integrity is OK. Initial local independent verification incorrectly assumed one operation per iteration; corrected to manifest.operationsPerIteration=2 before accepting the row (ingestion itself was correct). All four final debug/release emissions match. Performance page selects87 accepted runs/933 comparisons; two page tests and67-page build pass (3.643s/985,264KiB). Whole-project formatter passes with0 files/snippets/fingerprints changed.

Final production profile2026-11-03T20:32:00.823Z passes22.882s/369,248KiB,three identical samples,checksum404007:38,000 allocations/deallocations,0 reallocations,4,630,000 requested bytes,0 retained/1,596 peak,input102,both cleanup counters0. Exactly12,000 allocations/624,000 requested bytes removed versus parent. Installed validated release pair atomically,hashes CLI133822244d4c8ffcab4a9f3fdb30fcefbb80e425e69bf7949f41255bfef7d7e8,LSPd7d0634bc0f34668f7aa1bd690fc1a50645095ac02ff6881e1dc68e9fa916f49 (unchanged analysis-only binary). Installed CLI checks unchanged hello source; no editor process controlled. No schema source API or documentation signature changes. The recursive schema gap is still open. No new dependency,public API,source limit or generated unsafe. Existing Node/Rust controls stay cached. Full regressions/CI/push/publication remain deferred; recursive-schema gap remains open after this export fix. All jobs are terminal;616 closed schema scratch/raw files (9085759bytes) pruned after verified ingestion/evidence recording. Current native consumers/shared target/deferred broad-regression metadata remain. The327-byte /tmp/tsrust-schema-native-args.json retains exact serde artifacts for the upcoming JSON probes; no cache hashes are guessed.

**Schema allocation profiles — profile complete, source `2536b542`; no schema product change yet.**
Baseline production profiles2026-11-03T19:54:17.086Z pass43.162s/343,728KiB,three identical samples each,zero output/complete cleanup. Recursive(n2000,seed7):checksum8007,80,000 allocations/0 reallocations,8,344,000 requested bytes,0 retained/1,898 peak,input50bytes. Export:checksum404007,50,000 allocations/0 reallocations,5,254,000 requested,0 retained/1,596 peak,input102bytes. Current recursive route uses PreparedJson/project/restore/trial visitors; source inspection must distinguish their reversible bookkeeping from required validation. Error labels are already static borrowed strings; text visitors already reuse mutable path vectors. Do not repeat those older optimizations or assume older cursor-based text fusion is faster; malformed JSON must still throw where JSON.parse throws.

Export's shared object factory builds a Vec of fields then collects a Map, while all six callers currently pass exactly one field. Generated-only single-field helper uses direct Map::insert and takes name/value arguments. Four runtime callers are typed/reference/alternatives/intersection; two emitter callers are literal const/enum in emit/schema/export.rs. All must change together. Preserves owned field values, independent objects and JSON serialization/order. No new API,dependency,source limit or unsafe is proposed. No production source is changed yet.

Original2000/seed7/8warmups/3×31samples/two rotated rounds,identical source/output/argv[0],atomic executable swaps,five complete checksum/result cases and workload full-document assertions:1.827292→1.556667ms/1.805500→1.565375ms (1.174×/1.153×). Three warm rotated compile medians450.460→420.695ms improve. Max load3.054,watchdog26.497s/327,120KiB. Exact dependency artifacts come from existing prepareNativeDependencies with the shared target,not guessed cache hashes. Allocation profile three identical samples/variant,checksum404007:50,000→38,000 allocations/deallocations,5,254,000→4,630,000 requested bytes (12,000/624,000 fewer),zero reallocations/retained heap,unchanged1,596 peak/input102,both cleanup counters0. Saved Node/Rust timing references reused; no probe imported or shown as a public optimization. First preparation failed to match the emitted module's stripped documentation comment; corrected before any measurement.

Historical export implementation plan (completed above): implement the small shared single-field object factory plus all six callers; inspect schema_export.rs for focused ownership/output/error witnesses,then batch-build selected compiler/LSP binaries once with --no-run and execute affected suites sequentially/3threads under the5GB30min watchdog. Only retain after actual-compiler quiet parent A/B,canonical native-only export measurements,ingestion/performance refresh,whole-project formatter and measured local commit. Then continue recursive schema trial/input/result-copy profiling; do not substitute an export-only fix for that larger gap. Keep full regressions/promotion deferred until the large-gap milestone reaches nits. Historical schema scripts/raw diagnostics were pruned after the accepted checkpoint above; recorded measurements remain here and exact dependency args remain in /tmp/tsrust-schema-native-args.json for current JSON probes. All jobs are terminal. No CI,push or publication.

**Normalization page indexes and scratch bypass — accepted locally, parent `0f39fb8f`.**
Source/product base equals a62da6ad. Exact UTF-8 output sizing remains rejected for now: plain counting loop probe still costs repeatable compile time (parent370.263ms,candidate380.746ms median), despite3.514667→3.108292ms/3.579958→3.099625ms. No output sizing change is retained. Fresh lookup profile instead uses derived 256-code-point page ranges for existing CCC/decomposition binary searches. Two offset arrays use491+764 u16 entries (2,510bytes), preserve every pinned row and skip empty/out-of-range pages. A third composition index adds no useful gain and is omitted. First-scalar scratch bypass processes the original scalar directly before popping actual recursive decompositions, avoiding an allocation for inert non-ASCII input.

Final minimal generated-only probe: parent3.523791→2.564875ms and3.551917→2.592459ms (1.374×/1.370×),original2048/seed7/8warmups/3×31samples,two rotated rounds,all five exact result identities. Same source/output/argv[0],atomic swaps. Three warm rotated compile medians371.733→373.249ms,overlapping ranges. Max load1.963,watchdog7.265s/223,376KiB. Node/Rust controls reused; no candidate measurement ingested. Broader three-index probe also improves1.347–1.348× with lower compile medians but is larger without added runtime value.

Implemented coherent runtime/generator/generated tables/developer README/test changes. Generator checks sorted unique keys, page coverage and u16 offsets; pinned hashes/Unicode16 data unchanged. Offline generation is idempotent,assigned ranges unchanged. New formatted O0/O3 native witness compares page/full lookups over all0..10FFFF plusu32::MAX, verifies inert non-ASCII uses only scalar/output allocations and completely cleans up. Complete existing official399,300 relationships/1,101,784 identity/ownership/error/call-order tests remain required. No API/signature/IntelliSense change,new dependency,source limit or generated unsafe. All12 affected tests pass,none ignored,one binary at a time/3threads/5GB30min: compiler string_normalize10 (209.063s/1,383,920KiB including build),LSP intelligence2 filtered normalize hover/completion/help/native examples (198.032s/1,514,272KiB including a separate compiler-dependency rebuild). New witness passes O0/O3 with unused_assignments denied,exhaustively checks2,228,226 page/full results and confirms two allocations plus complete cleanup for inert non-ASCII input; official399,300 normalization and1,101,784 identity checks pass. Batch the selected no-run compiler/LSP build next time to avoid the observed duplicate dependency rebuild; execute binaries sequentially afterward. Four unrelated controls (interfaces,numeric_string_conversion,path basename,set algebra) emit byte-identical Rust. Parent release emission matches the original saved parent consumer exactly. Final actual-compiler quiet A/B passes:3.563417→2.638667ms and3.536875→2.660125ms (1.350×/1.330×). Three rotated warm compile medians385.514→383.203ms (no regression),same original protocol/five exact result identities,load max2.996,watchdog6.231s/197,472KiB. Final native-only release/measurement passes224.915s/1,531,696KiB. Canonical run2026-11-03T19:46:17.016Z passes all independent correctness cases,93 samples:2.754167ms/750,965.031mean ops/s,max timing load2.711,no busy override. npm run store:ingest adds exactly1 history; independently recomputed median,mean rate,count/timestamp match and SQLite integrity is OK. All five final debug/release emissions match. Saved Node/Rust references reused. Public performance page selects86 accepted runs/933 comparisons; two page tests and67-page build pass (3.691s/956,000KiB). Whole-project formatter passes with0 files/snippets/fingerprints changed.

Final production allocation profile2026-11-03T19:46:52.892Z passes3.623s/279,472KiB,three identical samples,checksum332162546:36,454 allocations/15,349 reallocations,22,117 deallocations during work,1,657,294 requested bytes,567,094 retained/567,136 peak,19 input bytes,both cleanup counters0. Versus parent:8,603 fewer allocations and137,648 fewer requested bytes,unchanged reallocations/retained output,16 fewer peak requested bytes; read-only page data is2,510bytes and is outside heap counters. Installed both validated release consumers atomically,hashes CLI9a5f264443d9cb3418503b504403ba29d58d3a408b33b8a512de55988548dfea,LSPd7d0634bc0f34668f7aa1bd690fc1a50645095ac02ff6881e1dc68e9fa916f49. Installed CLI checks unchanged hello/src/main.tsr; no editor process controlled.

Next: inspect/profile native_schema_recursive and native_schema_export against matched Rust source before implementing a coherent schema pipeline improvement; separate input/tree/result copies and allocation costs. Fresh recursive gap2.713916/1.548083ms remains larger in absolute excess than normalization's new2.754167/1.636000ms (1.683×,1.118167ms excess). Then object key order and path basename; do not reopen closed Map/Set snapshot gaps. Normalization remains above saved Rust, so no parity or milestone-completion claim. Full regressions,CI,push/publication remain deferred until the large-gap milestone reaches nits. Full regressions/promotion remain deferred. All jobs are terminal;93 closed normalization scratch/raw files (20863081bytes) pruned after verified ingestion/evidence recording. Current native consumers and deferred broad-regression metadata are retained.

**Current-gap measurement refresh — source `a62da6ad`, all11 selected workloads pass.**
Native-only run2026-11-03T19:20:05.023Z passes56.157s/348,336KiB,93 samples per workload,all independent boundary/full correctness cases,quiet max load3.240,no busy override. Existing source/protocol/oracles unchanged; all Node/Rust references reused. npm run store:ingest adds exactly11 histories; independently recomputed medians,mean rates,counts/timestamp match and SQLite integrity is OK. Public performance page now selects85 accepted runs/933 comparisons, with current results in its existing tables (no before/after table). Both page tests pass;67-page build passes3.662s/916,464KiB. Whole-project formatter passes with0 files/snippets/fingerprints changed. This measurement refresh exposes accumulated gains from earlier commits, not solely the last set change.

Current native/Rust milliseconds: map_values0.429625/0.543625,map_group_by2.249709/2.371708,map_for_each0.528041/0.558917,iterable_array_spread2.159625/2.161083,set_for_each0.430417/0.417667. These no longer justify large-gap optimization work. Remaining ranked signals: string_normalize_forms3.763417/1.636000 (2.300×,2.127417ms excess),native_schema_recursive2.713916/1.548083 (1.753×,1.165833ms),object_keys_order1.796417/1.086375 (1.654×,.710042ms),native_schema_export1.831208/1.127833 (1.624×,.703375ms),std_path_basename_ascii.866125/.431000 (2.010×,.435125ms),iterable_call_spread1.703666/1.272250 (1.339×,.431416ms). Decimal parser remains5.592875/4.178834 (1.338×,1.414041ms excess), measured in the preceding accepted run. Distinguish inherent owned-output/API overhead from removable work before implementation; no fresh hand-written Rust or Node timing was performed.

Normalization generated-only profiles against this unchanged source are now complete; no normalization product change or public candidate measurement is accepted yet. Same source/output/argv[0], atomic executable swaps,8warmups/3×31samples/two rotated rounds and all five exact correctness identities. Stack bypass processes the first scalar directly instead of pushing/popping it:3.525083→3.273458ms/3.533833→3.288000ms. Output capacity from input bytes:3.232666/3.242250ms. Combined:3.511667→3.002209ms/3.504042→3.018167ms (1.170×/1.161×), while stack+exact UTF-8 output sizing reaches3.085750/3.086625ms (1.138×/1.135×). Exact sizing makes an additional pass over decomposed scalars but avoids reserving the original, potentially much larger input size after NFC contraction. Three rotated warm compile medians374.303ms parent,376.447ms combined,380.813ms exact; small compile-cost differences need confirmation or a direct size-counting loop before acceptance. Combined run load max2.035,watchdog8.820s/179,520KiB; first separate run10.902s/231,152KiB,load2.752.

Allocation diagnosis uses three identical samples/version,checksum332162546,canonical unused input19bytes,zero output/complete cleanup counters. Parent45,057 allocations/15,349 reallocations,1,794,942 requested/567,094 retained/567,152 peak bytes. Stack bypass36,454/15,349,1,657,294 requested/567,094 retained; input-byte output sizing45,057/6,764,1,674,832 requested/538,804 retained; combined36,454/6,764,1,537,184 requested/538,804 retained/538,846 peak. Stack+exact sizing36,454/5,742,1,475,739 requested/489,539 retained/489,581 peak:8,603 fewer allocations,9,607 fewer reallocations,319,203 fewer requested and77,555 fewer retained bytes. Watchdog3.115s/182,704KiB. Initial diagnostic invocation supplied seed7 as the optional input (1byte) rather than the unused canonical text; corrected before these final profiles; work counters/checksum were unchanged. No probe data ingested; existing Node/Rust controls reused.

Historical normalization implementation plan (superseded by the accepted section above; output sizing was not retained): finish the coherent normalization section using the simple first-scalar scratch bypass and sized output, with exact sizing preferred to avoid over-reserving contracted NFC strings. A plain size-counting loop may remove the mapped-sum compile overhead; profile that before the final implementation choice. Require a quiet actual-compiler parent A/B and no material compile/memory regression. Run the complete affected Unicode conformance suite with its existing official/identity/ownership/error witnesses after the coherent source change, native-only production measurement/ingestion/page refresh only if accepted. Revisit larger recursive schema/object-key/export/path signals afterward; leave closed Map/Set snapshot gaps alone. Broad regressions/promotion remain deferred until these large gaps are closed. Current measurement log /tmp/tsrust-gap-refresh.log; raw is disposable after verified ingestion. Previous37 decimal scratch/raw files16,864,051bytes and30 set scratch/raw files10,967,515bytes were pruned after their accepted checkpoints; current native consumers/shared target/deferred broad-regression metadata are retained.

**Lazy small-set cross-key hashing — accepted locally, parent `8df00126`.**
Profile-first: set_algebra/relations emission eagerly computes set_key.tsr_hash() before has_with/matches_with even when the queried set scans at most8 slots and ignores the hash. Generated-only per-call guards improve runtime but add repeated emitter branches. Smaller shared-helper probe moves hash selection inside the two runtime helpers and removes eager hash arguments from all seven emitted call sites; existing cross-key comparisons/order/normalization remain. Original protocols,8warmups/3×31samples/two rotated rounds, same source/output/argv[0], atomic executable swaps, all five exact boundary/full identities equal: algebra2.079917→1.873042ms/2.057500→1.870500ms; relations0.565083→0.438334ms/0.546792→0.433792ms. Three warm rotated compile medians algebra549.847→550.348ms,relations479.591→481.023ms (noise). Load max2.748,watchdog13.497s/186,528KiB. Saved Node/Rust references reused; no probe ingested.

Implemented shared runtime has_with/matches_with lazy hashing with U:TsrSetKey, emitter callers and focused O0/O3 hash-count/collision/sparse-index/clear witnesses. No public API, default limit, dependency, allocation, ownership, IntelliSense signature or source workload change; no generated unsafe. All13 affected tests pass, none ignored, one binary at a time/3threads/5GB30min: algebra4 (208.674s/1,382,976KiB including build),relations3 (3.688s/246,976KiB),sets6 (3.145s/634,992KiB). Native O0/O3 witness covers0/1/8/9/17 slots, forced collisions, sparse indexed storage after deletion and reset after clear; formatted source round-trips tokens/idempotence. Final actual compiler emits byte-identically to both shared-helper probes. Four controls (set_for_each,set_entries,map_values,map_group_by) differ only in unused helper definitions; source behavior/call emission is unchanged. Instrumented complete workload diagnosis removes28,672 algebra and18,432 relation hashes per2048-item batch (zero remaining), preserving checksums594525782/203127518 and full exact outputs; watchdog3.113s/176,688KiB. Final actual parent A/B: algebra2.073875→1.867417ms/2.052125→1.876000ms (1.111×/1.094×),relations0.578459→0.453792ms/0.564750→0.444250ms (1.275×/1.271×). Three rotated warm compile medians563.786→568.634ms (+0.86%) and488.839→491.632ms (+0.57%); ranges overlap and prior exact-source run549.847→550.348ms/479.591→481.023ms establishes ordinary run variation, no material compile regression. Max load3.848 within runner quiet threshold4,watchdog12.463s/207,184KiB. All five exact correctness cases match. Final native-only production measurement passes227.545s/1,475,296KiB. Run2026-11-03T19:17:11.275Z: algebra1.897834ms/1,078,047.635mean ops/s,relations0.460542ms/4,456,937.617ops/s,93 samples each. All independent correctness cases pass; max timing load2.695,allowBusy=false. npm run store:ingest adds exactly2 histories; six-decimal medians,three-decimal mean rates,counts and timestamp independently match; SQLite integrity OK. All six final debug/release emissions match. Public performance page selects84 accepted runs/933 comparisons; two page tests and67-page build pass (3.646s/942,928KiB). Whole-project formatter passes with0 files/snippets/fingerprints changed. Installed release pair refreshed atomically, hashes CLIed023b8d711430672dece3864e8b5c7d248c2578733c55c55cf1adf57e65cbca,LSP51e88cf22e7df3c5e326f7061e68fc6d10c9e40c3134d7f022a12293803ba93e; installed CLI checks unchanged hello source, no editor process controlled. Current production measurements4.198/1.391ms for these workloads are stale versus fresh parent2.06/.56ms after earlier optimizations; refresh both without claiming all accumulated gains from this small change. Full regressions/promotion remain deferred. Next: native-only refresh of the remaining11 historically large-gap workloads (normalization,map groupBy,iterable spreads,recursive/export schemas,object key order,map values/forEach,set forEach,path basename) before choosing the next optimization. Current timings can already be much lower than old selected rows; do not optimize a stale gap. Node/Rust references stay cached. Two generated-only normalization alternatives are prepared but not measured or implemented (/tmp/tsrust-normalization-*): avoid the first scratch-stack push/pop for each input scalar, or size UTF-8 output from input byte length; no acceptance claim. Scratch /tmp/tsrust-set-* can be pruned after checkpoint; no new handoff.

**Decimal parser — accepted locally, parent `529647e3`.**
Source comparison: Number(text) trims the documented conversion whitespace, handles radix/Infinity, then manually scans decimal grammar before Rust f64 parsing scans again. Hand-written reference trims and calls f64::from_str, with correctness-only spelling differences excluded from matched timing. Generated-only removal of the duplicate scan retains a leading decimal digit/dot guard to reject Rust-only inf/infinity spellings. Quiet original500000/seed7/8warmups/3×31samples/two rotated rounds:6.286625→5.366250ms and6.261208→5.549417ms (1.171×/1.128×); three warm rotated compile medians306.357→299.389ms. All five boundary/full identities match, load max2.7974, watchdog7.803s/182,272KiB. Node/Rust references reused. Probe is not a production measurement.

Implemented the minimal shared runtime change (remove duplicate grammar scanner, preserve trim/radix/exact Infinity and delegate decimal validation/rounding to Rust), plus O0/O3 native grammar/special-spelling/rounding/borrow witnesses. Callers cover ordinary Number, unary plus and enum/union conversions through the same helper. No API/editor signature/docs/source workload change, new dependency, allocation or generated unsafe. All16 affected tests pass, none ignored, three threads and5GB/30min watchdog: numeric_string_conversion7 (211.744s/1,337,632KiB including build), numbers7 (4.717s/512,656KiB), scalar_unary_plus2 (2.083s/254,672KiB). New native witnesses format/token/idempotence round-trip and execute at O0/O3. Final actual-compiler quiet A/B against529647e3: decimal6.178291→5.370125ms and6.073875→5.261709ms (1.150×/1.154×); mixed numeric0.192583→0.190667ms/0.187250→0.189583ms stays within observed noise; unary plus0.432333→0.409042ms/0.438584→0.406000ms improves. Unaffected number_from_int emits byte-identically (its0.329–0.367ms timing spread confirms measurement noise). Compile medians decimal296.127→289.964ms,mixed400.441→397.499ms,unary391.640→384.276ms,unchanged control212.631→212.917ms. Original protocols, same paths/argv[0], atomic replacements, three warm rotated compile samples/version, two rotated runtime rounds, all five boundary/full identities equal. Load max2.8345, watchdog16.142s/185,552KiB. Node/Rust references reused. Final release/native-only production measurement passes229.383s/1,475,008KiB. Run2026-11-03T19:03:06.040Z passes all independent correctness cases,93 samples each: decimal5.592875ms/89,500,152.323mean ops/s,mixed numeric0.196292ms/10,323,743.715ops/s,unary plus0.419875ms/4,890,758.981ops/s. Timing max load3.21875,allowBusy=false. npm run store:ingest adds exactly3 histories; independently recomputed medians/mean rates/count/timestamp match and SQLite integrity is OK. Existing Node/Rust controls reused. Performance page selects83 accepted runs/933 comparisons, both page tests pass,67-page build passes3.700s/989,408KiB. Final debug/release emissions are byte-identical on all four A/B workloads. Whole-project formatter passes with0 files/snippets/fingerprints changed; accepted as a local measured source checkpoint. Installed both validated release consumers atomically; hashes CLI61bb49bf9544f92ec113c8e55b640ab53391f31b0b25b554cf523362bcad5c7e,LSPa3cd1e1fbf08da00b92dfa2e01e92ef801d435d8dfe34859b2b4080c8fdf1f43. Installed CLI checks unchanged hello/src/main.tsr. No editor process was controlled. Scratch /tmp/tsrust-decimal-* can be pruned after checkpoint. Full suites/CI/push/publication remain deferred. Next: inspect/profile set_algebra and string_normalize_forms against current comparable references before implementing; keep conservative safe Rust and ordinary source semantics.

**Constant counted builders — accepted locally; parent `ad4da4ba`.**
`emit/counted_builders.rs` proves a fresh empty local String array, a fresh zero local int counter, stable readonly array-length bound, exactly one constant-label append and a unit increment, with only infallible bounded reads/constant String conversions/concat. Reuses constant_string knowledge and the shared expression walker (crate visibility only). Captures/hidden named captures, aliases/reassignment, unsupported control/exception storage and fixed layouts retain original loops. Builds ordinary Array::Small through4 elements and Array::Heap(vec![borrowed_label;count]) above4; preserves the original checked bound and final counter. Remaining-count/initial-destination reads and final comparison retain clean read accounting. No source API, dependency, unsafe generated code, public ABI change or new source limit. Files emit/mod.rs, emit/counted_builders.rs, fixed_storage.rs, tests/counted_builders.rs.

All145 affected cases pass across resumed groups, none ignored, each binary separately with3 threads and5GB/30min watchdog: counted4, fixed_storage34 (24.486s/529,328KiB), arrays12 (5.229s/429,728KiB), owned_arrays12 (5.739s/394,208KiB), nested_arrays12 (6.227s/386,192KiB), literal spread11 (5.201s/576,208KiB), call spread13 (6.296s/622,432KiB), nullable4 (4.686s/387,776KiB), union arrays7 (3.148s/625,856KiB), exceptions22 (6.256s/621,520KiB), string concatenation2 (2.084s/254,912KiB), padding7 (4.182s/543,936KiB), integer fusion5 (4.271s/639,952KiB). Native witnesses cover zero/one/four/five/17 rows, exact values/Unicode/final counter, zero small-result allocations, complete cleanup, nonzero starts/steps, existing items, wrong indices, calls/effects/captures and signed length limits. Both actual spread workloads select exactly2 builders. Final4 pass5.212s/458,208KiB, including O0/O3 emitted-Rust compilation with unused_assignments denied for both real workloads. Preserve141 earlier passed cases after the read-only diagnostic repair.

Fixture repairs: vec![TsrNull;usize::MAX] clones at O0 and hit the30s witness timeout; use documented valid raw-parts ZST instrumentation (generated code is asserted unsafe-free). Panic checking originally required unrelated Array:Debug; removed that fixture bound. A final diagnostic check exposed new dead assignments after loop removal; repaired read accounting and added strict warning witnesses. Repair build205.511s/1,422,160KiB. Existing unused-label/zero-arity-comparison warnings are unchanged; no blanket warning suppression was added.

Quiet final actual-compiler A/B against exact parent consumers, identical source/output/argv[0], atomic swaps, original3×31 samples,8 warmups,two rotated runtime rounds,three warm rotated compile samples/version,five equal boundary/full results: literal0.059083→0.041667ms/0.058333→0.041750ms (1.418×/1.397×); call0.058666→0.041666ms/0.059417→0.041417ms (1.408×/1.435×). Compile medians277.735→280.964ms and280.936→278.761ms overlap; earlier repeats284.433→279.549ms/285.417→279.117ms. No material compile regression. Final watchdog7.783s/165,584KiB,max load2.239. Four controls remain byte-identical: nullable bool/int, conditional string arrays and interfaces. Final debug/release emit byte-identically on all six cases.

Final-source canonical run2026-11-03T18:44:06.962Z completes225.287s/1,473,440KiB, native-only, all five independent oracles/full results pass,93 samples each: literal0.045042ms/45,093,258.092mean ops/s,call0.044708ms/45,482,124.364ops/s. Actual timing maximum load2.889, no override. Saved Node and Rust references are reused, not retimed. Exactly2 ingested histories match six-decimal medians/three-decimal mean rates/count/timestamp; SQLite integrity OK. Provisional18:10 run was removed from the two uncommitted histories/config and its raw file deleted before final ingestion. No provisional/rejected records are retained as accepted measurements.

Final allocation profile18:45:15.192Z passes4.130s/249,584KiB,three identical samples each,checksum752386989:1,367 allocations/10 reallocations unchanged; requested1,113,824→1,015,472bytes,retained/peak688,256→589,904bytes (98,352 fewer),19 input bytes,zero cleanup. Performance config now selects82 accepted runs/933 comparisons; public tables show current values with existing Node/Rust references, no before/after table. Both page tests pass. Final67-page website build passes3.679s/976,704KiB; whole-project formatting passes with0 files/snippets/fingerprints changed. Full regressions, CI, push and publication remain deferred.

Next: re-rank remaining large gaps from current selected SQLite measurements and inspect comparable source before profiling. Current largest comparable absolute signals include number_parse_decimal6.377750/Rust4.178834ms, set_algebra4.198209/2.032833ms, string_normalize_forms3.787125/1.636000ms, map_group_by4.175791/2.371708ms, iterable_call_spread2.901292/1.272250ms. Confirm protocol/work equivalence and current source first; do not blindly repeat older rejected trials or treat historical rankings as current. Start with decimal parsing unless source comparison changes that ranking. Spreads now~1.40× faster; remaining~1.4× saved-Rust difference is mostly result representation and can be revisited after larger signals. Do not run full suites or promote until the owner-defined large-gap milestone reaches nits. Installed consumers now include accepted counted builders and decimal parsing as recorded above; never control/restart owner editor processes. Active evidence /tmp/tsrust-counted-*. Counted-builder checkpoint529647e3 is committed;92 closed scratch files/15,970,083bytes pruned. Preserve deferred broad-regression metadata. Active decimal work is described above.

**Consecutive push batches — rejected; only an obsolete object-array fixture is retained.**
Implemented independent infallible scalar-expression proofs, scheduled2–4-item batches with arguments before receiver borrowing, and Heap Vec::extend/Small existing-push fallback. All130 affected cases pass across resumed groups, none ignored: new batching3 plus fixed_storage34 (22.929s/549,024KiB), arrays12 (5.226s/301,632KiB), owned_arrays12 (5.736s/512,944KiB), nested_arrays12 (6.245s/504,912KiB), array_literal_spread11 (5.198s/469,104KiB), call_spread13 (5.793s/643,984KiB), nullable_unions4 (4.660s/427,520KiB), union_array_elements7 (3.136s/629,888KiB), exception_handling22 (6.226s/563,344KiB). Final new3 build+tests204.457s/1,493,264KiB, O0/O3 runtime checks pass; extended once-only-drop witness through heap reallocation passes4.147s/396,224KiB. Preserve completed evidence; no broad/full run occurred.

Final quiet actual-compiler A/B, same paths/argv[0], atomic swaps, original3×31 samples, two rotated runtime rounds and three warm rotated compile samples/version, five correctness cases equal. Boolean0.015959→0.009292ms and0.015541→0.009292ms; int0.023750→0.014625ms and0.022875→0.014791ms. But compile medians251.820→260.374ms (+3.40%) and258.924→268.529ms (+3.71%) reject the trial. Controls have no useful gain. Load maximum2.492, watchdog21.785s/217,184KiB. Generated-only alternatives also fail the compile-cost condition: one direct Small/Heap promotion method245.581→261.387ms (+6.43%), runtime0.015000→0.008666ms/0.014667→0.008750ms (5.159s/151,520KiB); one outlined batch helper245.655→253.835ms (+3.33%), runtime0.014875→0.010875ms/0.015875→0.011791ms (4.650s/206,624KiB). Plain/methodless preliminary probes run faster but do not establish acceptable compile cost. All production batching code and trial-only witnesses reverted; no rejected measurements are ingested or shown publicly. Existing Node/Rust controls were reused, not retimed.

Affected validation found owned_array_contexts_match_element_types_exactly still rejected `const a=[{n:1}]`, contradicting accepted anonymous object inference. Replaced that obsolete rejection with native output1; named missing-field/scalar mismatch rejections remain. This is the only retained source change. Final restored-source owned_arrays12 pass204.140s/1,325,968KiB (including rebuild), none ignored, three threads. Debug CLI emission matches accepted release4d97c4d2 byte-for-byte on all six controls: nullable boolean/int, conditional string arrays, both exact nullish spreads and interfaces. Whole-project formatter passes with0 files/snippets/fingerprints changed. npm run store:ingest adds0 entries from0 files; no counterfactual is imported. Installed release4d97c4d2 still matches current product sources; no installed refresh is needed for this test correction. No new performance records/CI/push/release.

Next: profile exact array_literal_spread_nullish_packs and call_spread_nullish_packs. Nullable dispatch remains a measured, explained gap with rejected inlining/private-Vec/batching tradeoffs; do not repeat these trials. Inspect widening conversion and one-element carrier construction directly; preserve safe Rust and public ABI. Full regressions remain deferred until the large-gap milestone. Scratch evidence /tmp/tsrust-push-batches-*, /tmp/tsrust-growing-* and /tmp/tsrust-nullable-batch-* can be pruned after durable evidence/checkpoint. Preserve deferred broad-regression metadata.

**Historical nullish spread profiles — superseded by accepted constant counted builders above.**
Exact parent emissions contain zero-sized null/undefined input packs and two loop-built String rows per iteration. String label concatenations are already folded. Generated-only compact Vec<Vec<String>> rows without capacity are slower: literal0.059208→0.082459ms/0.058959→0.082250ms; call0.058583→0.081375ms/0.059125→0.081042ms. The small-row conditional_array_string control also regresses0.120250→0.132750ms/0.124583→0.129291ms. Do not implement blanket private-Vec row storage.

The smaller capacity-only probe preserves Vec<Array<String>> and replaces the two empty String row initializers with Array::Heap(Vec::with_capacity(source_pack.len())). Literal0.059000→0.045458ms/0.058791→0.044792ms (1.298×/1.313×), call0.058791→0.044709ms/0.058167→0.044708ms (1.315×/1.301×). Warm rotated compile medians272.262→264.606ms and272.489→265.776ms improve. Quiet max load2.411, watchdog9.354s/153,488KiB. This is the simplest dependency-ready optimization to implement first; no source API, public ABI, new runtime method or driver change is needed.

Compact rows plus exact inner capacities reach near saved Rust times: literal0.060583→0.032916ms/0.059292→0.032959ms (1.840×/1.799×), call0.059458→0.033542ms/0.059041→0.033542ms (1.773×/1.760×). Compile medians271.807→250.864ms and271.582→248.096ms improve7.71%/8.65%. Quiet load1.545, watchdog9.322s/155,664KiB. This larger private-layout follow-up is not implemented and must reuse closed fixed_storage/rows proofs with ordinary external adapters and benchmark ABI support; do not apply it to arbitrary small/fallible row factories.

Fresh allocation diagnosis4.667s/156,480KiB, three identical samples per workload/variant, checksum752386989, both cleanup counters0. Both workloads: parent1,367 allocations/10 reallocations,1,113,824 requested bytes,688,256 retained/peak; compact rows without sizing2,733/1,376,721,056 requested,491,712 retained; capacity-only2,733/10,1,048,256 requested,622,688 retained; compact+sized2,733/10,393,216 requested,295,008 retained. Allocation count alone is misleading: sizing removes1,366 reallocations from the naive compact alternative, while compact storage reduces retained bytes. All timing probes use original3×31 samples,8 warmups, two rotated rounds, identical source/output/argv[0], atomic executable replacement, warm rotated compile samples and all five boundary/full result identities. Saved Node/Rust references were reused. No probe is an accepted compiler benchmark or public metric.

Next executable action: trace emitter loop scheduling, existing fixed_storage proofs and known-string folding; implement a proved counted array builder. Keep ordinary Array ABI first. Require an initially empty owned local result, a zero-based unit-step loop bounded by a stable checked array length, exactly one append per iteration and infallible/effect-free element construction (or equivalent proof). Evaluate the original checked bound before sizing; preserve all source effects, checked indexing, arithmetic and failure order. Avoid the earlier rejected preallocation before outer checked arithmetic. Exclude captures, aliases, reassignment, unsupported control/exception storage and unproved loops; preserve small/fallible factories. Add native boundary/side-effect/overflow/ownership witnesses before batched affected checks. After implementation, quiet A/B against this checkpoint including small-array controls and consumer compile cost, then native-only canonical measurements with existing Node/Rust controls, ingestion and public performance refresh if accepted. Full regressions/promotion remain deferred.

Active scratch: /tmp/tsrust-dense-rows-*, /tmp/tsrust-rows-capacity-only-*, /tmp/tsrust-row-allocation-* plus three parent generated consumers /tmp/tsrust-growing-{array_literal_spread_nullish_packs,call_spread_nullish_packs,conditional_array_string}-parent.rs and /tmp/tsrust-growing-ab-metadata.json. These are disposable profiling tools/consumers, not handoff records. No owned worker is running. Prior scratch preparation failures were missing optional sentinel boilerplate and an unadapted inferred array-literal constructor in the conditional control; corrected before recording any results.

**Private Vec growth trial — rejected; source restored to `b39adc75`.**
Implemented fixed_storage local proofs and ordinary Array/NumericArray return wrappers. AST loop membership is required: statement spans identify keywords, not full bodies. The first suite exposed that proof defect and an invalid captured-array fixture; repaired both. Final native semantics pass at O0/O3; all36 fixed_storage cases pass across resumed groups (35 in227.542s/1,525,360KiB plus repaired representation-only witness5.196s/324,992KiB). Record arrays retain existing RecordArray aliases. Builds208.114s/1,331,696KiB and final CLI205.171s/1,553,296KiB. These tests establish the rejected trial only, not a retained compiler change.

Quiet actual-compiler A/B versus previous commit, same source/output/argv[0], atomic swaps, original3×31 samples, two rotated rounds, all five boundary/full result identities equal. Nullable boolean0.015959→0.007750ms and0.016000→0.007750ms (2.059×/2.065×); nullable int0.023792→0.015250ms and0.023875→0.015250ms. But conditional_array_string regresses0.132250→0.139042ms (5.14%) and0.126708→0.136541ms (7.76%): its short helper arrays lose inline storage. Interfaces stays~0.141ms; nullish spreads show no useful gain. Boolean compile median252.020→240.794ms, int262.307→249.349ms; runtime regression rejects the whole trial. Max load3.266; watchdog19.726s/215,168KiB. No Node/Rust retiming, ingestion or public metric changes. Source changes and trial-only witnesses reverted; installed release consumers remain accepted4d97c4d2. Debug CLI/test artifacts currently contain rejected code and must not be used for acceptance until rebuilt.

Next alternative profile: generated-only four-element push batching preserves inline storage. A tiny inline Heap/Small branch extends a heap Vec once; its cold helper uses existing per-item pushes for Small. No source/ABI change yet. Three quiet rounds, same path/protocol/results:0.015125→0.008959ms,0.014875→0.008958ms,0.014916→0.008750ms (1.661–1.705×). Load2.374; watchdog4.681s/157,776KiB. Preliminary compile median253.356→258.906ms needs a proper rotated comparison; do not accept a repeatable compile regression. Next: implement consecutive independent infallible push batches through the existing statement scheduler and Array runtime, retaining ordinary individual pushes for receiver-dependent/effectful/fallible values, captures, fixed/private layouts and exception storage. Include small-array allocation and operand/failure-order witnesses, then affected suites and matched controls. Scratch evidence /tmp/tsrust-growing-* and /tmp/tsrust-nullable-batch-*; never ingest rejected/counterfactual timings as compiler measurements.

**Nullable boolean profiling — source `4d97c4d2`, no production change yet.**
Fresh production allocation profile2026-11-03T15:54:16.521Z passes three identical samples: checksum724599172, one allocation/ten reallocations,393,024 requested bytes,196,608 retained/peak bytes, zero cleanup residue. Watchdog3.628s/251,712KiB. Values are already borrowed string literals; string allocation is not the cause.

Generated-only counterfactuals isolate local output storage from the ordinary array ABI. Same source/output paths and executable argv[0], atomic executable replacement,2048 items/seed7,8 warmups,3×31 samples per variant per round, three rotated rounds; all checksum/exactBits results match. Baseline medians0.014917/0.015166/0.014792ms; preallocated Array::Heap0.013500/0.013542/0.015375ms; private local Vec with ordinary Array::Heap return0.007292/0.007250/0.007292ms (2.046×/2.092×/2.029×). Vec plus preallocation0.006292/0.006417/0.006292ms. Four-element batch extension0.006750/0.008417/0.006708ms is inconsistent and unnecessary. Quiet load maximum2.678; final watchdog8.249s/148,288KiB. Preliminary compile medians251.037ms baseline,242.813ms Vec; baseline first sample was cold2s, so these are diagnostic figures, not an accepted compile-cost comparison. No Node/Rust reference retiming, SQLite ingestion or public performance update: compiler behavior has not changed.

Historical plan (implemented and rejected above; follow counted-builder sizing instead): prove private heap-backed local array storage for an initially empty append result, reusing fixed_storage plans/proofs and native Vec operations. Preserve Array return/call ABI through the existing Heap wrapper; reject unsupported alias/capture/escape/reassignment uses rather than changing public storage. Start with the exact nullable workload, then representative growing boolean/string/record controls. Avoid automatic capacity reservation: it changes allocation/error timing before loop arithmetic and is not needed for the main gain. Do not repeat rejected global push inlining or cold-helper trials. Implement complete emitter/proof/ownership/drop/exception witnesses before affected suites; only accept after quiet matched parent A/B and consumer compile checks. Profile scripts/results remain /tmp/tsrust-nullable-isolate*; fresh generated allocation consumer is target/runtime-comparisons/profiles/nullable_boolean_undefined.rs. These probe executables do not replace canonical timing consumers.

Latest completed optimization: fixed private return arrays for discarded nested
forEach results. Final matched A/B improves10.602×, repeat7.551×, removes6,000
allocations/1,248,000 requested bytes per2000-item batch, with slightly lower
consumer compile cost. All88 affected tests pass. Local existing performance
tables select81 accepted runs:636/933 Tsrust wins over Node,median1.60×.
No before/after table. Full regressions/promotion remain deferred; next review
nullable boolean and nullish spreads, not the now-closed nested-result gap.

**Fixed private nested-array returns — accepted locally, parent `e5189463`.**
The borrowed-range helper returns a fixed Rust array when every return is a
same-width nested-array literal within the existing4KiB stack budget. Its ordinary
call wrapper uses the existing From conversion to preserve the owned source-array
ABI. Callback trait adapters also retain that ordinary ABI. The existing storage
bound supplies conservative element sizes; width analysis is cached once per
callback. Spread/nonliteral/mixed-width/oversized returns, local exception syntax
and registered Result error layouts retain ordinary result storage. Existing
range/source/ownership/task/counting/native/scoped guards remain. No new runtime
storage class/template, source limit, dependency, public API or generated unsafe.
Source: `emit/foreach_slices.rs`, `emit/mod.rs`, shared fixed-storage budget
visibility, `tests/array_for_each.rs`.

Profile-first finding: call_range already inlines; more inlining is not the cause.
Its discarded outer Vec materializes two compact inner-array payloads (208bytes
per visit), with memcpy/drop work. Generated-only fixed-outer return runs8.066×/
7.806× faster and removes6,000 allocations; four boundary cases/full exact results
match. Alternative one-slot storage runs8.029×/7.766× but changes general storage;
not implemented. Heap-only strings increase12,001→18,001 allocations and run only
1.023×/1.107×; not implemented. These probes are not public measurements.

Final ABI-repair build210.971s/1,419,280KiB. All88 affected tests pass, none
ignored, one binary at a time, three threads,5GB/30min watchdog: forEach11
(7.329s/495,696KiB), slice7 (8.903s/479,360KiB), reduce7
(4.231s/463,776KiB), fixed storage34 (20.869s/560,352KiB), declared functions5
(10.399s/554,656KiB), exceptions22 (5.234s/644,192KiB), callback errors2
(1.038s/233,328KiB). New O0/O3 witnesses cover retained ordinary/forwarded callback
returns, empty/differing widths, spread fallback, typed exceptions, oversized
fallback, allocation upper bounds and cleanup after partial element construction.
Initial witnesses read a mutable capture before scope exit, used an ineligible
base.length slice argument, and used unsupported catch annotations; repaired
fixtures without weakening guards. Final review also found native_result shadowing
could leak the private layout into callback traits; repaired to body_result and
added forwarded-callback coverage before acceptance. Earlier passed groups were
retained; six unrun groups resumed after the affected array suite passed.

Quiet final A/B against exact parent, original3×31 samples,2000 items,seed7,
8 warmups, identical source/crate/output paths and executable argv[0], atomic
executable swaps:0.493458→0.046542ms (10.602424×), repeat0.404916→0.053625ms
(7.550881×). Three compile samples/version: median wall229.830→226.482ms,
user273.746→270.197ms; no compile regression. Max load2.338,
watchdog4.154s/153,120KiB. Final candidate consumer restored.
All five controls emit byte-identically in final debug/release compilers:
discard-string/record/callable and void int/recursive-record. Final nested emission
exactly matches the consumer from the canonical run collected before the final
trait-adapter repair; therefore retain that run instead of rerunning Node/timing.

Final release + production profile220.241s/1,593,648KiB. Three identical samples,
checksum131080745: allocations12,001→6,001,requested bytes1,374,040→126,040,
peak275→67, no reallocations, retained/input bytes or cleanup residue. Work
allocation/deallocation counts both6,001. Ordinary entry points still materialize
owned results when retained; only the private discarded-result path avoids the
outer heap container.

Canonical run `2026-11-03T11:49:22.841Z`,223.266s/1,193,056KiB; all five
independent correctness cases and complete timed results pass,93 samples/engine.
Native0.024917ms/78,504,925.441mean ops/s,Node0.220625ms/8,945,437.116ops/s.
Actual timing max load2.016, no override. Exactly2 histories ingested and matched
to raw medians/mean rates/counts/timestamp; SQLite integrity OK. Unchanged saved
Rust0.122583ms is reused, not retimed. This case is now faster than that ordinary
Rust reference; it does not establish a universal Rust lower bound. Existing
performance tables select81 runs:636/933 Tsrust wins,621/933 Rust wins over Node,
median1.60×. Two page tests pass;66-page site builds3.641s/988,064KiB.

Whole-project formatter passes token/idempotence checks with zero source, snippet
or fingerprint changes (4.128s/201,568KiB). Pruned45 closed scratch/raw files
(10,531,028bytes) after exact verification. Current production timing/profile consumers
remain, all owned workers terminal. No broad regressions, remote CI or push.

Superseded continuation: nullable dispatch and exact nullish spread profiling are now complete above. Global inlining, private Vec and batched-push trials were rejected; follow the counted-builder sizing plan. Stored original nullable0.014833ms/Rust0.003625ms and spread0.063709/0.032208ms,0.063166/0.032000ms remain accepted measurements until a new compiler change passes acceptance. Full suites, CI, push and deployment remain deferred until remaining large gaps are addressed/explained and only nits remain.

**Borrowed forEach slices with owned results — accepted locally, parent `8ec408a9`.**
`emit/foreach_slices.rs` removes the void-result-only restriction, keeping Task
results excluded. All existing source-read, capture/move/mutation, native/scoped,
counting/optional/rest, fixed-layout and exception-region guards remain. Existing
call_range return signatures and immediate-drop statements already preserve
owned results, ordinary callback entry points and drop-before-next-invocation.
`tests/array_for_each.rs` adds O0/O3 range witnesses for nested arrays, records and
owned callables, plus an ordinary call retaining a returned nested array. No
public API, ownership rule, runtime template, dependency, limit or unsafe change.

Build198.366s/1,711,712KiB. All87 affected compiler/native tests pass, none
ignored, one binary at a time, three threads,5GB/30min watchdog: forEach10
(5.182s/469,232KiB), slice7 (8.290s/442,432KiB), reduce7
(4.128s/416,208KiB), fixed storage34 (21.302s/492,832KiB), declared functions5
(10.396s/414,368KiB), exceptions22 (5.207s/670,576KiB), callback errors2
(1.563s/236,256KiB). All8 forEach oracle checks pass (0.520s/880KiB).
An initial build command used the wrong `exceptions` target name and stopped
before compilation; corrected to `exception_handling`, no product failure.

Parent consumers rebuilt from the still-verified previous release compiler match
all four fresh profile emission hashes. Existing void int/recursive-record control
emission stays byte-identical in both debug and final release compilers.
Quiet matched A/B uses identical source/crate/output paths and executable argv[0],
original3×31 samples,2000 items,seed7,8 warmups; full results equal:
- String0.094583→0.053625ms (1.763786×), repeat0.078917→0.041708ms (1.892131×).
- Record0.108125→0.063375ms (1.706114×), repeat0.065875→0.033833ms (1.947064×).
- Nested0.420542→0.357834ms (1.175243×), repeat0.410417→0.356458ms (1.151375×).
- Callable0.082000→0.046500ms (1.763441×), repeat0.073875→0.041667ms (1.772986×).
Three compile samples/version (wall/user ms): string208.075/247.541→
201.154/242.789; record204.645/244.951→205.843/244.832;
nested234.813/279.374→226.717/271.858; callable201.490/244.818→
202.146/243.176. No material consumer compile regression; final candidates restored.

The initial callable A/B process was killed by macOS taskgated: crash report says
SIGKILL (Code Signature Invalid), CODESIGNING/Taskgated Invalid Signature. It was
not a watchdog/resource stop or product assertion. Preserve completed compile
samples. Atomic replacement fixes the unfinished callable comparison; repeat the
three earlier runtime comparisons with signature-verified binaries and atomic
swaps to remove that uncertainty. Numbers above are these final atomic results.
Max load3.567; watchdog phases7.781s/225,984KiB (interrupted),3.642s/220,192KiB
(callable),2.092s/32,064KiB (three runtime-only repeats). **Always atomically rename
A/B executable replacements on macOS; do not overwrite existing executable inodes,
even after the prior process exits.** Fresh focused Node controls come from the
canonical run below; unchanged Rust references are reused.

Fresh production profiles: parent4.676s/268,064KiB, candidate3.112s/276,176KiB;
three identical samples/workload, same checksums, no reallocations, retained bytes
or cleanup residue. String/record/callable each7,601→6,001 allocations,
174,040→126,040 requested bytes,107→67 peak bytes; nested13,601→12,001,
1,422,040→1,374,040 requested bytes,315→275 peak. Combined6,400 fewer
allocations/192,000 requested bytes per four-workload batch.

Canonical run `2026-11-03T10:46:38.159Z`,310.537s/1,526,544KiB, all five
independent correctness cases/full timed results pass,93 samples/engine:
string native0.024958ms/67,692,616.662mean ops/s,Node0.221625ms;
record0.024958ms/76,698,220.206ops/s,Node0.219834ms;
nested0.229250ms/8,631,142.506ops/s,Node0.220125ms;
callable0.024917ms/80,032,582.838ops/s,Node0.220500ms.
Actual timing max load3.998 below4, no override. Exactly8 ingested histories match
raw medians/mean rates/counts/timestamp; SQLite integrity OK. Existing performance
tables select80 runs,635/933 Tsrust wins over Node,621/933 Rust wins,median1.60×.
Two page tests pass;66-page site builds3.645s/1,002,480KiB.
String/record/callable are now1.045–1.055× saved Rust;
nested0.229250/0.122583ms remains1.870×,0.106667ms excess.

Whole-project formatter passes token/idempotence checks with zero source, snippet
or fingerprint changes (4.116s/202,336KiB). Pruned38 closed scratch/raw files
(10,180,100bytes) after exact measurement verification. Current production timing/
profile consumers remain; all owned workers terminal. No broad regressions, remote
CI or push occurred.

The fixed-return acceptance above supersedes this historical nested-result gap.
Current continuation is the counted-builder sizing plan above; nullable dispatch trials are closed.

**Direct singleton remaps — accepted locally, parent `869651fd`.**
`runtime/maps.rs` constructs a remapped inline singleton directly. Its source Map
already guarantees unique normalized keys, so rebuilding through `set()` is
unnecessary. Identical materialization/indexed fallback branches are merged to
avoid duplicate generated iterator closures. No storage layout, public API,
ownership policy, dependency, source limit or generated unsafe-code change.
`tests/maps.rs` adds sparse singleton owner-move/drop witnesses for small/indexed
sources and panic cleanup after singleton conversion. Ordinary-program runtime
constant and all emission selection logic remain byte-identical to the parent.

Affected build200.944s/1,440,224KiB. All240 tests pass, none ignored: Maps8
(5.722s/453,280KiB), declared functions5 (10.374s/494,736KiB), exceptions22
(4.690s/657,600KiB), scoped composition205 (175.234s/671,024KiB), one binary
at a time, three threads, usual5GB/30min watchdog. Canonical release + native/
Node run273.288s/1,550,224KiB. Production emission exactly matches the prepared
candidate generated consumer. Quiet final A/B, identical canonical source/crate/
output paths and executable argv[0], original3×31 samples:23.202250→21.312875ms
(1.088649×), repeat23.318708→21.408000ms (1.089252×). Max load2.969,
watchdog13.439s/33,344KiB. Fresh unchanged Node control comes from canonical run
below; no extra Node/Rust reference timing for exploratory native-only rounds.
Three rotated compile samples/version: median wall29,337.746→29,354.621ms
(+0.058%, noise), user35,415.463→35,379.528ms (-0.101%); max load2.830,
watchdog175.972s/1,094,880KiB. Final candidate source/binary restored after A/B.

Production allocation profile37.740s/1,050,208KiB, three identical samples:
checksum108435446,245,355 allocations,2,056 reallocations,243,306 deallocations
during work,32,374,976 requested bytes,2,326,528 retained/peak requested bytes,
input11, both cleanup counters0. These equal the parent. Canonical run
`2026-11-03T09:09:28.439Z`: native21.408417ms/95,460.327mean ops/s,
Node151.765375ms/13,478.104mean ops/s,93 samples each, full exact results equal,
max load2.320, no busy override. Exactly two ingested histories match raw
medians/rates/counts/timestamp; SQLite integrity OK. Saved Rust18.420917ms is
reused; composition is now1.162× /2.987500ms above it. Existing performance page
selection advances to77 runs, with unchanged635/933 wins and1.60× median across
all workloads. Two page tests pass;66-page site builds3.623s/888,416KiB.

Rejected probes, retained as findings rather than production changes:
- Allocation attribution:171,626 singleton→Vec promotions all allocate; remaps
  allocate nothing (32,766 empty,456,302 singleton,13,927 two-entry).157,290
  singleton remaps have a four-slot Vec, no inline first, inline destination and
  compatible40→32-byte entry layouts. Two buffer-reuse variants change no
  allocation counters and are not implemented.
- Two-entry inline storage cuts allocations245,355→75,777 and requested bytes
  32,374,976→5,406,336, with unchanged retained/peak heap, but runs3–5% slower
  (0.956719×/0.966656×) and compiles26.8% slower (29.734→37.692s). Rejected.
  Allocation counts alone are insufficient evidence for an arena/storage change.
- Direct construction with duplicated fallbacks runs1.104×/1.110× faster but
  compiles9.1% slower (30.023→32.757s). Merging fallback closures removes this
  compile regression; only the final merged form is implemented/ingested.

Whole-project formatter passes token/idempotence checks with zero changes
(4.136s/202,784KiB). Rendered existing composition row shows95.5k Tsrust ops/s
and7.08× Node; no extra table. The ordinary Vec-only runtime constant equals the
parent byte for byte. Pruned50 closed generated/experimental files
(72,235,282bytes), including all current attribution/reuse/pair/direct scratch files
and exactly verified raw results. Current production timing/allocation consumers
remain, all owned workers terminal. No broad suites, remote CI or push occurred.

**Stored-callable dispatch — accepted locally, parent `503f3bdf`.**
`crates/tsrust-compiler/src/callables/dispatch.rs` marks concrete family `call` and
`call_with_count` methods `#[inline(always)]`. This exposes known closure origins
at their call sites without forcing factory or closure-body inlining. No public
API, ownership/storage change, dependency, input limit or generated unsafe code.
Existing O0/O3 dispatch witnesses cover owned drops, readonly/shared identity,
fallible results and argument-count forwarding; no new logic or fixtures needed.

Affected build484.143s/1,660,304KiB. All162 tests pass, none ignored, one binary
at a time under5GB/30min watchdog, three threads: dispatch units6+9
(2.604s/330,464KiB and1.558s/203,088KiB), shared callback4
(1.038s/160,016KiB), callable fields35 (16.277s/451,088KiB), recursive owned
callbacks15 (3.624s/362,896KiB), fallible record callbacks4
(1.556s/493,536KiB), testing runtime18 (0.523s/32KiB), testing library60
(18.200s/780,064KiB), async testing11 (8.318s/1,353,376KiB).
The unit selection excludes unrelated suites; this is not a full regression run.

Quiet final A/B against the exact parent, identical source/crate/output paths
and executable argv[0], original3×31 samples,4096 items,seed7,8 warmups:
0.490875→0.372834ms (1.316605×), repeat0.470500→0.358166ms (1.313637×).
Removing only the new dispatch attributes exactly restores the saved parent
consumer source hash. Candidate binary shrinks510,080→509,696bytes.
Default-parameter stored-family and nullable-owned-record controls produce
byte-identical parent/candidate executables (including disassembly); their
~2% timing fluctuations are noise. Fallible-record callback source is unchanged.
Max load2.151 across final controls; watchdog4.660s/186,112KiB followed by
4.148s/179,984KiB. The first control attempt omitted required input text in the
scratch harness; repaired only that harness and resumed unfinished controls.
No product failure occurred and the completed expectation results were retained.

Three compile samples/version: expectation median wall344.172→351.194ms,
user390.634→393.974ms. A reverse-order three-sample repeat resolves the small
wall difference:350.500→349.784ms, user391.657→393.280ms; no material compile
regression. Repeat max load2.217, watchdog2.581s/169,296KiB. Default family
wall258.804→257.333ms/user308.759→304.215ms; nullable record
wall271.665→270.267ms/user327.416→326.257ms. Final candidate consumers restored.

Production profile2.065s/326,528KiB: three identical original4096-item samples,
checksum730275589,11,606 allocations,0 reallocations,11,606 work deallocations,
305,152 requested bytes,77 peak bytes, zero retained/input bytes and both cleanup
counters zero. Equal to the parent. Allocation attribution:4,096 shared string
identity environments,4,096 shared record identity environments and3,414 string
repetitions. String/record expectations only call length matchers here, but still
construct identity callbacks. Preserve non-Copy shared environment identity;
independent payload clones are not a valid optimization. Earlier factory-inlining
probes leave allocations unchanged and improve only1.034×/1.050×; not implemented.
Earlier generated-only dispatch probes suggested1.385×/1.369×; use the final
production-path A/B numbers above for the accepted change.

Canonical run `2026-11-03T10:09:28.218Z` (216.387s/1,910,528KiB), original93
samples/engine across four workloads, all five independent correctness cases and
exact timed results pass. Expectation native0.392791ms/230,729,918.938mean ops/s,
Node0.373334ms/242,959,160.608mean ops/s. Fresh focused Node control only;
unchanged Rust references reused. Max load2.487, no busy override. Exactly8
history entries ingested with `npm run store:ingest`, independently matched to
raw medians/mean rates/counts/timestamp; SQLite integrity OK. Public existing
results refreshed to78 selected runs:635/933 faster than Node, median1.60×.
Two performance-page tests pass;66-page site builds3.117s/972,128KiB.

Saved Rust0.123667ms leaves a3.176×/~0.269ms canonical gap. This benchmark builds
real reusable expectation objects; Rust uses direct standard assertions and Node
uses direct Node assertions. The manifest already documents this API difference.
Do not describe the residual as pure record construction/access overhead. Unused
identity-callback construction remains a possible general record-field-demand
optimization, requiring preserved owner drops and callable identity. Avoid a
benchmark-specific matcher rewrite or arena dependency without new evidence.

Whole-project formatter passes token/idempotence checks with zero changes
(4.144s/202,864KiB). Pruned53 closed scratch/raw files (15,511,167bytes) after
exact ingestion verification. Current production timing/profile consumers remain.
All owned workers terminal. No broad regressions, remote CI or push occurred.

**Byte-view allocation policy — reference repair, parent `0b53dee2`.**
No compiler/runtime/Node workload change. Fresh native profile3.653s/223,856KiB,
three identical samples: checksum554157932,8,193 allocations,8 reallocations,
6,144 work deallocations,489,088 requested bytes,294,912 retained/294,917 peak,
input5,both cleanup counters0. Existing rejected heap-only arrays, direct slices,
infallible native reservation and forced byte-helper inlining remain rejected.

The compact-error hypothesis is also rejected. Generated-only static metadata
reduces ByteFailure48→24bytes, with identical allocation counters, but quiet A/B
0.094833→0.098459ms (0.963172×), repeat0.094541→0.093833ms (1.007545×),
max load1.852,1.040s/20,656KiB. No repeated speedup; not implemented or ingested.
Four boundary cases and exact timed results match. The first scratch allocation
command omitted its required text argument; fixed before timing, no product bug.

Disassembly shows native calls two fallible RawVec growth helpers per successful
iteration, while infallible Rust collection/to_vec eliminates temporary buffers.
A same-path paired Rust diagnostic keeps algorithms/results but uses fallible
reservation: native0.095292ms, ordinary Rust0.021542ms, fallible Rust0.088042ms;
repeat0.093083/0.021500/0.088709ms. Max load1.748,1.031s/20,992KiB; original
3×31 samples,2048 items,seed7,8 warmups; all boundary/full exact results match.
This explains most of the apparent4.2× gap. Error-record size does not explain it.

The existing reference returned Result for invalid byte values but used infallible
allocation and aborted instead of preserving construction/copy allocation-error
fallbacks. Fixed only `parity/native_byte_views.rs`: validate before reserving,
use standard Vec::try_reserve_exact for construction/copy, preserve construction
[-1000000] and copy-length -2, then continue subsequent iterations. No input,
protocol, sample-count or native allocation-policy change. Test-only thread-local
reservation failure injection verifies both actual workload fallbacks, subsequent
recovery, empty-copy no-reservation and invalid-byte-before-reservation ordering.
Its code is excluded from the timed reference. Both Rust tests pass at O0/O3
(2.588s/127,232KiB); all11 byte-view registrations pass (0.521s/672KiB).

Canonical corrected reference/control run `2026-11-03T10:26:46.257Z`,4.684s/
335,120KiB: native0.093083ms/21,758,664.071mean ops/s, Rust0.088292ms/
23,017,561.811mean ops/s, Node0.603209ms/3,581,162.037mean ops/s;93 samples
per engine, all five independent correctness cases and full exact results pass.
Actual timing max load2.676, no override. Native/Rust1.054263×,0.004791ms excess.
Reference SHA25699bb4cfabdbc388e6d785d21ea468734f9cf50fcccfafcc4bdb291d90279d631.
Exactly3 histories ingested and matched to raw rounded medians/mean rates/counts/
timestamp; SQLite integrity OK. Earlier histories retained; only the current Rust
source/timestamp pin advances. Public Method text explains fallible byte buffers
and matcher-object/direct-assertion API overhead. Existing tables now select79
runs:635/933 Tsrust wins,621/933 Rust wins over Node,median1.60×. Two page tests
pass;66-page site builds3.116s/1,022,544KiB. This is a benchmark error-path repair,
not evidence that Tsrust byte code became faster. No broad suites, CI or push.

Whole-project formatter passes token/idempotence checks with zero source, snippet
or fingerprint changes (4.146s/201,696KiB). Pruned25 closed scratch/raw files
(11,125,784bytes) after exact measurement verification. Current production consumers
remain; all owned workers terminal.

Current continuation is recorded in the latest non-void forEach acceptance above;
its measurements supersede the earlier discard-result review numbers.

**Scoped Map singleton acceptance — 2026-11-03 (parent `17ce8e21`).**
The first entry stays inline for scoped Maps when an iterative layout check proves
safety. Owned non-Copy captures, nested/recursive Map payloads and native payloads
retain Vec storage. Ordinary Maps stay Vec-backed inside mixed programs. Programs
that cannot use the optimization retain the verbatim original runtime, aliases
and Map formatting implementation. The separate runtime preserves non-target
code/layout. Same-carrier epoch moves remain valid; singleton promotion, deletion,
cloning, remapping between both storage modes, panic cleanup, recursive values and
owned callback captures have O0/O3 witnesses. No new API, dependency, source cap,
ownership policy or generated unsafe code. Source changes: `emit/map_emit.rs`,
`emit/mod.rs`, `runtime/maps.rs`, `tests/maps.rs`.

Final affected build: 203.563s / 1,515,344KiB. All 240 tests pass, none ignored:
Maps 8 (5.728s / 494,272KiB), declared functions 5 (10.378s / 386,304KiB),
exceptions 22 (4.683s / 625,984KiB), scoped composition 205 (175.279s /
656,816KiB). Final release CLI: 187.015s / 1,532,592KiB. All 11 Map benchmark
workloads pass independent five-case native/Node oracles. Final source proof
5.156s / 202,192KiB: composition equals the timed candidate byte for byte; the ten
other consumers equal the parent. Runtime/compile/profile evidence is therefore
applicable to the final guard (actual reference captures or Copy values only).

Quiet A/B against the parent, same source path/crate name/output path/argv[0],
original 3×31 samples per engine/case and focused Node control: composition
26.892542→23.488875ms (1.144905×), repeat 26.952291→23.536333ms (1.145136×);
Node 150.857667/149.876834ms. Max load 2.100; watchdog 67.180s / 220,064KiB.
Original three-sample Rust consumer compilation: median wall 50,868.484→
29,091.080ms (-42.8%), user 56,992.696→35,193.241ms; max load 2.112,
watchdog 251.799s / 1,043,872KiB. Early differing-crate-filename control timings
were confounded; do not infer algorithmic regressions from them. Always fix all
four paths/names for future final A/B. A statically empty iterator probe is not
implemented; it is a small refinement outside the current large-gap work.

Production profile: three identical samples, checksum 108435446; allocations
557,472→245,355, requested bytes 79,711,904→32,374,976, reallocations 2,056,
deallocations during work 243,306, retained/peak requested bytes unchanged at
2,326,528, input 11, both cleanup counters zero (38.121s / 1,050,512KiB).
Canonical run `2026-11-03T07:36:04.534Z`: native 23.946333ms, Node 152.061042ms,
93 samples each, full exact results equal, max load 3.377, no override
(282.464s / 1,572,784KiB). Exactly two ingested history entries verified against
raw medians/mean rates/counts/timestamp, SQLite integrity OK. Saved Rust reference
18.420917ms remains unchanged; the gap at this checkpoint is 1.300× / 5.525416ms.

Owner performance-page correction, 2026-11-03: keep the existing results layout;
no before/after table. That refresh added 22 accepted run
timestamps (76 total before subsequent accepted gates), and the corrected plain-byte-view Rust reference. Local
summary is 635/933 faster than Node, median ratio 1.60× (previously 630/933,
1.55×). These working-branch changes are not deployed. Keep the local generated
page current after accepted gates; deployment/promotion still waits for the
large-gap milestone and full validation. Source is
`website/tools/performance-runs.json`, generated by
`website/tools/generate-performance.mjs` from SQLite. Earlier accepted gains
were in SQLite/handoff but absent from the stale curated selection. Website readability/data tests pass (2), and the
66-page site builds (3.748s / 998,288KiB); rendered results show635/933 and1.60×,
with no before/after table. Whole-project formatter passes token/idempotence
checks with zero changes (4.115s / 195,296KiB). Pruned 93 closed generated/
experimental files (67,556,786 bytes), including all current single-map scratch
files, verified raw results and ten stale non-target benchmark executables.
Current release CLI and composition timing/allocation consumers remain; every
owned worker is terminal. Continue profiling the remaining large gaps; no broad
suites, push, CI dispatch or release occurred.

Previous completed section:Copy-only shared readonly callback environments.
Expectation predicates improve1.318× (repeat1.255×),removing8,192 allocations;
79 affected tests pass. Mixed/owned families retain Rc identity and destruction
behavior. See the acceptance section below. Full regressions remain deferred.
Plain byte views also have a completed reference-fidelity correction below:
the Rust reference now preserves the source's recoverable construction fallback.
Fresh native/Rust timings are.094375/.022250ms (4.242×);the gap remains.
The reference correction preserves its fallback and is not a compiler speedup.
The old.021000ms Rust entry stays in history.

Owner object-profiling feedback is retained in **Object and string allocation —
next optimization plan** below: string-free controls, generated/reference code
comparison, allocation attribution, destination reuse and conditional arena review.
Follow its updated status before revisiting those hypotheses.

**Git:** branch `codex/optimize-native-hot-paths`. Published main **9c294876**
contains the v0.1.1 source from staging **fa2bd1a9**. Local post-release checkpoints
include **0a68a4f5** (932 Rust references), **2c538120** (normalization optimization)
and **ff32ab18** (release-note provenance fix); they are not pushed/published.
Published main ancestry is reconciled into this working branch without changing
those accepted source/measurement trees; only this handoff records the merge.
Latest accepted optimization:direct Map singleton remaps,composition1.089×
(repeat1.089×),unchanged allocation/heap and consumer compile costs. Previous
singleton inline storage1.145× (repeat1.145×),312,117 allocations removed,
consumer compile time42.8% lower. Previous Map identity moves remove440 remaps
and improve composition1.068× (repeat1.059×),consumer compile time8.2% lower.
Previous Copy-only shared callback environments,expectation
predicates1.318× (repeat1.255×),8,192 allocations removed,peak heap133→77 bytes.
Previous borrowed forEach ranges,nested records4.905×
(repeat5.280×),deep int4.213× (repeat4.129×),recursive3.691× (repeat3.784×);
all12 normal cases improve and now span0.962–1.278× saved Rust. Previous
constant-string concat results,literal spread3.414×
(repeat3.433×),call spread3.448× (repeat3.321×),8,196 allocations removed per
workload,retained/peak requested heap -15.2%. Previous read-only scalar reduction slices,Number16.705×
(repeat18.030×),boolean17.421× (repeat18.861×),int4.994× (repeat5.000×),
1,536 allocations removed each. Previous dead clone/push loop transfer,recursive reversal
4.856× (repeat4.789×),replacement4.095× (repeat4.126×),3,840 allocations
removed per workload,retained heap unchanged. Previous dead owned-array copying receivers,recursive
toReversed1.946× (repeat1.795×),with1.633× (repeat1.586×),3,840/3,200
allocations removed,retained memory unchanged. Previous small Map carrier buffer reuse,composition1.640×
(repeat1.607×),718,447 allocations removed,retained/peak heap unchanged. Previous
fixed floating-point literal returns,temporary number indexing21.503×
(repeat22.536×),32,768 heap calls removed. Previous
decimal int.toString helper reuse1.140×
(repeat1.137×),384 allocations removed,retained/peak heap -8.2%. Previous
readonly stack character views,string iteration
1.171× (repeat1.147×),allocations -44.4%,retained/peak heap unchanged. Previous
text-only self-assignment buffers,string iteration
2.307× (repeat2.288×),allocations -86.2%,retained heap unchanged. Previous
readonly small parameter slices:nested rest1.066×
(repeat1.046×),2,048 temporary allocations removed. Previous fixed result rows
and readonly small local slices:interfaces1.184×
(repeat1.143×),retained memory -28.7%,nested rest1.208×,numeric rows4.746×.
Previous borrowed local record string copies:interfaces1.260×
(repeat1.319×),2,048 allocations removed. Previous direct finite-number formatting (numeric literals1.014×,
mixed conversion1.021×,nullable1.036×,pop1.045×),with identical allocations.
Previous static nonfinite number text (nullable1.111×,
numeric conversion1.087×;513/1,430 allocations removed). Previous direct numeric
string appends removed1,024 temporary allocations (controlled concat1.034×)
and proven-whole enum appends are official1.077×. Previous fractional template formatting is1.099×,
4,100→2,052 allocations. Previous single-digit String(number) storage reuse
is enums1.750×, mixed-number arrays/unions1.07×. Previous String(int) reuse
is interfaces1.198×/splice and nullable1.45×. Previous plain JSON decoding with
complete integer validation is recursive1.059×/string reads1.108×; preserved object slots are
1.039×; identity-preserving Map/Set validation is 1.275× (**32cdfb23**).
Following **8da8e64d** (cached normalization classes, 1.040×), **ee108cdc** (table-bound fast paths, 1.174×), **23abf642** (in-place composition, 1.029×), **2c538120** (scratch
reuse, 1.316×), `28a3409c` (typed JSON pipeline restoration) and `1d3eb962` (single
replacement sizing acceptance). Earlier
accepted work: `260d1781`/`cea19c4c` Map planning, `5083586a` schema restoration,
`610c2bc0` scalar-union arrays, `a74da73f` Rust parser defaults, `a2f94dee`
plain-schema decoding, and `1e4d2547` complete milestone/test-support repair.
The rejected small-string experiment remains on `codex/small-string-experiment`;
do not merge it. See `git log` for exact latest evidence checkpoint.

**Map identity moves — accepted locally, parent7fe0e254:**
- Profile-first generated-consumer probe removes440 identity conversions between
  Map aliases with the same Rust carrier. Compilation50.819s/946,192KiB;probe
  allocation build49.711s/801,504KiB. Two same-path quiet rounds,original3
  processes×31 samples,8 warmups,2,048 iterations:27.588417→26.074959ms
  (1.058×),28.481834→26.494958ms (1.075×),Node146.910834/149.568417ms.
  All checksums/exact bits match,max load1.917969,54.339s/213,680KiB.
  Allocation counters remain exactly equal,so the gain removes traversal work.
- same_map_carrier checks key/value types,ordered versus ordinary Map,and the
  actual emitted entry carrier. Reused existing shared-entry fallback selection
  in one lazy linear index consumed by alias declarations and this guard. This
  avoids repeated scans and prevents the guard drifting from actual Rust types.
  Scoped carriers use canonical declaration IDs;ordinary carriers keep their
  original IDs. The two epoch emitters move identical owners directly. Empty,
  absent-payload and transferred-owner fallbacks retain their prior behavior.
  No runtime storage,API,editor,limits,dependency or unsafe change.
- New O0/O3 witness covers standalone and record-field Maps,tombstones,reinsert
  order,lookup,small/indexed threshold transitions,finally overriding pending
  return/throw/continue/break,and capture release after clear. Parent emits83
  identity remaps;final emits0. First conservative guard left16 because raw entry
  metadata omitted the shared-entry fallback;the witness caught it before other
  tests ran. A direct-move Rust diagnostic compiled,proving actual carriers equal.
  The first scratch source also exposed unsupported returned local borrows and
  a missing named-record annotation;replaced it with supported local callbacks.
- Affected builds198.490s/1,560,592KiB and197.899s/1,502,336KiB;final build has
  no diagnostics. Final239 tests pass,none ignored,one binary at a time with3
  threads:O0/O3 witness1 (3.619s/240,704KiB),Maps7 (2.605s/497,632KiB),
  declared callbacks5 (10.332s/534,304KiB),exceptions22 (4.661s/571,088KiB),
  remaining scoped composition204 (170.061s/662,928KiB). The focused witness
  was skipped in the later file run to preserve its completed evidence.
- Exact-parent formal preparation84.449s/1,096,128KiB;current release/profile
  268.111s/1,570,976KiB;current formal2-case run82.310s/954,704KiB. Three
  allocation samples match exactly:checksum108435446,allocations557,472,
  reallocations2,056,deallocations555,423,requested79,711,904,retained/peak
  2,326,528,input11,both cleanup counters0. Map aliases and entry declarations
  are byte-identical;from_mapped calls1,222→782,identity calls440→0.
- Formal native28.440958→27.146584ms,Node149.923750/151.931917ms;93 samples
  each,quiet max2.401855/1.984863,no override. Saved Rust18.420917ms reused;
  current native/Rust1.474×,8.725667ms absolute excess remains material.
- Production same-path A/B,2 rotated rounds,93 samples/engine/workload/round:
  composition27.422541→25.668375ms (1.068×),27.761583→26.222167ms (1.059×),
  Node144.370916/147.099209ms. Control byte-view Maps.477000/.481750ms then
  .470000/.463541ms;source and complete253,860-byte Mach-O __TEXT,__text
  section are bit-identical at address4294969480,SHA256
  1cf58775c1b886d68c1343279bf4808b677dcd74cca561bfc72676ef98a68735.
  Checksums/exact bits match,max load1.790527,54.340s/225,360KiB.
- Consumer compilation,original3 samples/engine at identical source/output paths:
  parent54,824.298ms,current50,347.051ms (-8.2%);user CPU63,147.845→56,466.383ms.
  Compiler emission,3 rotated samples/input/engine,stable paths/identical argv0:
  composition4402.378→4413.315ms wall,user4385.504→4384.321ms;small Map
  control7.846→7.787ms wall,user4.801→4.956ms. Combined342.682s/1,030,640KiB,
  quiet max2.437988. A warmed11-sample/engine×2-round check resolves that tiny
  control CPU drift:user4.609/4.601ms then4.561/4.616ms,wall7.223/7.156 then
  7.249/7.213ms;identical emitted hash,max load1.742188,.513s/1,600KiB observed.
  No material compiler regression;do not claim frontend speedup.
- Exact parent emission for caller_borrowed_record_stores,scoped_callable_records,
  fallible_record_callbacks and native_byte_views_maps has zero identity remaps
  (1.042s/51,776KiB). The last is the unchanged control;no broad benchmark run.
  npm run store:ingest added6 baseline/current history entries across2 files;
  verified exact rounded medians/mean rates,93 samples,timestamp uniqueness and
  SQLite integrity. Other Rust references were not retimed.
- Whole-project formatter4.135s/200,512KiB:zero source/snippet/fingerprint changes.
- Pruned30 closed scratch/parent/prototype/imported raw files,76,486,042 bytes.
  Current verified tools,consumers,profiles and passed-test binaries remain;all
  owned workers are terminal.
- Next:remaining composition cost includes genuine value-layout widening with
  live1 slot32→40 transitions;the earlier dense-count/reallocation attempt is
  already rejected. A safe inline-single-entry generated-consumer probe could
  bound the opportunity before any compiler storage design;default inline
  embedding must not create recursive Rust layouts. Plain byte views still have
  a real4.242× gap;isolate transient native error/Result layout before more
  attribute experiments. Expectation/nullable/discard opportunities stay open.
  Preserve deferred full-regression evidence. No public export,push,CI or release.

**Plain byte views — profiling and reference correctness, parent848b6ac7:**
- Fresh accepted-source baseline.095750ms,Node.601292ms,93 samples each;
  quiet max2.224609,no override,2.589s/273,968KiB. Native profile
  3.629s/325,344KiB:three equal samples,checksum554157932,8,193 allocations,
  8 reallocations,489,088 requested bytes,294,912 retained/294,917 peak,
  input5,both cleanup counters0. No production byte implementation changed.
- Rejected generated-consumer probes,two quiet rounds each,3 processes×31
  samples,8 warmups,2,048 iterations,focused Node controls and exact-result
  equality. Direct borrowed literal slices remove2,048 allocations/81,920
  requested bytes,but speedups1.004×/.960× do not repeat. Byte ABI adapter
  forced inlining1.011×/.993×;native allocation/range helper forced inlining
  1.016×/.991×;no allocation changes. Heap-only NumericArray storage reduces
  retained49,152 bytes to245,760,but only1.008×/1.011×. Do not introduce
  that global representation change for this CPU gap.
- Diagnostic-only infallible reservation is slower,.096542/.140541ms then
  .093750/.142584ms. It changes allocation-failure policy and was never a
  production candidate. Preserve recoverable allocation errors;this experiment
  does not justify an API/Rust-default design change.
- Original unwrapping Rust reference instrumentation:three equal samples,
  6,145 allocations,9 reallocations,309,152 requested bytes,245,760 retained/
  245,765 peak,cleanup0. Sampling repeated consumers for3s each at1ms intervals
  locates allocation/free work,not byte ABI wrappers,as the dominant cost.
  The diagnostic loop includes constructors and result destruction;it is not
  formal execution timing. CPU sampling/compilation7.204s/188,256KiB.
- A one-pass Result-collection reference variant appeared to explain the gap:
  .062792/.062791ms against native.094042/.092417ms. That attribution was
  wrong. A two-pass reference preserving the same recoverable branch runs
  .021417/.021458ms against native.096292/.093875ms (4.496×/4.375×).
  Five protocol-valid exact-result cases match. Result collection inhibits an
  optimization;do not conflate that algorithm change with error-path fidelity.
  Its allocations remain6,145/reallocations9,but requested315,296/peak245,768;
  the final two-pass variant restores309,152/245,765 in three equal samples.
  The provisional .062542ms formal reference/control run was ingested too early;
  removed only its3 unaccepted entries and raw file. Original accepted histories
  remain. Do not quote that provisional1.487× ratio as current or as a speedup.
- Corrected only parity/native_byte_views.rs:validate before allocation,collect
  the known-size byte conversion,return Result,and preserve the source's
  [-1000000] fallback with continue. This fixes reference behavior without
  changing the original timing domain or making the reference do avoidable work.
  New standalone witness covers first invalid offset,negative-seed fallback,
  subsequent recovery and valid extremes. Negative-seed checking is outside the
  timed nonnegative-seed domain;the bounded oracle/observer protocol is unchanged.
  Final Rust witness passes O0/O3 (2.596s/109,968KiB);existing Node registration
  suite11 passes,no skips (initial combined checks3.100s/169,792KiB).
  Scratch preparation errors (wrong function index,an out-of-domain observer
  case,and an incorrect borrow in the range check) were repaired before timing.
- Final reference SHA256
  ba6f6061a4588cb2441516cb0d224469f69a1e559fc22e6179a7b92a5e735090.
  Remeasured only this changed Rust reference,with native/Node controls:
  native.094375,Rust.022250,Node.596125ms,93 samples/engine. The runner waited
  through initial load5.72/4.82;actual timing max3.973633,below4,no override.
  Watchdog34.843s/277,552KiB. Native/Rust4.242× is still a real gap;the
  reference behavior repair is not a Tsrust optimization. Other Rust references
  were not retimed. Do not change reference algorithms to hide compiler gaps.
- Whole-project formatter4.210s/203,776KiB:zero source/snippet/fingerprint changes.
- npm run store:ingest added5 accepted baseline/final history entries;verified
  exact rounded medians,mean rates,93 samples,timestamp uniqueness,reference
  source hash,original-history retention and SQLite integrity. Pruned54 closed
  diagnostic/prototype/imported raw files,19,689,090 bytes. Current verified tools,
  consumers/profiles and passed tests remain;all owned workers are terminal.
- Next:return to scoped composition's material absolute gap. Trace identity
  from_mapped conversions and canonical Map aliases before revisiting widening
  layouts. Dense-count collection is already rejected. For byte views,a future
  bounded probe can isolate the large emitted error-record/Result layout from
  hand Rust's compact Error before changing storage;this remains a hypothesis.
  Preserve the expectation/nullable/discard queue and deferred full-regression
  snapshot. No broad regressions,public export,push,CI or release.

**Copy-only shared callback environments — accepted locally, parent e6ef644c:**
- Profile first:expectation predicates19,798 allocations,zero reallocations,
  534,528 requested bytes,zero retained,133 peak. Constructor inline hints
  were rejected:two rounds only1.024×/1.003×;forced inline1.035×/.993×.
  A direct-storage generated-consumer prototype measured1.316×/1.237× and
  removed8,192 allocations before production changes.
- Private callable-family emission stores an environment directly only when
  every origin captures exclusively owned Copy scalars,without capture references.
  Clone reconstructs those scalar fields;empty environments work too. A single
  non-Copy capture keeps the whole family reference counted. Strings,records,
  arrays,identity,destruction and counted-call forwarding retain their behavior.
  No unsafe,new dependency,API,signature or editor change.
- Two new unit witnesses cover multiple origins,empty captures,mixed Copy/owned
  families,cloning after original drop,and singleton counted invocation. First
  build exposed one direct Family test initializer missing its new private field;
  repaired it. Failed build189.500s/1,710,304KiB;successful affected build
  458.895s/1,831,392KiB. One existing unit-harness dead-code warning remains for
  stdlib/json/typed.rs::decode_plain;generated JSON emission uses that runtime
  function and its source is unchanged here. Only affected tests executed,one binary
  at a time with3 threads:dispatch15 (3.626s/164,288KiB),testing_library60
  (18.701s/724,416KiB),shared_read_callback4 (1.042s/118,384KiB);all pass,
  none ignored. Generated witness execution includes O0/O3 and existing owned
  identity/drop,nullable,NaN,signed-zero,negation and asynchronous-main checks.
- Production profile212.618s/1,911,920KiB including release build:three identical
  samples,checksum730275589,allocations11,606,reallocations0,requested305,152,
  retained0,peak77,input0,both cleanup counters0. Allocation count falls41.4%,
  requested heap42.9%,peak42.1%. No ownership defaults changed.
- Formal parent/current native medians.632000/.494000ms,Node.347667/.371833ms.
  Original3 processes×31 samples,8 warmups,4,096 iterations;quiet maxima
  2.608398/1.977051,no busy override. Baseline two-workload refresh
  5.682s/338,528KiB,current3.642s/273,072KiB. Saved Rust.123667ms is reused;
  the remaining3.995× gap is still substantial,not closed by this change.
- Two same-path rotated A/B rounds:parent/current.639000/.485000ms (1.318×)
  and.638666/.508958ms (1.255×);focused Node.349792/.353625ms. Each engine
  retains93 samples per round. Byte-view Map control fluctuates .477750/.491292
  then .496833/.475000ms,but emitted source and complete253,860-byte Mach-O
  __TEXT,__text section are bit-identical at address4294969480,SHA256
  1cf58775c1b886d68c1343279bf4808b677dcd74cca561bfc72676ef98a68735.
  All checksums/exact results match. A/B watchdog2.078s/64,384KiB,max load1.686035.
- Consumer Rust compilation,original3 samples/engine/round,same source/output
  paths:375.633→347.640ms then377.966→348.669ms (~7–8% faster). Compiler
  emission uses stable executable paths and identical argv0,3 samples/input/
  engine/round:expectation35.026→33.940ms then35.181→34.517ms;composition
  4322.874→4303.953ms then4367.429→4358.950ms. User CPU also stable/faster.
  Composition emitted bytes unchanged. Combined timing57.384s/173,008KiB,
  max load2.266113;no compiler regression or forced inlining.
- Whole-project formatter4.143s/203,584KiB:zero source/snippet/fingerprint changes.
  npm run store:ingest added8 baseline/current history entries across2 files;
  verified exact rounded medians,mean rates,93 samples,timestamp uniqueness and
  SQLite integrity. Reused saved Rust references;only focused Node controls ran.
- Pruned40 closed scratch/prototype/imported raw files,59,512,837 bytes.
  Current verified compiler,consumers,profiles and passed-test binaries remain;
  all owned workers are terminal.
- Queue correction:the historical4.82–4.85× label incorrectly grouped byte-view
  Maps with plain byte views. Fresh Maps.498625ms vs saved Rust.384583ms is
  only1.297×;do not optimize that control as a large gap. Plain native_byte_views
  was subsequently profiled and its reference corrected above:current4.242×.
  Continue remaining expectation work,plain byte views,and scoped composition's
  material absolute gap. Deferred full-regression evidence remains untouched;
  no public metric export,push,CI or release.

**Borrowed forEach slice ranges — accepted locally, parent428a1769:**
- Profile-first parent2026-11-03T01:16:02.663Z,216.852s/1,538,320KiB:
  nested/deep/recursive traversals make8,006/1,601/6,004 allocations,retained0.
  Source clones base.slice before callbacks that only read values and source.length.
  An isolated generated-consumer range probe removes8,000/1,600/6,000 allocations,
  preserving checksums and cleanup in three equal samples (3.117s/158,016KiB).
- Private foreach_slices index proves immutable owned local roots,all source-name
  uses solely slice receivers,and all snapshot-name uses eligible forEach receivers.
  Capture/move/mutation/escape/native loans/exception-region/fixed-layout roots keep
  ownership. Native/unknown/callable fields propagate through recursive dependencies
  in linear time;only plain data is borrowed. Record proof is computed lazily.
  Concrete callbacks must have3 required read/copy parameters,a void result,and
  only .length reads of the source parameter. Forwarded/polymorphic/async/task/
  scoped/counting/rest or unproved callback interfaces retain the ordinary path.
- One call_range body accepts &Range<usize>;the normal call wrapper retains the
  original array parameter and public/trait ABI. Original bounds/evaluation/error
  order and relative callback indices remain. SliceView enum reuses existing bounds
  code and preserves scalar-reduction views. ArrayIter.nth delegates to its slice
  iterator,keeping prefix skips O(1). No unsafe,RC,dependencies,public APIs,limits,
  source restrictions or IntelliSense changes. Non-void callbacks stay unoptimized.
- Initial9-test validation caught an invalid new witness that returned a mutable
  capture before its closure scope ended. Corrected the witness scope;no compiler
  ownership weakening. Final O0/O3 witnesses cover relative/default/negative/extreme
  ranges,dynamic strings,normal and trait callback entry points,source moves on
  exited branches,mutation/whole-array-read fallback,nested catches/typed errors,
  argument order/unwind and nth/next_back/exhaustion parity against std iterators.
- Final86 affected tests pass/no failures or ignored:forEach9 (6.242s/446,752KiB),
  slice7 (8.839s/481,936),reduce7 (4.702s/462,736),fixed_storage34 (20.748s/
  490,928),declared_function_values5 (10.913s/519,920),exceptions22 (5.750s/
  644,720),callback_errors2 (1.562s/349,216). Final shared build200.474s/
  1,547,952KiB. Earlier build198.827s/1,485,856;intermediate release profile
  215.353s/1,572,048. Range iterators made the initial string exclusion unnecessary;
  removed it,added a dynamic-string witness,and reran the complete affected section.
- Exact saved parent compiler SHA256
  98f63316c2786dd17509df3eb375b250eccfbee095f2de79463fb4337a54537e;
  re-emission matched all three baseline profile hashes. Parent consumers rebuilt
  from that compiler as needed. No source snapshots or archive records retained.
- Actual completed release/profile213.331s/1,537,872KiB,2026-11-03T02:19:03.525Z;
  remaining seven profiles03:03:14.313Z,5.848s/265,424KiB. Exact parent observers
  4.678s/256,128 and4.656s/257,968. Every profile has3 equal samples,no reallocs,
  no retained output bytes,both cleanup counters0. All12 normal workloads halve
  peak requested heap and remove39,600 allocations in aggregate across the cases.

| forEach workload | Allocations before → after | Peak requested bytes before → after | Same-path A/B, repeat | Final ms / saved Rust ms |
| --- | --- | --- | --- | --- |
| int | 1,601 → 1 | 80 → 40 | 1.730× / 1.712× | 0.023667 / 0.023708 |
| number | 1,601 → 1 | 80 → 40 | 1.225× / 1.172× | 0.104042 / 0.104541 |
| boolean | 1,601 → 1 | 10 → 5 | 1.451× / 1.477× | 0.022292 / 0.023167 |
| string | 3,202 → 2 | 262 → 131 | 3.326× / 3.082× | 0.028875 / 0.026459 |
| record | 3,202 → 2 | 822 → 411 | 3.065× / 2.919× | 0.037250 / 0.029209 |
| nested_int | 1,601 → 1 | 480 → 240 | 2.042× / 2.173× | 0.025000 / 0.025084 |
| nested_number | 6,405 → 5 | 336 → 168 | 2.125× / 2.173× | 0.102875 / 0.105125 |
| nested_boolean | 1,601 → 1 | 240 → 120 | 2.011× / 2.096× | 0.024917 / 0.024459 |
| nested_string | 3,202 → 2 | 1,062 → 531 | 2.937× / 2.998× | 0.045917 / 0.035917 |
| nested_record | 8,006 → 6 | 902 → 451 | 4.905× / 5.280× | 0.038083 / 0.037750 |
| deep_int | 1,601 → 1 | 2,000 → 1,000 | 4.213× / 4.129× | 0.025667 / 0.025292 |
| recursive_record | 6,004 → 4 | 806 → 403 | 3.691× / 3.784× | 0.051334 / 0.042708 |

- Formal parent baseline01:23:17.842Z:5 workloads,93 samples/engine,quiet max1.736,
  5.693s/240,496KiB. Final full affected measurement02:45:27.474Z:12 normal
  traversals+discard-string control,93/engine,quiet max3.44140625,no override,
  12.428s/301,040KiB. Intermediate accepted-source6-case run02:19:09.723Z
  also retained (5.704s/295,536KiB). Existing Rust references reused,not retimed;
  final normal cases are0.962–1.278× saved Rust,median1.012×. Residual record,
  nested-string and recursive costs are now8–10µs in these batches,not zero.
- Two complete family parent/candidate/Node rounds rotate engines at identical
  executable paths:117 process observations/round,93 samples/engine/workload/round,
  max loads2.112/2.103,5.684s/88,352KiB. Earlier focused6-case repeats corroborate
  nested5.145×/5.282×,deep4.329×/4.524×,recursive3.784×/3.721×.
  Discard control drifts -2.9%/-4.4% in the family repeats,but its entire238,128-byte
  Mach-O __TEXT,__text section is bit-identical at the same address:SHA256
  2e44166b01e3c26cf10e1ad674e49f66a3092b6cfba8b5a21cbec72c01da4110.
  Source differs only by unused nth support;allocation profile is exactly equal.
- Consumer Rust compilation:original3 samples/input/engine/round,two rotated rounds
  at the same physical source/output path,35.680s/206,880KiB,max load2.4814.
  Recursive367.624→272.699ms and371.654→274.653ms (~26% faster);deep
  212.018→186.792 and211.474→186.206 (~12%);others stable or faster,unchanged
  control200.087→198.926 and199.436→200.163ms. No forced-inline code growth.
- Host compiler comparison:exact saved parent/candidate,same-path rotated11 samples
  per input/engine/round,2 rounds/132 outputs,216.895s/237,088KiB,quiet max2.561.
  Map wall4487.801→4455.678ms then4599.413→4604.701ms;user CPU+1.1%/+0.8%.
  Interfaces and Map emitted code match after removing unused nth support. Small
  launch timings fluctuate6–216ms under executable renames;do not infer a compiler
  regression or speedup from those wall values. Independent warmed fixed-path,
  identical argv0 check with interfaces,nested and64 repeated traversal functions:
  11 samples/engine/input/round,2 rounds,3.103s/42,656KiB,max load2.339. Scaled
  user CPU29.077→29.029ms then28.585→28.841;small nested3.313→3.241 then
  3.255→3.254ms. No material compiler regression;no compiler-emission speedup claim.
- Whole-project formatter4.131s/201,664KiB:zero source/snippet/fingerprint changes.
  npm run store:ingest added48 accepted parent/candidate entries from3 files;
  verified93 samples,exact rounded medians/mean rates,timestamp uniqueness and
  SQLite integrity. Only focused native/Node comparisons ran;no full matrix.
- Pruned104 closed files/168,049,690 bytes:passed test executables,exact parent
  compiler/native consumers,diagnostic probes,imported raw runs and scratch tools.
  Current release/debug compiler/LSP and13 native consumers/profiles remain;all
  owned workers are terminal. Deferred full-regression inventory is untouched.
- Next:large-gap milestone remains open. Prioritize scoped composition's remaining
  28.994625/18.420917ms and profiled widening carriers,then byte views/maps and
  expectation predicates. Non-void/discard traversal remains a separate opportunity
  (discard-string .042500/.023666ms saved Rust,1.796×). Preserve the deferred full
  regression snapshot;do not restart full suites until only small refinements remain.
  No public export,push,CI or release was performed.

**Map dense-count prototype — rejected at e6ef644c:**
- Tested count-preserving `map` collection for small dense `from_mapped` inputs;
  deleted-entry inputs retained the existing compacting path. Production source
  was not changed. Prototype compilation took58.146s/1,076,880KiB for timing and
  57.539s/1,111,664KiB for allocation instrumentation, under the watchdog.
- Two quiet same-path A/B rounds used the original3 processes×31 samples,
  8 warmups,2,048 iterations and focused Node controls. Parent/candidate medians
  were28.025750/33.483875ms and28.917250/34.181166ms:18–19% slower.
  Node149.842958/150.189709ms;maximum one-minute load2.992188.
  Timing watchdog56.961s/214,320KiB;all checksums and exact results matched.
- Three identical allocation samples per version explain the regression:
  allocations557,472 unchanged,reallocations2,056→122,892. Requested bytes
  fell79,711,904→68,908,352 but retained/peak stayed2,326,528 bytes;both
  cleanup counters stayed zero. Exact-size widening buffers immediately grow
  on later insertions. Do not repeat this approach as a memory optimization.
- Next profiles at unchanged e6ef644c:expectation predicates19,798 allocations,
  zero reallocations,534,528 requested bytes,zero retained,133 peak;byte-view
  Maps8,193 allocations,4,104 reallocations,1,461,888 requested bytes,
  180,224 retained/180,570 peak. Three equal samples each;cleanup zero.
  Profile watchdog4.647s/266,256KiB. Investigate expectation-constructor
  inlining before changing shared ownership or introducing a new storage layout.

**Constant-string concat results — accepted80ad63d3, parent36dbd769:**
- Previous goal turn made concrete progress:profiler timeout repair,accepted-source
  baseline import and rejection of three measured array-push variants. Source is
  clean at36dbd769 before this candidate;full suites remain deferred.
- Profile-first isolated generated-consumer probe preserves every receiver/argument
  setup and changes only the two fixed concat operations to static labels. Both
  exact nullish spread workloads drop9,563→1,367 allocations (8,196 removed),
  reallocations remain10;requested1,113,824 bytes,retained/peak688,256,input19,
  checksum752386989 and both disposal counters0 in three equal samples each.
  Watchdog3.127s/230,256KiB. Sources/executables are disposable
  target/runtime-comparisons/{profiles/,}*.static-concat{.rs,};results temporarily
  /tmp/tsrust-static-concat-probe.json. No trial timing has been accepted/imported.
- Private emit/constant_string.rs index follows immutable local declaration facts
  by checked binding ID,with borrowed string references (no recursive alias walk).
  Recognizes literals,nullish typeof and nullish String conversions;unresolved
  builtin names deliberately fall back. It changes only concat's final operation,
  retaining original operand code and all initialization/evaluation/errors.
  Rebindable/mutable/parameter/capture-reference/borrowed bindings fall back.
  Folded output over4,096 bytes uses unchanged runtime concat;this bounds duplicated
  emitted literals and never restricts source acceptance or Rust allocation limits.
- New O0/O3 tests in string_concat_padding cover effect order,immutable aliases,
  mutable fallback,typeof console/String,closure capture,bounds errors and large
  Unicode literal fallback. No public API/IntelliSense/ABI/limits/unsafe/dependencies
  change. Existing matched spread/concat benchmark support is reused.
- Candidate tests passed70/no failures or ignored:string_concat_padding7,
  array_literal_spread11,call_spread13,value_types8,string_construction4,
  string_concat_integer_fusion5,exception_handling22. Test durations3.614/4.153/
  5.215/5.161/2.072/3.117/5.708s,max RSS544,080KiB. Shared build204.499s/
  1,392,480KiB. All new O0/O3 witnesses passed. No broad suite was run.
- Parent preparation1.030s/259,040KiB verified exact generated source equality
  against the four retained36dbd769 consumers;two dynamic concat controls are
  emitted by the previous verified tool,with its known rejected push attribute
  reset to the committed runtime spelling. Parent native executables and generated
  consumers use .constant-parent{,.rs};source hashes in temporary
  /tmp/tsrust-static-concat-parents.json. This does not alter compiler source.
- Actual release rebuild/profile221.516s/1,415,760KiB,2026-11-03T00:57:32.779Z:
  both workloads match the isolated probe exactly,including checksum752386989,
  1,367 allocations/10 reallocations,three equal samples and both cleanup counters0.
  Emitted hashes:literal aa44543fd9f04e61ee1913b57010d811d61860060a21a6686067964e2fd099e4;
  call e0aa8047ad84a3af514a0034a2bb3c278c0479e5155916ec359c2e87bf0531a7.
- Matched baseline allocation observer restores only prior concat calls from the
  retained parent consumer (1.040s/67,936KiB). Both:9,563→1,367 allocations
  (-85.7%),reallocations10 unchanged,requested1,236,764→1,113,824 bytes,
  retained/peak811,196→688,256 bytes (-15.2%),input19,cleanup0. Three equal
  samples per workload. No changes to small-array layout or thresholds.
- Formal native/Node run2026-11-03T00:57:42.618Z,9.852s/334,304KiB,
  93 samples/engine/workload,quiet max2.70947265625,no override:
  literal .063709/.240375ms;call .063166/.256667ms. Saved Rust references
  .032208/.032000ms are reused,not retimed (remaining1.978×/1.974× gaps).
- Two rotated same-path parent/candidate/Node rounds,93 samples per engine per
  workload per round,54 complete process observations each:literal3.414×
  (repeat3.433×),call3.448× (repeat3.321×). Watchdog10.334s/203,360KiB;
  quiet maxima2.580078125,both rounds,no override,checksums/exactBits unchanged.
  All four controls have byte-identical generated source:interfaces,
  nullish_array_value,string_concat_zero_ascii,string_concat_variadic_unicode.
- Consumer Rust compilation uses identical physical source/output paths,rotated
  order,original3 samples per engine/workload,2 rounds (21.250s/174,272KiB):
  literal282.567→271.059ms then289.925→281.019ms;call284.254→275.722ms then
  286.797→278.288ms (3–4% faster). Unchanged-source control timings vary0–3%;
  no compiler-emission speedup is claimed. Binaries shrink1,696/1,680 bytes.
- Concat hover's old independent-buffer guarantee was too strong for static text.
  Reworded only the short ownership note;behavior/signatures/examples unchanged.
  Focused intelligence hover/signature/completion test passed1,330 filtered,
  watchdog .515s (test .05s). Its dependency/catalog build217.805s/1,483,056KiB;
  the compiler documentation API also embeds builtins.json. The documented
  Hello/world example emits the folded literal and prints Hello world under O3
  (.519s/1,440KiB observed watchdog). All70 compiler tests remain applicable;
  the subsequent catalog wording change does not change emitted workloads.
- Final safety review probed string reassignment through a parameter and through
  a captured const on an exited branch. Both reject before emission:only let
  bindings can be rebound. Let bindings are never indexed;unknown/mutable/captured/
  borrowed bindings fall back. Probe .520s/1,840KiB;two disposable source probes
  were removed after the main cleanup. No implementation change was needed.
- Whole-project formatter4.119s/201,488KiB:zero source/snippet/fingerprint changes.
  npm run store:ingest added12 entries;verified93 samples,exact six-decimal medians,
  three-decimal mean rates,timestamp uniqueness and SQLite integrity.
- Pruned59 closed files/186,648,130 bytes:passed test executables,matched parent/
  trial consumers,imported raw runs and scratch tools. Current compiler/LSP and six
  generated consumers/profiles remain;all owned workers are terminal.
- Next:remaining large gaps stay open. Inspect the residual spread container loops,
  Array<String> layout/capacity and reference preallocation before further changes;
  do not equate allocation reduction with parity. Scoped composition remains
  28.994625/18.420917ms;return to profiled widening carriers or the ranked record/
  byte-view/testing gaps. Continue affected suites and quiet A/B only;full regression
  snapshot remains deferred and preserved. No public export,push,CI or release.

**Fresh large-gap profiling and rejected array-push experiments — parent 600498f9:**
- Full suites remain deferred under the owner milestone rule above. Refreshed
  scoped composition is28.994625ms versus saved Rust18.420917ms;Node152.112833ms
  (2026-11-02T23:45:39.402Z,93 samples,quiet maximum3.40234375).
  Actual emitted hash89f2eabe9a1514c801267b8ee815c8cf9d6f239b4a2b8df4dc085df791c65068
  is unchanged from the accepted Map gate. Profile63.740s/1,075,536KiB:
  557,472 allocations/2,056 reallocations,79,711,904 requested bytes,
  2,326,528 retained/peak,checksum108435446,three equal samples,cleanup0.
- Allocation-neutral transition instrumentation (57.024s/1,076,640KiB) finds
  Option slot32→40 widening178,180 times at live length1;40→32 length1 occurs
  258,052 times. Other counts:32→32 length1=112,232;32→40 length0=18,841;
  40→32 length0=3,276;40→40 length0/1/2=10,649/224,053/124,110.
  Widening callback carriers remain an open buffer-reuse cause. Empty-buffer
  array_chunks reshape was rejected:E0658 on pinned stable;no unsafe/unstable
  workaround or production change. Uniform padding needs separate evidence.
- Correct remaining spread IDs:array_literal_spread_nullish_packs and
  call_spread_nullish_packs;nullish_array_value is a separate fast control.
  Fresh baseline2026-11-03T00:16:15.566Z,93/engine,quiet maximum2.11865234375,
  9.346s/303,504KiB:nullable_boolean_undefined .014833ms/Node .025125ms;
  selector .010000/.014083ms;literal spread .209167/.232000ms;
  call spread .202375/.229709ms. Saved Rust .003625/.039583/.032208/.032000ms.
- Both exact spread workloads make9,563 allocations/10 reallocations
  (profile2026-11-03T00:12:42.165Z,4.167s/251,056KiB). Repeated labels
  object:null and undefined:undefined allocate;future concat folding must retain
  receiver/argument evaluation,including original indexing failures and effects.
  Nullable boolean has1 allocation/10 reallocations,196,608 retained/peak bytes;
  selector1/10,393,216 bytes (00:08:05.795Z,4.177s/305,424KiB).
- nm confirms an out-of-line owned Array<T>::push in nullable boolean. Isolated
  generated-consumer #[inline(always)] trial:two quiet same-path rounds,
  93 samples/engine/workload/round,rotating parent/candidate/Node,checksums and
  exactBits unchanged. Nullable boolean1.915×/1.594×,selector1.678×/1.703×;
  literal spread1.025×/1.011×,call spread1.019×/1.051×. Quiet maxima1.917/1.843,
  watchdog8.801s/247,984KiB. First trial preparer stopped on an ambiguous helper
  match before timing;repaired to identify owned Some slots,completed2.589s/
  151,344KiB. That trial was later rejected;canonical consumers were restored below.
- Actual-compiler forced-inline timing2026-11-03T00:36:56.586Z (10.309s/
  318,000KiB) and two same-path repeats confirm nullable boolean2.011×/1.964×,
  selector1.689×/1.689×. However,three compile samples per workload/engine/round,
  two rotated rounds at identical source/output paths (22.637s/216,656KiB)
  find repeatable compiler costs:nullable boolean250.761→279.339ms then
  255.827→283.392ms;selector205.722→223.120 then207.168→224.049ms;
  spreads5–7% slower to compile. Boolean executable grows511,208→527,592 bytes.
  **Rejected;owned Array<T>::push source and all six canonical consumers restored.**
- Two bounded alternatives outline inline-storage growth into a cold helper.
  Ordinary inlining yields boolean1.122×/1.114×,selector1.054×/1.191×;
  compile costs remain0–2.5% higher (22.124s/214,224KiB). Forced fast-path
  inlining yields boolean1.530×/1.547×,selector1.342×/1.339×,with mixed or
  slower object controls. Neither replaces the original implementation. Each
  rotated runtime round used93 samples/engine/workload with Node controls,
  checksum/exactBits assertions and quiet maxima below4. Trial allocations are
  unchanged. These rejected runs are not imported as accepted compiler results.
- Trial validation passed120 tests/no failures or ignored:fixed_storage34,
  array_literal_spread11,call_spread13,arrays12,owned_arrays12,nested_arrays12,
  nullable_unions4,exception_handling22. Each binary separately,three threads,
  watchdog5GB/30min;test durations20.744/4.166/5.204/3.627/4.670/4.654/3.615/
  5.717s,max test RSS619,392KiB. Initial shared build201.574s/1,356,464KiB.
  A nonexistent nullable_primitives target was corrected before any test ran.
  These validate the discarded attribute trial,not a new optimization milestone.
- **Profiler runner defect fixed:**Cargo inherited the ordinary120-second child
  timeout,orphaning owned rustc20892 at PGID20840. Verified identity and stopped
  only that compiler. Cargo now gets30 minutes;ordinary consumers retain120s.
  Successful actual six-workload rebuild/profile229.720s/1,507,040KiB proves the
  longer path;all three allocation samples per workload agree and dispose cleanly.
  Initial timeout120.545s/1,456,336KiB was a harness failure,not a compiler failure.
- Next:profile/compiler work for known string concatenation in exact nullish spread
  workloads. Retain original operand evaluation,alias/ownership constraints,bounds
  failures and effects. Reuse immutable-local facts where sound;unknown values
  fall back. Do not impose source acceptance limits or retry forced array inlining
  without a different cause. At that checkpoint,release/debug tools still contained
  the rejected attribute trial;the subsequent accepted constant-concat gate rebuilt
  from restored source. The rejected trial never entered production compiler source.
- Finalization:whole-project formatter4.150s/202,240KiB,zero source/snippet/
  fingerprint changes. npm run store:ingest added16 accepted-source baseline
  histories;verified24 histories including8 prior entries,with exact six-decimal
  medians,three-decimal mean rates,93 samples,timestamp uniqueness and SQLite
  integrity. Initial verification expected16 total instead of16 new+8 existing;
  corrected the inventory without changing any stored value. Rejected timing was
  excluded. Pruned74 closed files/192,165,279 bytes,including eight test binaries,
  imported raw runs and generated trial consumers/tools;canonical six consumers
  retained. No full suites,public metric export,push,CI or release performed.

**Read-only scalar reduction slices — accepted locally, parent 8cafb9f8:**
- Profiled before edits:2026-11-02T22:28:00.324Z,4.154s/246,304KiB. Each
  declared int/Number/boolean reduction makes1,536 allocations/no reallocations,
  retained0;int/Number request40,960 bytes/peak40,boolean5,120/peak5.
  Checksums401739032/820452567/0,three equal samples,both cleanup counters0.
  Callback wrappers are simple forwarders;owned temporary slice ranges explain
  the allocation cost. Matched unchanged Rust references borrow those ranges.
- Generated-consumer borrowed-range probe2.611s/128,064KiB removes every heap
  call and requested/retained/peak byte,with identical checksums and clean disposal
  in three samples per workload. This is allocation diagnosis,not formal timing.
- Private scalar_reduce_slices index proves immutable owned local scalar sources
  and read-only local int/Number/boolean slice results. **Every source use must
  be a checked slice receiver**,and every result use a direct reduce/reduceRight
  with a concrete non-rest closure accepting fewer than four arguments. Native
  view loans,other source calls/aliases/moves/writes,escape,captures,rebinding,
  whole-array callback ABI,unknown contracts and exception regions keep ownership.
  Bounds still run once at the original declaration,in order,using unchanged
  normalization. Reuse existing visitors/capture checks and slice emission. No
  public API,ABI,limits,small-storage thresholds,RC,unsafe or dependencies change.
- **Caught/fixed a lifetime defect before acceptance.** Final binding metadata
  can forget a source move on an exited branch. Parent accepts/executes the owned
  snapshot (3 3),but the unguarded trial emittedE0505. Probe2.592s/153,136KiB.
  The all-source-use proof above prevents it;permanent O0/O3 exiting-move witness
  now passes. Initial unguarded runtime wins are discarded,not ingested. Capture
  scans also run only after real slice candidates exist. Early build resources:
  201.147s/1,391,584KiB and207.221s/1,413,456KiB;unguarded release219.098s/
  1,426,624KiB. Earlier53-test passes lacked that witness and are not final proof.
- **53 affected tests pass on guarded source**,four sequential watchdog binaries,
  --test-threads=3:array_slice7 incl exited-move witness (9.362s/464,736KiB),
  array_reduce7 (4.688s/476,560KiB),declared_function_values5 (10.398s/
  546,160KiB),fixed_storage34 (20.826s/482,240KiB),no failures/ignores. Each
  identity counted once. Guarded build201.409s/1,462,576KiB;release218.382s/
  1,544,416KiB. Native O0/O3 checks cover zero-copy allocations,heap sources,
  retention,extreme signed bounds,zero/one/two-argument defaults,empty/no-seed
  failure/unwind,both directions,NaN/signaling-NaN/-0/infinity bits,bound operand
  effects and ownership/receiver/capture/exception fallbacks. All runs alone under
  5,000,000KiB/1,800s,one Cargo job,incremental off,shared target.
- Corrected actual profile**2026-11-02T23:19:17.598Z**,4.154s/302,560KiB:
  all three reductions have**zero allocations/reallocations/deallocations and
  requested/retained/peak bytes**,cleanups0,three equal samples. Target emission
  matches the winning trial exactly,while the exited-move case now falls back.
  Source hashes:int23d5971615fc11c3e65a4e95955ac8f348e119eecc3ba73d7fd209b63222d4f0;
  Number7cf9fce3cb035c3850fa030e0985d88663c550d48f5798192aae4b62efda53ee;
  boolean952f5ddd17d149ca11791b7f5c9d7c29f675d25232157252b0cde25340ef7529.
  Integer reversal control source/counters unchanged;complete242,048-byte Mach-O
  __TEXT,__text is bit-identical,hash4aff5d81a3af08afdfe0781d2460536176fa4f0c1675801cbc37cf63a5a15229.
- Formal parent**2026-11-02T22:30:49.554Z**,6.244s/270,272KiB:
  Number native.031916ms/Node.025708;boolean.028917/.028542;
  int.035667/.075750;integer reversal.002917/.008291. Corrected candidate
  **2026-11-02T23:19:31.248Z**,4.145s/249,392KiB:
  Number**.001791ms,17.820×**,Node.025125;
  boolean**.001542ms,18.753×**,Node.031750;
  int**.007083ms,5.036×**,Node.083334;control.002917/.007792.
  Original93 samples/engine,three processes/eight warmups/31 samples;quiet maxloads
  2.38818359375/2.38330078125,no override. All24 corrected process observations
  and original correctness cases agree with parent;no coverage reduction.
- Two same-physical-path/argv0 rotating parent/candidate/Node rounds,93 samples
  per engine:Number**.032708→.001958ms,16.705×**,repeat**.032292→.001791ms,
  18.030×**;boolean**.029041→.001667ms,17.421×**,repeat**.029083→.001542ms,
  18.861×**;int**.035375→.007084ms,4.994×**,repeat**.035417→.007083ms,
  5.000×**. Node Number.031000/.025458,boolean.031083/.028583,int.079083/
  .074083ms. Control.002959→.002916 then.002958→.002916ms. Machine waited
  once at load4.05;quiet maxima3.9033203125 both rounds,no override. All36
  complete observations per round agree;16.068s/68,768KiB including quiet wait.
- Additional **guarded compiler emission A/B**,11 samples/engine/input per round,
  132 complete outputs,210.979s/162,032KiB;quiet maxima2.56494140625/
  2.70947265625. Interfaces10.554917→9.621958ms,then9.374084→12.217541;
  scoped composition4517.725083→4545.606666,then4668.083125→4624.719583;
  Number reduction205.426750→175.186250,then175.151125→209.136125.
  Unchanged controls are byte-identical (42,980/3,556,579 bytes). Short startup
  timings vary in opposite directions. Independent child CPU comparison,21 runs
  per engine/round,identical interface output,1.559s/20,128KiB,maxload1.7294921875:
  wall5.840791→6.053375 then5.675375→5.556166ms;user3.044→3.237 then
  2.991→2.927ms;system1.559→1.690 then1.493→1.438ms. No sustained compiler
  regression is observed;do not claim a compile speedup. Parent CLI identity was
  independently preserved,hash209917771034d65205229f644e6414e117a7daa58f9841962628d1ccfefe20d9.
- store:ingest adds**16 histories**,eight parent/eight corrected candidate.
  Verified exact median6/mean-rate3/93sample fields,unique timestamps,all complete
  observations,quiet policy,rejected-trial exclusion and SQLite integrity. Saved
  Rust references.001584/.002167/.007333ms are reused,not retimed;new results
  are1.131×/.712×/.966× those references. Number/boolean large gaps are closed
  for these measured cases. Whole-project formatter changes zero files/snippets/
  fingerprints,4.129s/198,752KiB. Public metrics remain deferred until final
  optimization milestone;no broad regressions,CI,push or deployment.
- Next:refresh scoped composition under current source (its stored large-gap row
  predates loop-transfer optimization),then profile remaining nullable-boolean,
  nullish/call-spread and record/byte callback gaps. Keep the owner-defined
  milestone open until substantial gaps are addressed/explained. Closed trial/
  parent/probe consumers,passed test binaries,imported raw runs and scratch
  diagnostics are pruned (38 files,127,184,623 bytes);current CLI/LSP,latest
  profile/current native consumers,shared target and deferred regression state
  remain. All owned jobs terminal.

**Dead clone/push loop transfer — accepted locally, parent 7375144b:**
- Previous turn made measured progress,committed7375144b;clean worktree at start.
  Owner milestone policy remains:affected checks only;no broad regression restart.
- Fresh allocation profile2026-11-02T22:10:43.632Z,4.157s/243,296KiB confirms
  current recursive reversal6,273/with6,657 allocations,903 reallocations each,
  retained195,584/peak197,480,all cleanup counters zero. Generated for-of still
  borrows a dead local's elements and clones each into its output.
- Generated-consumer-only probe reuses existing TsrOwnedItems adapters,no runtime
  invention:reversal2,433 allocations/seven reallocations/295,264 requested bytes;
  with2,817/seven/318,304. Both retained195,584/peak195,920,input12,
  checksums988007362/421497852,three identical samples,both cleanup counters0.
  Probe1.041s/152,688KiB;this diagnoses allocation work,not formal timing.
- Fresh formal parent2026-11-02T22:12:45.684Z,6.756s/268,128KiB:
  reversal native.149916ms/Node.033917ms;with.160459/.039584;
  nested reversal.006542/.010750;integer control.002958/.007792.
  Original93 samples/engine,quiet maxload2.830078125,no override. Four parent
  executable consumers retained as target/runtime-comparisons/<id>.transfer-parent.
- Trial extends the existing dead-const/single-reference proof to a for-of whose
  sole body expression is destination.push(clone(element)). Require named distinct
  source/destination,same checked array type,no captures/borrowed source/rebinding,
  same statement list and iteration. Emission additionally excludes Copy elements,
  unsupported fixed layouts,fixed destinations and storage resets. Reuse existing
  tsr_owned_items and existing push,without changing layouts,growth policy,API,
  ownership,limits or deep iterative cleanup. Multi-statement/effectful loops,
  live/captured/repeated sources and exception-region fallback keep borrowing.
- **29 affected tests pass**,five sequential binaries,--test-threads=3:
  receiver_moves7 (4.672s/295,776KiB),for_of10 (2.607s/391,648KiB),
  array_copying4 (2.667s/527,184KiB),owned_disposal1 (1.553s/170,208KiB),
  recursive_storage7 (2.087s/390,736KiB). Build205.127s/1,546,560KiB.
  All under5,000,000KiB/1,800s watchdog,shared target,one Cargo job,incremental
  off. O0/O3 witnesses prove only one source snapshot of selected strings,
  preserved independent owners,normal/unwind byte cleanup and100,000-level
  transfer/drop. Live/captured/rebindable/repeated/effectful/borrowed/exception
  fallback witnesses pass. No test failures or ignores.
- Release217.837s/1,530,912KiB. Actual allocation profile
  **2026-11-02T22:23:30.622Z**,4.673s/284,720KiB matches the probe dictionaries
  exactly:reversal2,433 allocations/with2,817,seven reallocations each. Removes
  **3,840 allocations and896 reallocations each**,requested bytes drop635,904
  each;retained195,584 unchanged,peak195,920 (1,560 lower). Cleanup counters0,
  all three samples equal. Nested129 allocations/seven reallocations,requested
  134,784/retained49,152/peak49,440,checksum856074192,cleanups0. No fresh
  nested parent allocation delta is claimed. Integer source and every allocation
  counter are byte-identical to parent. Actual recursive source hashes:
  reversal1fb2067279dfb0824989180d6301e39fb3787484fea7d84e1b59d7ff40a53428;
  with5dd7388016dc89ad74d4b219e63f6a25b5d2d48d86461c798a0fb06d852b1cb3.
- Final formal candidate**2026-11-02T22:23:42.828Z**,4.662s/295,344KiB:
  reversal**.149916→.030458ms,4.922×**,Node.033917→.034833;
  with**.160459→.039667ms,4.045×**,Node.039584→.039417;
  nested.006542→.006541,Node.010750→.011083;
  integer.002958→.002958,Node.007792→.007958. Original93 samples/engine,
  quiet maxload1.89453125,no override;all24 complete candidate process
  observations match parent and original correctness cases pass.
- Two rotating same-physical-path/argv0 rounds,93 samples per engine:
  reversal**.150125→.030917ms,4.856×**,repeat**.149458→.031209ms,4.789×**;
  with**.163625→.039958ms,4.095×**,repeat**.162958→.039500ms,4.126×**.
  Node reversal.033583/.034000,with.039417/.039500ms. Nested.006917→.007208
  then.006709→.006542;integer.003042→.003167 then.002958→.003041ms.
  Tiny control variation is contextualized by its **bit-identical**242,048-byte
  Mach-O __TEXT,__text,hash4aff5d81a3af08afdfe0781d2460536176fa4f0c1675801cbc37cf63a5a15229.
  Quiet maxloads1.62744140625 both rounds,no override;36 complete outputs agree
  per round,2.589s/164,320KiB total. No sustained nested regression is observed;
  formal nested result is unchanged within timer resolution.
- store:ingest adds**16 histories**,eight parent/eight candidate. Independently
  verified unique timestamps,exact median6/mean-rate3/93sample fields,quiet policy,
  full process observations and SQLite integrity. Saved Rust.027500/.034833ms
  reused,not retimed. Current formal results are1.108×/1.139× those saved
  references,closing the prior5.798×/4.976× recursive-copy gaps for these cases.
  Public metrics refresh remains deferred. Whole-project formatter changes zero
  files/snippets/fingerprints,4.133s/202,688KiB.
- Follow-up:the scalar reduction gaps are now closed by the read-only slice
  optimization above. Recursive-copy residuals are small for these measured
  cases;other recursive/record callback gaps remain queued. Follow the current
  queue before profiling. No broad regressions,CI,push or deployment here;all
  owned runners terminal.
- Pruned27 closed files (118,788,974 bytes):parent/probe consumers,passed test
  binaries,verified imported raw runs and scratch diagnostics. Latest four-workload allocation diagnosis,
  current native consumers,shared target and deferred full-regression state stay.

**Dead owned-array copying receivers — accepted locally, parent 1a74cf8f:**
- Owner milestone policy above remains active: no broad suite restart. The Map
  gate is committed as1a74cf8f; working tree was clean before this section.
- Fresh allocation profile2026-11-02T21:45:58.503Z,4.689s/250,592KiB:
  recursive toReversed10,113 allocations/1,671 reallocations/1,481,056 requested
  bytes;with9,857/1,543/1,412,448. Both retain195,584/peak198,560 bytes,input12,
  checksums988007362/421497852,all three dictionaries equal,both cleanup counters
  zero. Integer reversal control129 allocations/five reallocations,retained8,192,
  peak8,240. Originaln128/seed7. Actual generated code copies fixed-array owners
  into the method result,then separately clones each borrowed iteration element
  into the output. The hand reference legally consumes each dead receiver and
  extends its output;its saved baseline is reused,not retimed.
- Fresh formal parent2026-11-02T21:46:45.333Z,6.222s/236,096KiB:
  recursive toReversed native.273708ms/Node.033667ms;with.260042/.039375ms;
  integer reversal.002958/.007875ms. Original93 samples/engine,three processes,
  eight warmups,quiet maxload2.77392578125,no override. Not the historical row.
- Narrow implementation reuses the existing dead-const/single-reference proof.
  Admit fixed owned-record receivers in copying emission,and admit with() in
  the proof. Reverse owned storage or replace its validated element before
  converting a fixed array into its existing public result layout. Live,
  captured,borrowed,rebindable,loop-carried and unsupported layouts keep copying.
  No new API,limits,dependency,RC or unsafe. Ordinary for-of clones remain a
  separate residual optimization;do not claim they were removed.
- **44 affected tests pass**,four sequential binaries,--test-threads=3 under
  5,000,000KiB/1,800s watchdog:array_copying4 (2.618s/483,088KiB),receiver_moves5
  (2.091s/473,824KiB),array_copying_owned_disposal1 (1.551s/160,240KiB),
  fixed_storage34 (20.760s/507,392KiB). New O0/O3 witnesses cover fixed
  scalar/recursive/heap results,independent ownership,NaN/-0/infinite bits,
  failure operand order and exact live-byte cleanup after owned bounds unwinds.
  Added with() variants to existing live/captured/rebindable/loop-carried proof
  witnesses;all5 receiver tests pass again3.628s/342,064KiB,not counted twice.
  Builds200.016s/1,391,840KiB,3.121s/481,808KiB,then test-only1.561s/270,512KiB.
  Caught a whitespace-sensitive new fixture replacement during review and used
  a separate formatted failure fixture before execution;no test failure occurred.
- Initial trial release214.237s/1,495,312KiB. Candidate allocations6273/6657
  versus10113/9857;retained unchanged and peak197480,all cleanup checks pass.
  Formal recursive timing.273708→.155625ms and.260042→.171625ms;paired
  reversal1.792×/repeat1.798×,replacement1.628×/repeat1.584×. Scalar fixed
  reversal gave no memory gain,formal.002958→.002875ms but both paired rounds
  .002958→.003000ms (one timer tick). Reject that unnecessary scalar expansion:
  restrict fixed transport to owned-record arrays and preserve scalar emission.
  Revalidated the narrowed source before acceptance,as recorded below. That
  initial scalar trial is superseded;its parent consumers have been pruned.
- Narrowed final source:**all44 affected tests pass** again,array_copying4
  (3.134s/429,136KiB),receiver_moves5 (3.644s/191,552KiB),owned_disposal1
  (1.556s/164,400KiB),fixed_storage34 (20.770s/452,704KiB). Rebuild199.883s/
  1,554,288KiB plus3.629s/305,328KiB for remaining links. Earlier passes are
  historical validation,not additional tests. Final release219.022s/1,477,456KiB.
- Final allocation profile**2026-11-02T22:07:51.817Z**,3.120s/290,368KiB:
  reversal6,273 allocations/903 reallocations/931,168 requested bytes;
  replacement6,657/903/954,208. Removes**3,840/3,200 allocations** and
  549,888/458,240 requested bytes. Both retain195,584 bytes,peak197,480
  (1,080 lower),all three samples identical,both cleanup counters zero.
  Recursive emitted sources match the initial winning trial exactly:
  reversal2c6ab6d508218d1916d514efb42ac46971519fb770ede27bc89ab77b56fa0d37;
  replacementef0dd42bb91b746fa178e07adf711dcd1607bfd3b8b59193dd0cae276dcdd087.
  Scalar emission matches parent7811df8fa6aa15e0e365f63c3b48407a7702d2a04e9a2fdc11202920840a977e,
  including its complete allocation dictionary. Its242,048-byte Mach-O
  __TEXT,__text section is bit-identical,hash
  4aff5d81a3af08afdfe0781d2460536176fa4f0c1675801cbc37cf63a5a15229.
- Final formal candidate**2026-11-02T22:08:04.914Z**,4.147s/290,384KiB:
  recursive reversal**.273708→.159458ms,1.716×**,Node.033667→.034416ms;
  replacement**.260042→.173333ms,1.500×**,Node.039375→.040334ms;
  integer control.002958→.002917ms,Node.007875→.007875ms. Original93 samples
  per engine,quiet maxload2.2626953125,no override. All18 candidate process
  observations match complete parent outputs;original correctness cases pass.
- Final same-physical-path/argv0,rotating parent/candidate/Node,93 samples/engine:
  reversal**.293583→.150833ms,1.946×**,repeat**.273917→.152584ms,1.795×**;
  replacement**.271833→.166416ms,1.633×**,repeat**.268250→.169083ms,1.586×**.
  Node reversal.036917/.034291ms,replacement.042625/.040875ms. Quiet maxima
  2.4990234375 both rounds;27 complete outputs agree per round,2.581s/
  159,152KiB total. Bit-identical integer code times.002959→.003125 then
  .002917→.002917ms;this illustrates noise in the tiny control,not code changes.
- store:ingest adds**12 histories**,six parent/six final candidate. Independently
  verified each exact rounded median/rate/93sample count,unique timestamp and
  SQLite integrity. Rejected scalar-trial raw result is excluded. Saved Rust
  references.027500/.034833ms are reused,not retimed. Public report refresh
  remains deferred until the final optimization milestone. Final whole-project
  formatter changes zero files/snippets/fingerprints,4.136s/202,768KiB.
- Remaining gap is still material:formal reversal5.798×/replacement4.976× saved
  Rust. Generated for-of borrows each element and explicitly clones it into the
  output;the Rust reference moves those dead owners and pre-reserves output.
  Next:profile that remaining loop transfer,trace caller/borrow/capture and
  exception constraints before reusing or extending the existing move proof.
  Do not weaken independent ownership or iterative deep cleanup. Other large-gap
  review targets remain queued;do not restart full regressions yet.
- Closed parent consumers,passed test binaries,imported raw runs and scratch
  trial/profile diagnostics are pruned after verification. Latest allocation
  diagnosis,current candidate consumers and shared CLI/LSP target remain. No
  broad regressions,CI,push or deployment;all owned runners are terminal.

**Small Map carrier buffer reuse — accepted locally, parent 3fa9cd8d:**
- Profiled the remaining scoped_record_composition gap before edits. Three
  identical generated samples: 1,275,919 allocations/2,056 reallocations,
  182,814,336 requested bytes, 2,326,528 retained/peak bytes, checksum108,435,446,
  both cleanup counters zero. Profile59.896s/962,768KiB. Matched unchanged Rust
  reference:886,791 allocations/2,057 reallocations,36,251,240 requested bytes,
  2,277,376 retained/2,277,968 peak,identical checksum and cleanup;compile
  3.639s/184,256KiB,profile.515s/2,608KiB. This is allocation attribution,not a
  fresh Rust timing baseline. The profiler stdout-cap fix is checkpoint3fa9cd8d.
- Size/tag probes preserve every original counter in three samples per engine.
  Map-slot growth accounts for generated1,202,190 requests/178,063,360 bytes
  versus reference213,402/27,315,456. Generated blocks are160 bytes (755,720
  requests) or128 bytes (446,470);reference slots128 bytes. Compile51.799s/
  847,312KiB and2.079s/172,240KiB,profile1.035s/32,336KiB. Emission contains
  1,222 value-carrier transports rebuilding Maps. CPU sampling confirms Map::set,
  Vec growth,malloc/free activity;130 complete observations match the independent
  831,486-byte oracle. CPU compile52.879s/849,664KiB,sample7.234s/38,656KiB.
  Initial diagnostic preparation duplicated an observer and assumed output>1MiB;
  both probe mistakes were corrected before using its evidence. Sampling flags
  mean those durations are not benchmark measurements.
- Add private from_mapped transport to existing TsrMap. For the existing small
  storage form,standard Vec filter_map/collect compacts holes and can reuse the
  allocation while converting value types. Indexed Maps retain the original
  owned-iterator/rebuild path,including early drop of their old bucket table.
  Change only entry-epoch and completion-storage emission;keys,order,ownership,
  public APIs,limits and thresholds remain unchanged. No new dependency,RC or
  generated unsafe. The indexed fallback is explicitly documented in code.
- **253 affected tests pass**,three sequential binaries,--test-threads=3:
  maps7 (3.111s/480,160KiB),scoped_callable_records42 (7.860s/562,320KiB),
  scoped_record_composition204 (202 initial passes,173.885s/662,608KiB;two source
  guards repaired to match actual from_mapped call sites rather than the generic
  helper body,then pass2.588s/186,176KiB and1.550s/189,472KiB). Native disposal
  witnesses remain intact;do not double-count reruns. New O0/O3 witnesses cover
  buffer reuse,empty/sparse maps,threshold transitions,indexed fallback,changed
  payload layouts,NaN/-0 keys,ordering,mutation and move-only panic cleanup.
  Final source build201.431s/1,581,408KiB;release217.332s/1,446,928KiB. An earlier
  build203.913s/1,243,728KiB preceded the final fallback drop-order correction.
- Actual candidate profile **2026-11-02T21:18:26.196Z**,64.763s/1,190,736KiB:
  557,472 allocations (**718,447 fewer,-56.3%**),2,056 reallocations unchanged,
  requested79,711,904 bytes (**103,102,432 fewer**),retained/peak2,326,528
  unchanged. All three samples and cleanup checks agree. Control emitted source
  is byte-identical:2,049 allocations/eight reallocations/310,912 requested bytes/
  212,992 retained. Composition emission hash89f2eabe9a1514c801267b8ee815c8cf9d6f239b4a2b8df4dc085df791c65068.
- Fresh formal parent **2026-11-02T20:49:53.668Z**,101.059s/950,576KiB:
  composition native45.633209ms/Node151.257375ms;control.034000/.068625ms.
  This replaces the historical121.497209ms as the acceptance baseline. Candidate
  **2026-11-02T21:21:22.367Z**,87.490s/1,119,552KiB:29.321042/151.226125ms,
  **1.556× faster**;control.033959/.063959ms. Original93samples/engine,
  quiet maxloads3.775390625/2.22900390625,no override. All twelve candidate
  process observations match complete parent outputs and unchanged source hashes.
- Same physical executable path/argv0,rotating parent/candidate/Node order,
  three fresh processes/eight warmups/31samples (93 per engine):
  **46.047042→28.077541ms,1.640×**,Node148.859791ms;repeat
  **46.430125→28.889667ms,1.607×**,Node149.638833ms. Nine complete consumer
  observations agree per round. Quiet maxloads2.54443359375/2.50048828125,
  no override;58.946s/276,880KiB total. Control's complete244,816-byte Mach-O
  __TEXT,__text section is bit-identical to parent,hash
  582a918bba27e337010dcd36ccd991f0ed5d1c224bedc8f28d137fe9f347be06.
- store:ingest adds **eight exact histories** (four parent,four candidate).
  Verified unique timestamps,median durations,mean rates,93sample counts and
  SQLite integrity. Saved hand Rust18.420917ms is reused;no new Rust timing.
  Public performance-page pins/export/build remain deferred until the final
  optimization milestone,as requested. Whole-project formatter changes zero
  files/snippets/fingerprints (4.128s/201,936KiB). Pruned193 closed files (252,100,256 bytes):
  obsolete probe/parent consumers,passed test binaries,verified imported formal
  runs and disposable scratch diagnostics. Current release/debug CLI/LSP,current
  candidate native/profile consumers and latest allocation diagnosis remain.
  Deferred regression operational state remains available. No broad regressions,
  CI,push or deployment;all owned jobs are terminal.
- Next:profile remaining large gaps from the review queue below. Start with fresh
  recursive-array copy controls and audit equivalent work in the Rust reference;
  keep the residual Map gap on the queue. This accepted win does not finish the
  optimization milestone. All test/build/profile/timing runners above are terminal.

**Deferred broad regression snapshot — source 250e1d24 plus test-only repairs:**
- Owner stopped this pass until the remaining large gaps are addressed and only
  small refinements remain. Preserve **419 completed targets,4,991 passed,
  18 ignored**,not a completed/current-source full milestone. single_use_callbacks
  was interrupted during build,not a failed test. The old inventory is494 targets:
  399 compiler integrations,91 LSP integrations,two libraries and two binaries.
  Six unchanged source250e1d24 suites contribute76 reused passes exactly once.
- Both libraries/binaries passed:compiler library1,298/eight ignored (build
  261.247s/1,659,792KiB,test72.548s/1,445,808KiB),LSP library20 (build182.219s/
  1,928,656KiB,test1.042s/24,112KiB),CLI40 (build12.940s/609,600KiB,test7.233s/
  359,936KiB),LSP binary six (build6.217s/625,904KiB,test.523s). All91 LSP
  integrations:836 passed/three ignored. checker_agreement3,519.883s/100,048KiB;
  completion_matrix3,498.963s/50,288KiB;editor_consistency27,14.065s/113,008KiB;
  intelligence329/two ignored,646.628s/896,048KiB. Third LSP ignore:project_scaling.
  No matrix row filter/report-only mode was active.
- Snapshot compiler acceptance includes all24 JSON/JSON Lines targets128 passes;
  HTTP13/one ignored,316.209s/1,448,800KiB;native declarations64,154.041s/
  1,111,008KiB;dependency bundle2,23.347s/340,640KiB;native package export7,
  333.490s/1,107,168KiB;shared sources6,44.418s/543,040KiB. These results
  belong to that source snapshot;assess affected coverage against later changes.
- Earlier snapshot repairs:array_of assumed Vec<f64> instead of the accepted
  fixed return;all4 pass after test-only repair92715b93 (build3.628s/307,888KiB,
  test14.542s/474,544KiB). Compound/update observers had the same private-layout
  assumption;all8 pass after eb91e84f (build1.039s/302,464KiB and1.040s/268,480KiB,
  test2.084s/391,376KiB and2.079s/506,288KiB). Native O0/O3 bit/allocation/cleanup
  checks remain intact. Partial failed-binary passes were never added twice.
- Direct-run environment corrections:use Cargo's per-crate test working directory
  (array_to_string4 pass3.633s/332,512KiB) and explicit resolved RUSTUP_HOME
  (native_cache9/one ignored6.774s/364,640KiB). Pinned Rust was installed;the
  isolated PATH fixture changes HOME. No compiler changes for either correction.
- Each build/test ran alone under5,000,000KiB/1,800s process-group watchdog,
  one Cargo job,CARGO_INCREMENTAL=0,shared target,--test-threads=3 and explicit
  prepared portable-source packs. No watchdog stops. Completed executables were
  pruned. Owner-stop terminated only owned descendants;all old runners are terminal.
- Disposable operational state:/tmp/tsrust-250e1d24-regressions.json;
  runner:/tmp/tsrust-milestone-run.py;metadata:/tmp/tsrust-milestone-metadata.json.
  Source guard intentionally remains pinned to250e1d24 and three repaired test
  hashes. Do not blindly resume it after optimization changes. At the final
  milestone reconcile required unrun/affected checks against current source;
  preserve applicable evidence and run both doctest drivers separately afterward.
  Doctests are outside the494-target subtotal. No remote full-regression run was
  dispatched;call CI full only with -f full_regressions=true.

**Fixed floating-point literal returns — accepted locally, parent e09d0150:**
- Rejected nondecimal one-character storage trials before production edits.
  Early range-check reuse removes256/384/512 binary/hex/base36 allocations;
  regular timings1.073×/1.132×/1.192×,but wide hex/base36.973×/.976×.
  Post-format reuse has the same memory wins but wide binary/hex/base36 are
  .975×/.926×/.957×,repeat.980×/.947×/.953×. Original93samples,quiet maxloads
  2.492/1.957/1.709;40 full parent/trial/Node boundary outputs agree per round.
  Decimal controls are machine-code identical. Reject both variants;these are
  consumer-only diagnostics,not public compiler measurements or runtime changes.
- Reference ranking exposed a larger gap:array_temporary_index_number had
  recorded.554833ms versus hand Rust.022459ms and Node.165375ms. Fresh allocation
  baseline **2026-11-02T16:55:06.634Z**,4.170s/225,792KiB:Number makes32,768
  allocations/requested1,048,576 bytes,peak32,retained0;int control has zero heap
  calls,both cleanups0. Fresh two-case formal parent **2026-11-02T16:56:45.525Z**
  passes4.686s/236,768KiB,native Number.513458ms/Node.150000ms.
- Safe get extraction leaves every allocation unchanged;do not confuse it with
  fixing layout. A consumer-only four-element return removes all heap calls/
  bytes,checksum376,843,207 unchanged,three identical samples,3.102s/201,696KiB.
  Five complete IEEE observations agree across parent/get/fixed/Node. Quiet
  original93sample matched-path timing,maxload2.188:parent.509000ms,get.470625
  (1.082×),fixed.024250 **20.990×**,Node.150625;1.023s/17,120KiB. Generated
  producer used Vec;the independent Rust reference already uses a fixed array.
- Extend existing fixed_storage/rows facts to equal-width nonempty number[]
  literal returns,using existing byte/variant/proof budgets,eligibility and
  whole-use validation. Scalar indexing approves the closed result use;empty
  binding fact maps with outputs still run validation. Planned returns reuse
  fixed_literal emission. Unproved/growing/captured/variable-width uses retain
  Vec storage. No source limit,public type/ownership change or generated unsafe.
- Review caught a real proof defect before acceptance:statement-only scans missed
  enclosing returns inside match expressions and could declare the wrong Rust
  array width. Shared row statement traversal now includes match arms for seeding,
  propagation,return merging and use approval. Equal-width match returns optimize;
  varying/mixed forwarded returns fall back. O0/O3 regressions guard both cases.
- Final repair build203.702s/1,480,864KiB;**76 affected checks pass across six
  sequential binaries**,--test-threads=3,max652,960KiB:fixed storage34/temporary
  index5/temporary length3/receiver moves5/exceptions22/match values7. Witnesses
  cover zero heap calls,all f64 bits including NaN payloads/signed zero,coercions,
  effect order,index writes,clones,forwarding,alternatives,owned/callback/capture
  boundaries,bounds cleanup and budget fallback. Completed earlier checks reused
  until the match repair justified rerunning the complete affected section.
- All933 result ABIs and explicit layout contracts pass;932 emitted programs
  unchanged from parent,onlyarray_temporary_index_number changes37.189s/
  326,208KiB. Match repair preserves all933 benchmark emissions37.775s/
  329,200KiB. Final all-target diagnostics55.010s/1,345,216KiB pass with existing
  decode_plain warning only. Release215.835s/1,451,264KiB;all933 release/debug
  emissions agree37.729s/331,456KiB. Five actual runtime harness checks pass;
  corrected the registration filename after Node silently skipped a missing file.
- Actual profile **2026-11-02T17:49:00.784Z**,3.642s/281,872KiB:three identical
  samples per workload. Number32,768→0 allocations,requested1,048,576→0 bytes,
  peak32→0,retained0 and both cleanup counters0. Integer profile unchanged at
  zero;its complete Mach-O __TEXT,__text section is bit-identical to parent.
- Quiet same-path/argv0 A/B1.045s/17,872KiB,maxload2.493:Number
  .521458→.024250ms **21.503×**,Node.151542. Repeat1.042s/18,672KiB:
  .546500→.024250ms **22.536×**,Node.150375. Integer control.028375→.028416
  (.999×),repeat.028375→.029042 (.977×),Node.092875/.092792;identical source/
  machine code means no attributable regression. Ten complete observations agree
  per round;original3processes/eight warmups/31samples (93 per engine).
- Formal accepted runs **2026-11-02T17:49:03.381Z** and
  **2026-11-02T17:49:07.950Z**,2.596s/260,272KiB and2.607s/273,856KiB,retain
  original protocol with no busy override,maxloads2.275/2.493. Number native/
  Node.024292/.161334ms and.024833/.160750ms. Saved hand Rust.022459ms is reused,
  not a fresh A/B. store:ingest adds12 exact histories (4 parent,8 accepted).
  Compiler/workload/counterpart/emission/driver hashes,public pins and SQLite
  integrity verified. Only the final two runs are newly public.
- Public export933 comparisons/54 curated runs;66-page build3.654s/941,424KiB,
  existing duplicate404 warning only. Three website checks pass;all links/assets/
  anchors in116pages pass1.560s/21,200KiB. Pruned139 closed files
  (471,221,913bytes),including obsolete tests/codegen,verified imported older raw
  runs and disposable trial consumers/scripts. Current tools,six fresh test
  binaries,native consumers,the two accepted runs and current profile remain.
  Final whole-project formatter changes zero files/snippets/fingerprints;all
  owned runners are terminal. Local measured checkpoint,no CI,push or deployment.
- Next:owner correction above defers the broad regression milestone until the
  remaining large generated/hand-written Rust gaps are addressed and only small
  refinements remain. Preserve the interrupted current-source regression snapshot;
  continue scoped_record_composition and recursive-array profiling now. Use only
  affected suites and matched A/B checks during this optimization phase.
  Do not retry rejected nondecimal digit branches without a different cause.
  Static follow-up:small TsrMap already scans slots without hashing/buckets;
  scoped_record_composition needs fresh allocation/CPU attribution rather than
  assuming the small-map reference alone explains its gap.
- Remaining large-gap review queue (latest stored matched workload IDs,not fresh
  A/B measurements except the accepted Map update above):composition now
  21.408417ms vs saved hand Rust18.420917ms (1.162×,2.987500ms excess),replacing
  the historical121.497209ms row;recursive copy gaps are now small after the
  accepted dead-loop transport above:toReversed.030458/.027500ms (1.108×),
  with.039667/.034833ms (1.139×). Declared Number/boolean reductions are now
  .001791/.001584ms (1.131×) and.001542/.002167ms (.712×),after the accepted
  read-only slice proof above. Next remaining large signals:fresh nullable boolean4.092×;
  array_literal_spread_nullish_packs1.978×/call_spread_nullish_packs1.974× after
  accepted constant-string concat above;
  normal forEach gaps now0.962–1.278× saved Rust after accepted range views;
  non-void discard string/record/callable traversal is now1.045–1.055×,
  with nested results now0.203× after fixed private returns;plain byte views
  are now1.054× after matching recoverable allocation policy,while byte-view Maps are1.297×;
  expectation predicates are now3.176× after dispatch inlining, with reusable-matcher
  versus direct-assertion work documented above.
  Review absolute cost and equivalent work,
  inputs,ownership and observation before accepting a ratio as removable overhead.
  Existing Rust/Node references are reused unless source/protocol changes;rerun
  focused Node controls for actual candidate A/B rounds. Address or explain these
  substantial gaps before the owner-defined final broad-regression milestone.

**Decimal int.toString helper reuse — accepted locally, parent 28fcf7f1:**
- Clone-output inspection found independently owned escaping text,not a removable
  copy. A distinct measured lead:decimal int.toString bypassed existing
  tsr_int_text digit storage. No global integer writer change is retained.
- Fresh four-radix allocation baseline **2026-11-02T16:12:22.686Z** passes
  5.185s/251,008KiB;formal parent **2026-11-02T16:14:31.961Z** passes7.237s/
  273,504KiB with original93samples and focused Node controls.
- Consumer-only reuse preserves20 complete boundary outputs. Three identical
  samples:decimal2,049→1,665 allocations,8 reallocations unchanged,
  requested137,920→130,624,retained/peak88,960→81,664,both cleanup counters0.
  Other radices' complete dictionaries and machine __TEXT,__text match parent.
  Profile preparation4.149s/206,480KiB. Quiet same-path/argv0 trial3.114s/
  92,032KiB,maxload1.902:decimal.029458→.026292ms **1.120×**,Node.035291;
  identical-code binary/hex/base36 controls.997×/1.009×/1.000×.
- Matched nondigit-only pool checks that small-digit wins do not hide larger-int
  cost. Three-sample allocation dictionaries match exactly (2,049 allocations,
  8reallocs,requested137,920,retained/peak88,960,cleanup0);checksum263,903,081.
  Five complete native/Node outputs agree. Initial93sample comparison,maxload
  1.621:parent.030167→trial.029375ms (1.027×),Node.034625. After final build,
  a.029583→.031000ms **.954×** observation at load3.358 is retained as a warning.
  Three quiet repeats are.030417→.029541 **1.030×**,Node.034917;
  .029209→.029375 **.994×**,Node.035917;and.029583→.029125 **1.016×**,
  Node.035291,maxloads1.979/2.060/2.060. The slowdown is not repeatable;claim no
  wide-pool speed gain. This is a diagnostic,not a new public benchmark row.
- Runtime change is one line:return tsr_int_text(value) only for radix10,after
  the existing2..36 assertion. The existing static ASCII representation now also
  serves decimal digits here;remaining values retain standard Rust formatting.
  Tests preserve exact full-i64/all-radix text,errors and cleanup,with zero-or-one
  allocation budgets plus independent mutation and source-level owned transfers.
  Private Cow Owned-variant assertions now exempt reused decimal digits;the
  source ownership contract is unchanged. No ownership-checker,public API,limit,
  dependency,RC or generated-unsafe change.
- Build198.842s/1,391,856KiB;all19 affected checks pass across four sequential
  binaries,--test-threads=3,max557,840KiB. All933 ABIs/emissions agree after
  normalizing only the helper body;exactly four actual callers,36.686s/321,344KiB.
  Compiler/LSP all-target diagnostics pass56.064s/1,243,344KiB,existing decode_plain
  warning only. Release215.753s/1,485,632KiB;all933 final release/debug emissions
  agree38.271s/323,968KiB. Six affected runtime harness checks pass.
- Actual profile **2026-11-02T16:33:12.621Z**,4.693s/305,456KiB:decimal removes
  **384 allocations (-18.7%)**,requested bytes -5.3%,retained/peak bytes **-8.2%**;
  exact three-sample dictionaries match the trial. All other radices' dictionaries
  equal parent. Independently owned output behavior and both zero-live-byte
  cleanup checks remain intact.
- Quiet same-path/argv0 actual A/B2.578s/57,568KiB,maxload3.358:
  decimal.031208→.027375ms **1.140×**,Node.036333. Repeat2.580s/26,800KiB:
  .031125→.027375ms **1.137×**,Node.036375. Binary/hex/base36 code sections are
  bit-identical to parent;observed1.006×/.969×/1.005× and1.004×/1.011×/1.001×
  are control variation,not source gains or regressions. All20 complete outputs
  agree in each round;original3processes/eight warmups/31samples (93 per engine).
- Formal accepted four-case native/Node runs **2026-11-02T16:33:17.846Z** and
  **2026-11-02T16:33:29.328Z** retain the original protocol and no busy override,
  maxloads3.303/3.169. Decimal native/Node.027459/.036500ms and.027458/.037625ms.
  Watchdogs5.171s/281,328KiB and4.650s/314,496KiB. Existing Rust references are
  reused. store:ingest adds24 exact histories (8 parent,16 accepted);compiler,
  workload,counterpart,emission/driver hashes,public pins and SQLite integrity
  verified. Only the final two runs are newly public.
- Public export has933 comparisons/52 curated runs;66-page build3.682s/
  947,040KiB,existing duplicate404 warning only. Three website checks pass;
  links/assets/anchors in116pages pass1.555s/21,920KiB. Pruned99 closed files
  (522,053,069bytes),including obsolete test/codegen outputs,verified imported
  older raw runs and disposable trial consumers/scripts. Current tools,four
  fresh test binaries,native consumers,the two accepted runs and current profile
  remain. Final whole-project formatter changes zero files/snippets/fingerprints;
  all owned runners are terminal. Local measured checkpoint,no CI,push/deployment.
- Next: profile remaining nondecimal one-character output allocations and their
  nondigit controls before another change. Preserve independent owned output
  semantics and the original numeric append writer;do not assume digit storage
  helps every formatting path or replace required escaping string copies.

**Readonly Unicode character materialization — accepted locally, parent 856cd34e:**
- Parent compiler source is unchanged from77868f7a. Profiling isolates8,192
  temporary non-ASCII character strings. A consumer-only borrowed source-slice
  trial removes them with unchanged full outputs and retained/peak heap;initial
  timing1.114×Unicode/1.004×ASCII. Actual compiler slices then measure1.099×/
  .972×,repeat1.132×/.963×. Reject source slicing because ASCII slows repeatedly.
  Rejected formal run **2026-11-02T15:43:30.175Z** is ingested,not publicly pinned.
- Profiled plain stack encoding and an ASCII-branch variant next:both preserve
  the same allocation counts/bytes and five complete parent/Node boundary outputs.
  Quiet original93samples,matched physical executable path/argv0,maxload2.003:
  parent1.188583ms,source slice1.075625,plain stack1.014375 **1.172×**,
  ASCII-branch stack1.049625 (1.132×),Node1.208000. ASCII parent.736625,
  plain stack.710917 **1.036×**,branch.723792 (1.018×),slice.741625.
  Choose plain stack without another helper or ASCII policy;profile3.112s/
  150,800KiB and paired comparison7.242s/243,536KiB.
- emit/string_character.rs indexes per-function string-loop bindings whose every
  use is a direct comparison,length or accepted destination append. Copies,
  calls,captures,unknown uses and exhausted10,000-node proofs keep the owning
  path;source is never rejected for the proof budget. Eligible loops retain
  chars() and encode each scalar into a four-byte stack buffer. Only proved
  character reads use Cow<'_,str>;global String ownership,helpers and all public
  APIs stay unchanged. No generated unsafe,dependency,cache,arena or RC change.
- Added O0/O3 native witnesses for zero heap calls on readonly Unicode/ASCII/
  NUL/max-scalar reads,byte lengths,ordering,nesting,owned escaping outputs,
  input disposal,checked control flow/finally,copies,calls,captures and proof
  fallback. Final build206.473s/1,502,640KiB;**76 checks pass across seven
  sequential compiler/LSP binaries**,--test-threads=3,max660,720KiB. The initial
  fallback assertion matched unrelated runtime char_indices;the repaired assertion
  identifies the emitted loop. Nine focused runtime harness checks pass.
- All933 result ABIs pass;932 emitted programs stay byte-identical to the accepted
  parent through the slice/stack comparisons. Only string_iteration changes,
  with three proved loops. Final stack comparison37.707s/323,056KiB,current
  all-target diagnostics53.854s/1,590,640KiB,only existing decode_plain warning.
  Release217.993s/1,753,536KiB;all933 release/debug emissions agree37.981s/
  327,648KiB. Whole-project formatter changes zero files/snippets/fingerprints.
- Actual profile **2026-11-02T16:00:13.815Z**,3.606s/277,664KiB:three identical
  samples,checksum37,313,883. Allocations18,433→10,241 **(-44.4%)**,
  reallocations28,681 unchanged;requested1,629,824→1,605,248 bytes (-1.5%).
  Retained530,432/peak530,571 unchanged,both cleanup counters0. Four-byte
  automatic storage replaces transient heap strings;escaping outputs remain owned.
- Quiet same-path/argv0 A/B5.137s/250,624KiB,maxload2.368:Unicode parent
  1.275667→1.089583ms **1.171×**,Node1.267083. ASCII.766417→.763209
  (1.004×),Node.754625. Repeat5.658s/250,816KiB,maxload2.338:Unicode
  1.266500→1.104375 **1.147×**,Node1.286375;ASCII.779958→.765417
  (1.019×),Node.757083. No observed ASCII regression with the stack revision.
- Original-protocol formal native/Node runs **2026-11-02T16:00:18.564Z** and
  **2026-11-02T16:00:34.026Z** retain3processes/eight warmups/31samples
  (93 per engine),no busy override. Native/Node1.087708/1.268000ms and
  1.093917/1.273875ms;maxloads2.368/2.526. Watchdogs4.637s/427,600KiB
  and4.632s/332,544KiB. Existing Rust references are reused.
- store:ingest adds6 histories (2 rejected source-slice,4 accepted stack).
  Exact median/mean-rate/93sample histories,compiler/workload hashes,public pins
  and SQLite integrity verified. Only the final two runs are newly public.
  Public export has933 comparisons/50 curated runs;66-page build passes3.691s/
  900,816KiB with the existing duplicate404 warning only. Three website checks
  pass;links/assets/anchors in116pages pass1.551s/21,712KiB. Pruned127 closed
  files (486,448,635bytes),including completed codegen objects,verified imported
  older raw runs and disposable trial consumers/scripts. Current tools,seven
  test binaries,native consumers,the two accepted runs and current allocation
  profile remain. Final whole-project formatter changes zero files/snippets/
  fingerprints;all owned runners are terminal. This is a local measured
  checkpoint;no CI,push or deployment.
- Next: profile remaining owned output allocations and clone-heavy string-loop
  paths with this compiler before extending the proof. Explicit copies must
  remain independently owned;never let a stack borrow escape. Preserve the
  rejected ASCII slicing result and original numeric append paths.

**Text-only string self-assignment reuse — accepted locally, parent 456783ad:**
- Followed the recorded string_iteration allocation lead before implementation.
  Consumer-only replacement of its three nonescaping self-prefix assignments
  with the existing staged append helpers preserves five complete native/Node
  boundary outputs. Three identical samples:allocations133,121→18,433,
  reallocations9→24,585,requested3,473,024→1,455,744 bytes,but retained
  530,432→684,032 and peak530,466→684,096. Both cleanup counters0.
  Build/profile3.124s/137,408KiB. Do not hide the29% retained-memory increase.
- Exact-growth trial preserves requested/retained/peak bytes but makes114,697
  reallocations;build/profile2.579s/145,200KiB. Quiet original-protocol matched
  physical path/argv0 timing:parent2.725541ms,amortized1.122708 **2.428×**,
  exact2.906875 **.938×**,Node1.210375,maxload2.213;4.655s/247,504KiB.
  Reject per-append exact growth.
- Amortized construction plus compacting the two escaping output strings keeps
  retained530,432 bytes;peak530,496 (+30 bytes from the nonescaping trace).
  Allocations18,433,reallocations28,681,requested1,629,824 (-53.1%),cleanup0;
  three identical samples,2.594s/147,200KiB. Repeat with all five consumers:
  parent2.735291ms,amortized1.134667,exact2.820083,compact1.174542
  (**2.329×**),Node1.212000,maxload2.986;5.192s/247,168KiB. All five complete
  output cases agree. These are diagnostic consumers,not accepted compiler runs.
- Accepted:emit/self_append.rs indexes owned local self-prefix assignments and
  their binding transfers once per checked body,using the existing capture
  analysis and pointer-index pattern. Self references in suffixes,parameters,
  capture/view aliases and unknown bodies retain copies. A10,000-node proof
  budget falls back without limiting source. Existing string-part staging and
  storage selection preserve RHS effects before mutation,including cleanup-owned
  slots. Ordinary/void-expression assignments share the lowering;owned transfers
  compact only flagged local strings with the new safe tsr_string_finish helper.
  Bindings with any fused numeric self-append retain the entire original path,
  including mixed text/numeric builders:short builders regress with extra numeric
  reservations and compaction. This is a type/operation cost guard,not benchmark
  IDs or a new runtime size cutoff. Existing += lowering and integer writer stay
  unchanged. No new public API,
  ownership policy,dependency or generated unsafe.
- Added O0/O3 witnesses for amortized allocation count,tight owned outputs,
  Unicode/input disposal,snapshots,numeric/slice operands,comma assignments,
  parameters/fields,self references,closure capture rules,checked failures and
  finally cleanup. An11,000-assignment witness preserves proof-budget fallback.
  Initial build194.758s/1,452,928KiB;13 string-addition checks passed,one fixture
  correctly rejected replacement while a closure retained access. Keep that
  rejection assertion and use a consumed Once closure for the valid fallback
  witness. Its diagnostic assertion initially required Analysis:Debug;changed it
  to err().expect after build177.119s/1,540,992KiB. Storage selection now handles
  initialized cleanup slots rather than assuming every owner is a plain local.
  Storage-aware rebuild195.142s/1,631,808KiB passes without new warnings. The
  consumed Once capture correctly transfers its owner;the valid witness now
  updates/returns that owner inside the closure. Parameter rebinding is also
  rejected as before;test a mutable local copy and assert the original rejection.
  Checked the repaired complete fixture with the current CLI before its3.637s/
  338,672KiB test rebuild;its focused native witness passes2.073s/232,512KiB.
- **138 checks pass across11 sequential binaries**,--test-threads=3:completed
  string_addition14 (13 preserved plus repaired witness),comma4/assignment4/
  interfaces7/string_iteration3;closures14/once11/owned_arguments14/
  exceptions22/return_moves34/named_owned11. First group's watchdogs7.252s
  (13 pass/fixture failure),then1.563/2.071/1.578/1.043s;additional suite build
  7.269s/353,008KiB and runs4.161/1.551/2.596/5.215/13.559/2.600s,
  max642,336KiB. Cleanup,mutable/owned capture,partial-return and source error
  regressions all pass;do not restart completed checks.
- Candidate debug/accepted release933-workload emission/ABI comparison and
  parent preparation pass40.858s/330,256KiB. Removing only the new runtime helper
  proves930 user programs unchanged. Actual changes:set_values,map_iteration,
  string_iteration. Rebuilt these three parents with matched canonical source
  paths,flags,drivers/ABI,15 complete boundary observations and deterministic
  three-sample allocation baselines before replacing release tools.
- Generic candidate release212.511s/1,370,816KiB and official3-case native/Node
  run **2026-11-02T13:41:44.348Z** pass correctness;full emit/profile/timing block
  23.715s/431,232KiB. Actual A/B6.744s/245,808KiB,maxload2.248:string iteration
  2.293×,but set.901× and map.678×. Reject the generic numeric rewrite.
  Consumer actual-growth trial4.677s/180,080KiB plus paired6.723s/264,176KiB,
  maxload2.320 restores set1.004× but map remains.774×;reject that helper change
  too. Preserve the original reserve helper and numeric paths instead.
- Numeric/mixed exemption build196.988s/1,345,936KiB;string_addition15 and
  string_iteration3 pass7.834/1.583s,max533,936KiB. New O0/O3 witness preserves
  the original four allocations and33-byte capacity of a short mixed numeric
  builder. Pure-text comma witness ensures expression lowering remains exercised
  after the exemption;focused rebuild/run3.622s/258,480KiB and2.080s/179,648KiB.
  Total139 affected checks remain accepted;completed ownership suites are reused.
- Final933 debug/parent ABIs pass26.905s/315,888KiB:removing only the unused
  finish helper proves932 bodies unchanged,including restored set/map. String
  iteration code exactly equals the previously profiled candidate. Compiler/LSP
  all-target check53.807s/1,378,864KiB and final release212.610s/1,613,472KiB
  pass;only existing decode_plain dead-code warning. Final release933 hashes/ABIs
  match tested debug10.866s/315,120KiB. Final test-source all-target recheck
  2.595s/264,640KiB and8 focused harness checks1.038s/162,176KiB pass.
- Actual final allocation profile **2026-11-02T14:29:58.445Z**,3.625s/294,704KiB:
  three identical samples each. String iteration133,121→18,433 allocations
  **(-86.2%)**,9→28,681 reallocations,total allocation/reallocation calls
  133,130→47,114;requested3,473,024→1,629,824 **(-53.1%)**. Retained530,432
  bytes unchanged;peak530,466→530,571 **(+105 bytes,0.020%)** from temporary
  capacity while output cells are staged. Cleanup0. Set/map dictionaries exactly
  equal parent;their complete Mach-O __TEXT,__text sections are bit-identical.
- Official accepted3-case run **2026-11-02T14:30:07.070Z**,8.808s/425,168KiB,
  maxload2.164,no busy override,original3processes/8warmups/31samples per engine.
  String native1.238958ms/Node1.242292ms. Matched same-path/argv0 parent A/B
  6.725s/259,680KiB,maxload2.689:parent2.729292→candidate1.183292ms
  **2.307×**,Node1.230292;identical-code controls1.007×/1.006×,no attributable
  regression. Focused repeat paired plus official block9.839s/336,080KiB:
  parent2.725333→candidate1.191375ms **2.288×**,Node1.238167,maxload2.120.
  Official repeat **2026-11-02T14:38:24.653Z**,native1.167208/Node1.224458ms,
  maxload2.189. Both original formal runs retained;do not select individual samples.
- store:ingest adds14 minimal histories (6 rejected generic,6 final,2 repeat).
  Verified every rounded median/mean rate/93sample entry and SQLite integrity.
  Public pins select only the final two runs;933 saved Rust timings are reused.
  Both accepted formal runs and final allocation record match current compiler
  source hashes exactly. Performance export933 comparisons/48 curated runs,
  three website checks,66-page build3.688s/915,504KiB and links/assets/anchors
  in116pages pass1.550s/21,808KiB;existing duplicate404 warning only.
  Whole-project formatter4.132s/202,112KiB changes zero files/snippets/
  fingerprints. Pruned1,187 closed files (8,823,347,016bytes),including527
  obsolete project test executables/dependency files identified from Cargo source
  paths and checked against live processes. Current tools,two fresh test binaries,
  accepted native consumers/profiles,two accepted raw runs and imported histories
  remain;all owned runners are terminal. Local measured checkpoint,no CI/push/
  deployment.
- Remaining:do not call numeric/mixed self-append optimized. They preserve the
  fast existing fused formatter;future work must compare short and long numeric
  builders and preserve short-case performance before changing growth policy.
  Reprofile remaining Unicode character materialization and escaping allocations
  with the accepted compiler before choosing another change. No arena/RC/cache
  policy is implied by this result.

**Single-digit integer appends — profiled and rejected, parent 71a947af:**
- Fresh generated/reference interfaces CPU sampling,without allocation counters
  or result observers in the repeated workload/drop loop:both checksum473727250,
  watchdog31.023s/133,920KiB. Generated2,128 samples include288 direct
  tsr_push_int samples (13.5%) and217 from_utf8 samples (10.2%);allocation/free
  and copying remain visible. Reference format! formatting/allocation/drop work
  is separate;this diagnostic is not a new hand-written Rust timing baseline.
- Consumer-only trials compare the existing writer,a0..9 push(char) fast path,
  and std write! formatting. All five complete native/Node boundary cases agree.
  Three allocation samples per consumer are exactly equal:6,142 allocations,
  reallocations9,requested775,655,retained/peak530,375,both cleanup counters0.
  Trial builds/checks3.626s/146,416KiB.
- Quiet matched argv0 parent/trials/Node run,original3processes/8warmups/
  31samples:parent.133583ms,single digit.120167ms **1.112×**,std writer.149791ms
  **.892×**,Node.117125ms,maxload2.473;watchdog1.560s/38,096KiB. Reject the
  standard formatter replacement. The initial runtime candidate added only a
  single-digit branch;all other integers retained the existing writer.
- **36 compiler checks pass**,six sequential binaries,--test-threads=3:
  string_addition11/enums6/interfaces7/numeric_string_conversion6/number_text2/
  int_to_string4. New O0/O3 witness checks all digits,zero-allocation UTF-8
  destination reuse,signed zero,integer extremes and1,024 random values against
  standard Rust spelling. Build196.746s/1,542,864KiB;watchdogs5.209/1.572/
  1.562/2.095/1.053/2.081s,max531,600KiB. Generated code remains safe Rust.
- Parent933 emission/ABI checks pass10.340s/314,048KiB;18 actual callers.
  Rebuilt all18 parent timing/allocation consumers with original ABI/drivers,
  source paths and flags;90 complete boundary observations and three identical
  allocation samples each pass18.634s/321,920KiB. The first preparation attempt
  omitted the manifest's default source-path convention;corrected that diagnostic
  script before any consumer compiled. No source snapshots are retained.
- Candidate release207.875s/1,495,040KiB. Candidate933 emission/ABI checks pass;
  571 support-code differences normalize exactly to the added four-line block.
  Eighteen actual callers remain. Official native/Node run
  **2026-11-02T12:05:26.222Z**,52.271s/499,936KiB including emission/preparation,
  passes all five correctness cases per engine with original93samples and
  maxload2.986,no busy override. Paired all18 run27.376s/262,400KiB,maxload2.639:
  interfaces1.130×,labeled control1.295×,enums1.201×;four cases.978–.991×.
  Focused8-case repeat10.869s/182,368KiB,maxload2.596 preserves winners,but
  iterable call spread is.972×. Do not keep a global branch based on interfaces.
- Tested three consumer refinements before changing more source:a small wrapper
  around the unchanged digit kernel,an inline-never kernel,and digit-fast-path
  plus standard formatting for larger values. Wrapper18 build9.335s/202,720KiB;
  paired34.120s/259,968KiB,maxload2.657. Other two variants build16.570s/
  236,080KiB;paired41.940s/221,152KiB,maxload2.319. All complete boundary
  observations agree. None clears all regressions:string iteration remains
  **.968×/.968×/.968×**,and standard formatting additionally makes
  array_from_map_string.959×. Reject every variant,including the outlined
  interfaces1.181×/labeled1.369× wins;do not cherry-pick those measurements.
- Ruled out loader-path differences:parent and actual candidate run through the
  same temporary hard-link pathname with identical native argv0. Same original
  protocol/Node control,90 full boundary comparisons pass;28.467s/243,392KiB,
  maxload2.435. Interfaces.136167→.118084ms (**1.153×**),labeled1.319×,
  enums1.199×,but string iteration2.748875→2.810584ms (**.978×**) and iterable
  call spread1.589792→1.639125ms (**.970×**). The slower larger-value paths
  persist,so the generic optimization fails acceptance despite real local wins.
- Reverted runtime/strings.rs exactly to71a947af;no compiler/runtime change is
  retained. Keep the new O0/O3 spelling/destination-reuse witness,which is a
  contract of the existing writer too. store:ingest adds36 minimal histories
  from the formal rejected run;public performance pins remain unchanged.
  All36 histories pass exact rounded median/mean rate/93sample checks and SQLite
  integrity. No dependency,public API,ownership-policy,CI,push or deployment
  change. Do not retry the global digit branch or these formatter/wrapper variants
  without a new measured cause;larger-integer paths must remain fast.
- Restored compiler/LSP release tools and the retained test build pass
  404.173s/1,647,152KiB combined. All11 string-addition checks pass against the
  original writer5.168s/443,184KiB,including the new O0/O3 witness. All933 restored
  emissions are byte-identical to parent10.853s/315,968KiB. Native/Node correctness
  for all18 cases passes19.197s/436,464KiB. Restored allocation profile
  **2026-11-02T12:29:48.247Z**,10.868s/363,008KiB:all18 three-sample dictionaries
  and emitted-source hashes exactly equal parent,including cleanup0. Current
  compiler source hashes equal accepted71a947af and this restored profile.
  Public performance export is byte-identical (933 comparisons/46 accepted runs),
  two readability/data checks pass,and whole-project formatter4.128s/203,088KiB
  changes zero files/snippets/fingerprints. Pruned222 closed probe/raw/obsolete
  test/intermediate files (580,034,320bytes) after import verification. Current
  accepted consumers/profiles,release/debug tools and the fresh11-check binary
  remain;all owned runners are terminal. Local profiling/test checkpoint only.
- **Next distinct hypothesis:**string_iteration makes133,121 allocations at
  n2048 (65 per scenario plus the outer allocation),with9 outer reallocations.
  Inspect/profile the nonescaping trace/filtered/nested updates written as
  `text = text + ...`. Compare their generated code with the existing uncaptured
  local `+=` lowering and direct destination writes before another integer writer
  trial. Stage all RHS work before mutation;preserve errors,side effects,borrows,
  captures,assignment-result ownership and independently escaping strings.
  Establish a consumer allocation/timing diagnosis first;do not assume a syntax
  rewrite is safe or faster. This uses the owner's destination-reuse profiling
  suggestion where intermediate strings do not escape.

**Readonly small parameter slices — accepted locally, parent 23f578f3:**
- Reprofiled nested_rest_patterns.describe, not interfaces.describe: the former
  copies its destructured parameter tail through the heap-default slice helper.
  A consumer-only substitution to the existing inline helper removes one
  temporary container per iteration. Three identical allocation samples:
  39,710→37,662 allocations,requested2,032,131→1,982,979 bytes;reallocations2,057
  and retained837,743 bytes unchanged. Peak838,075→838,051;both cleanup counters0.
  Checksum634209508 and five complete native/Node boundary outputs agree.
  Diagnostic build/profile3.628s/172,480KiB.
- Quiet diagnostic parent/trial/Node timing uses normalized native argv0 and
  original3processes/8warmups/31samples:parent.862791ms,trial.840042ms
  (**1.027×**),Node.351208ms,maxload3.009;watchdog2.659s/115,696KiB.
  This is a consumer experiment,not an accepted compiler measurement.
- Accepted fixed_storage.rs intersects all direct named-call inputs for readonly
  array parameters. Only small literals or existing proved-small locals qualify;
  mixed,unknown,mutable,omittable and callable/closure paths retain default
  lowering. One call scan,no iterative whole-graph inference;forwarded parameters
  remain unknown. The runtime still safely handles a heap receiver. No physical
  ABI,public API,ownership policy,dependency or generated unsafe change.
- **74 affected compiler checks pass**,five sequential binaries with
  --test-threads=3:array_slice6/destructuring_rest24/fixed_storage29/
  nested_rest3/primitive_storage12. New O0/O3 witness covers known local/literal
  callers,mixed widths,mutation,callable fallback,Unicode,independent results and
  an injected heap receiver. Initial new test used an unsupported From<Vec>
  constructor;corrected the test to the existing From<[T;5]> heap path.
  Build192.910s/1,263,328KiB;affected test build4.655s/417,104KiB;binary
  watchdogs3.626/4.161/18.172/1.042/2.592s,max546,544KiB.
- Parent933 emission/ABI checks pass10.353s/311,296KiB;candidate933 pass
  11.363s/261,488KiB. Only nested_rest_patterns changes:three→four inline calls,
  and normalization proves replacing that helper name is the sole difference.
  Other932 emitted programs are byte-identical. First release build passes
  211.136s/1,384,720KiB.
- The first official runner attempt timed out during its built-in Cargo build
  before preparation/timing (122.673s/1,321,408KiB). A comment edit during the
  preceding release build required Cargo to refresh it. Its compiler child was
  initially still alive,but the process group had exited before the cleanup
  signal;no owned child was restarted on an observation timeout. Fixed the real
  harness mismatch:run.mjs now gives only the release build the30-minute deadline;
  ordinary child commands retain120seconds. The retry passes under the outer
  watchdog216.155s/1,581,328KiB,including final release rebuild and official timing.
- Official accepted native/Node run **2026-11-02T11:42:23.865Z** preserves all five
  independent correctness cases and original3processes/8warmups/31samples:
  native.877917ms,Node.379542ms,maxload3.198,no busy override. Paired actual
  compiler-output comparison normalizes native argv0 and verifies full outputs:
  parent.881958→candidate.827625ms **1.066×**,Node.360541ms,maxload3.019,
  watchdog3.106s/170,560KiB. Repeat parent.864667→candidate.826917ms
  **1.046×**,Node.355291ms,maxload2.752,watchdog2.589s/118,624KiB.
  Only one emitted workload changed;do not infer a speedup for other workloads.
- Fresh actual-compiler allocation profile **2026-11-02T11:43:26.538Z** confirms
  all three samples exactly match the consumer trial above;3.631s/266,960KiB.
  Ten affected harness checks pass10.855s/688,176KiB. Compiler/LSP all-target
  check51.745s/1,515,312KiB passes with only the existing decode_plain dead-code
  warning. Whole-project formatter4.134s/203,648KiB changes zero files/snippets/
  fingerprints;no editor or public signature changes.
- store:ingest adds2 minimal histories;verified both93sample entries,rounded
  medians/mean rates and SQLite integrity. Public performance pins select this
  accepted run;933 saved Rust references are reused. Two performance-data checks
  and the66-page website build pass3.640s/945,168KiB;existing duplicate404 warning.
  Link-check regression passes. Links/assets/anchors in116 built pages pass
  1.561s/20,992KiB. Final whole-project formatter4.133s/201,920KiB changes zero
  files/snippets/fingerprints. Pruned123 closed diagnostic/intermediate/old raw
  files (487,944,774bytes),after verifying all132 old histories remain imported.
  Current accepted consumers,formal run and allocation profile remain;all owned
  runners are terminal. This is a local source checkpoint,no CI/push/deployment.
  Next:profile residual escaping numeric strings and string construction with the
  current compiler/reference before another change. Follow the object allocation
  plan below;do not revisit rejected sizing/global-capacity trials or introduce
  arenas/RC without new evidence and owner design.

**Fixed result rows and readonly small slices — accepted locally, parent fa205130:**
- Parent compiler source is unchanged from c21ea2ce;66353533 was the profiling
  checkpoint and fa205130 repairs the harness. Profiled escaping five-string
  rows before implementation (see diagnosis below). The accepted implementation
  uses owned Vec<[T;N]> result rows and [T;N] row clones through the existing
  fixed-storage planner. New rows.rs tracks provenance across named calls,
  returns,locals,clones and loop aliases,with connected fallback. Mixed widths,
  resizing,unknown/callable/capture boundaries and checked-error transport retain
  ordinary storage. Existing proof budgets fall back without imposing source
  limits;integer/boolean rows<=4 retain existing zero-allocation storage.
- Added result facts/physical emission and explicit fixed-array-result contracts.
  Benchmark ABI parsing checks exact metadata keys,primitive type,rank,width,
  function-name type and complete declaration;observer supports Rust arrays with
  unchanged full boundary/string tokens. Invalid metadata cannot bypass ordinary
  ABI guards. All measured63 consumers pass the tightened guard;instrumented
  source hashes and compiler inputs match measured records exactly. No generated
  unsafe,RC,arena,dependency,public API or ownership-policy change.
- A generic inline-slice implementation exposed real heap-caller regressions:
  three workload bodies grew988→1,704 /1,585→1,697 /680→1,272 instructions.
  Rejected it,along with inconclusive direct-push/inline-hint trials. Restored the
  original heap-returning tsr_slice;only readonly,nonrebindable ordinary small
  literal locals and their move/clone/slice chains use tsr_inline_slice. Unknown,
  parameter,mutable and fixed-layout receivers keep existing lowering. Runtime
  matching still handles a heap receiver safely. Direct-slot construction reuses
  the existing From-array pattern. Actual benchmarks select five inline slice
  calls across three rest/destructuring workloads;all25 unchanged/control
  executables have bit-identical complete __TEXT,__text against the parent.
- **152 compiler checks pass**,12 sequential binaries,--test-threads=3:
  primitive12/slice5/fields12/literal-spread11/destructuring12/assignment2/
  defaults16/rest24/fixed29/nested14/nested-rest3/owned12. Final test build
  199.839s/1,592,464KiB;watchdogs3.679/3.096/3.637/4.180/2.599/1.041/
  3.120/4.159/18.695/4.661/1.040/3.639s,max556,368KiB. New O0/O3 witnesses cover
  forwarding,owned Unicode input,independent row clones,connected fallback,
  primitive/signed-zero payloads,readonly small chains,large/mutable/unknown
  receivers,zero container allocations,first spill and partial-clone unwind.
- Complete933-workload emission/ABI sweep passed (37.205s/327,232KiB);552
  support-code changes but only25 slice callers. Refreshed all63 affected/control
  identities after the refinement (1.553s/134,192KiB),no errors.37 row cases plus
  23 additional slice cases and3 numeric controls. Preserved old consumers before
  rebuilding;300 independent parent boundary/output checks passed. Official runs
  validate all five original correctness cases per engine,including full output.
- Harness checkpoint **fa205130** covers all586 checks through a sequential
  213-file pass plus focused repaired-file reruns. Repairs preserve operation,
  source-mutation and oracle guards while using live manifest source identities,
  shared compiler paths,actual counterpart paths,file/environment fixtures,
  CommonJS/JSON interop and current metadata anchors. Final two broad output/ABI
  checks pass with current release;additional8 adapter/ABI checks pass after
  rejecting non-string contract function names. No full matrix restart.
- Final release206.444s/1,539,568KiB. Compiler/LSP all-target check and whole-project
  formatter56.371s/1,516,944KiB pass;zero files/snippets/fingerprints changed.
  Existing decode_plain dead-code warning only. No editor/public signatures changed.
- Final allocation profile **2026-11-02T10:02:27.992Z**:63 workloads,three identical
  independent samples each,all output/input cleanup0. Interfaces **8,190→6,142
  allocations (-25.0%)**,requested955,943→775,655,retained/peak743,367→530,375
  **(-28.7%)**,reallocations9 unchanged. Versus row-only candidate,nested rest
  45,854→39,710 allocations and4,105→2,057 reallocations;destructuring rest
  42,182→40,134 and12,297→10,249. Other35/37 sample dictionaries unchanged;
  three numeric controls remain all zero. Final broad-check/profile block
  41.381s/1,085,280KiB. The more aggressive37,662 nested-rest count belongs to the
  rejected generic slice variant,not accepted source.
- Official accepted63-case native/Node run **2026-11-02T10:06:45.875Z**,
  95.140s/573,600KiB,maxload3.135,no override;focused three-case repeat
  **2026-11-02T10:20:52.247Z**,8.281s/337,440KiB,maxload2.154. Original
  3processes/8warmups/31samples (93 per engine). Hand-written Rust timing reused.
  A wrongly selected Node26 environment was rejected by the version guard before
  preparation/timing;all retained records use pinned Node24.11.1,no extra target.
- Causal A/B matches source paths,Rust flags,observer and binary identities,and
  normalizes native argv0 to the standard candidate path:drivers retain the
  executable-name allocation before timing. Verified the override on a native
  child. Final63-case paired run65.397s/683,024KiB,maxload3.198: **interfaces
  .160208→.135334ms (1.184×)**,nested rest **1.094625→.905791 (1.208×)**,
  loop destructuring .385542→.281125 (1.371×),numeric rows .022542→.004750
  (4.746×),native bytes1.411×. All complete outputs agree. Focused matched repeat
  5.141s/415,456KiB,maxload2.298:interfaces1.143×,nested rest1.208×,locale1.013×.
  Broad locale read.987×;short cases vary around parity alongside identical-code
  controls (up to4.4%). Do not claim every workload faster;no stable attributable
  regression remains. Keep original measurements rather than selecting samples.
- store:ingest added **338 minimal histories** from four formal runs;verified every
  rounded median6/mean-rate3/93count,quiet policy and SQLite integrity. Rejected
  intermediate runs remain history but are excluded from public pins. Performance
  page selects only the final two runs;933 saved Rust references remain unchanged.
  Three website checks,66-page build3.638s/887,728KiB and links/assets/anchors in
  116pages pass;existing duplicate404 warning only.
- Pruned609 closed/obsolete probe,parent,profile,raw and completed-build
  artifacts (1,654,168,979bytes) after import verification. Current accepted63
  consumers/profiles,latest12 test binaries,and two accepted raw runs remain.
  Final whole-project formatter4.134s/201,840KiB changes zero files/snippets/
  fingerprints;all owned runners are terminal.
  No CI,push or deployment. Next:profile residual escaping strings with this
  compiler and current reference before choosing another change. In particular,
  inspect describe's remaining parameter slice and standalone numeric strings;
  do not repeat rejected global array-capacity changes,integer-sizing passes,
  direct-push trials or adopt arenas/RC without new evidence and owner design.

**Escaping row storage — diagnosis complete; global capacity increase rejected:**
- Parent **c21ea2ce**, initially clean;compiler unchanged throughout this section.
  Current profiles show interfaces8,190 allocations,retained743,367 bytes. Each
  five-string result row uses Array<String>'s heap arm while its outer slot still
  carries four inline cells. This is a separate cost from the removed input copy.
- Compared two generated-consumer hypotheses before compiler work: Vec<[String;5]>
  with direct row literals,and a five-slot Array<T> runtime. Fixed-row conversion
  changes only result/local container types and the unique row constructor;adds
  a generic Rust-array SliceTree visitor with identical boundary/string tokens.
  Original instrumented source hashes asserted,original path/Rust flags/driver
  retained,sources restored after every compile. Neither is a compiler feature.
- Quiet paired3processes/8warmups/31samples,**2026-11-02T06:42:33.109Z**,
  maxload1.845,4.679s/289,024KiB: parent **.152583ms**,fixed-row **.136333**
  (1.119×),inline-five **.139459** (1.094×),Node .118750. Full original
  checksum473727250/exact output agree. Additional135 native/oracle checks
  (0.519s/896KiB) cover all three engines at0/1/4/5/2,048 rows,seeds0/7/17,
  empty/Unicode/quoted-newline text;these checks are not timing measurements.
- Three allocation samples per trial are deterministic,checksum-correct and
  cleanup0. Both remove2,048 allocations:8,190→6,142;reallocations9 and
  during-work frees0 unchanged. Fixed-row requested775,655/retained530,375/
  peak530,375;inline-five808,391/546,759/546,759. Parent requested955,943/
  retained743,367/peak743,367. Fixed-row saves212,992 retained bytes (28.7%);
  five-slot saves196,608. The simpler runtime change captures most of this gain,
  so checked its other consumers before committing to result-shape machinery.
- Seven existing string/record generated-consumer controls,maxload2.598,
  11.385s/268,672KiB: interfaces .153334→.139250,type aliases .124708→.115500,
  function expressions .321291→.317417,labeled .283209→.281541,concat
  .453167→.448084,ASCII chain .038917→.038542,Unicode chain .040500→.041791.
  Do not accept the runtime change from favorable interface timings alone.
- Fresh official current-parent14-workload run **2026-11-02T06:45:40.393Z**,
  21.222s/404,224KiB,maxload1.934,no override,original93samples/engine.
  Covers owned/nested/record arrays,churn,string slice/map/filter/concat/fill/
  with/join/splice/flat/of. Full independent outputs validated;record churn has
  no Array<T> representation and is unchanged by this proposed runtime patch.
- First broad paired diagnosis completed12 affected cases,then the unrun flat
  probe failed compilation:its owned iterator still used four-slot scratch
  storage. This is a diagnostic-patch omission,not an existing compiler defect.
  Fixed only that probe's owned iterator width;never changed compiler source.
  Initial medians already show record arrays .083875→.093833 (11.9% slower),
  nested .116041→.162042 (39.6%),owned .052292→.058167 (11.2%). These large
  regressions outweigh the interface win. Do not retry a global capacity5 change.
- Focused confirmation plus unrun flat,maxload3.591,49.214s/258,736KiB,
  waited for quiet load without override: record arrays **.091250→.103625**
  (13.6% slower),nested **.115958→.144834** (24.9%),owned
  **.052292→.056792** (8.6%),flat .767708→.782750 (2.0%). Node controls
  .324709/.356250/.314417/.482291 respectively. All complete outputs match.
  **Global five-slot Array rejected before implementation.** Existing four-slot
  runtime,record/array storage bounds,ownership and small-array wins preserved.
- Ingested28 official current-compiler histories;exact rounded medians/mean
  rates/93counts and SQLite integrity verified. Public selection uses only this
  official run,not diagnostic variants. Saved handwritten Rust timings reused.
  Whole-project formatter4.127s/202,944KiB,zero changes;three website tests pass,
  66-page build3.634s/987,312KiB and links/assets/anchors in116pages pass.
  Existing duplicate404 warning only. Product/compiler source unchanged;reuse
  preceding70 compiler checks and all-target agreement rather than rebuild it.
- All21 diagnostic input sources restored byte-for-byte. Pruned30 closed
  trial executables/scripts/results (12,675,920bytes);current compiler/profiles and
  official parent timing remain. All owned runners terminal;no compiler tests
  rerun,no CI,push/deployment for this profiling checkpoint.
- Next executable section: implement proved fixed-width result rows using the
  existing fixed-storage planner,not a wider global Array. FunctionPlan currently
  tracks parameter/local shapes only;add result facts and propagate them through
  named calls and local result bindings. Require one agreed width for every
  inserted/replaced row and every reachable return. Mixed widths,resizing inner
  rows,unknown calls/captures/exports or unsupported operations retain ordinary
  storage. Keep output elements owned;do not borrow escaping string data.
- Cover callers and native boundaries before enabling emission. Support normal
  row iteration/indexing/cloning and preserve mutation/ownership/error behavior;
  fall back coherently when result representation cannot cross a boundary.
  Update benchmark ordinaryWorkloadBinding and sliceRustObserver with a proved
  array-result contract plus [T;N] visitor;current ABI guard correctly rejects
  Vec<[String;5]>. Never relax guards or special-case the interfaces workload.
  Add independent fixture/oracle/boundary tests before official measurements;
  diagnostic upper-bound gains are not accepted product performance.

**Borrowed local record string copies — accepted locally, parent 14c2c616:**
- Reused preceding current-parent allocation/CPU profiles and quiet official
  **2026-11-02T06:05:27.292Z**;compiler source unchanged between2bcb344d and
  14c2c616. Interfaces copies an owned19-byte input into plain Player.name every
  iteration,then only borrows that local for describe. This is a measured
  transient allocation,not an escaping output or a general arena opportunity.
- Extended existing fixed-storage Record shapes with borrowed string-field facts.
  Only String(input) from a nonrebindable shared string parameter is eligible.
  Existing whole-record use/callee proofs reject moves,mutation,captures,escapes
  and unproved uses;all such cases retain owning layouts. Existing bounded call
  specialization carries the shape through named readers;no new planner/limit.
  Conservative Copy-union layout accounting admits optional integer fields.
- Private fixed records hold &'a String;field selection dereferences the physical
  loan before ordinary logical string operations. Explicit clones/spread and
  eager object snapshots materialize ordinary owning fields. Source ownership
  checking remains unchanged;no RC,arena,dependency,public API or generated unsafe.
  Shared parameter loans cannot outlive the call or retain backing storage in an
  escaping result. Current row representation and retained output bytes unchanged.
- Three formatted native witnesses cover forwarded/optional-field string reads,
  owned Unicode input,explicit clone/mutation independence,String(field) output,
  object keys/values/entries and spread snapshots in constructor order,plus
  owning fallbacks for returned/mutated/captured records. Counting-allocator
  witness proves0 allocations at O0/O3 for128 repeated record reads per input,
  including empty/ASCII/Unicode,and0 retained bytes after dropping each owner.
- Initial196.530s/1,527,408KiB build;21 fixed-storage checks passed and one test
  assertion failed by matching an unrelated runtime iterator's &'a String.
  Corrected to assert the actual FixedRecord declaration. Optional Copy-union
  build194.700s/1,562,880KiB was superseded before execution by the reviewed
  physical-field normalization fix. Final source build196.722s/1,592,080KiB.
  Test-only builds3.120/2.076/3.108s,max317,936KiB. One new allocation oracle
  initially counted Unicode scalars instead of documented UTF-8 bytes;corrected
  the test,not runtime semantics. No native semantic failure left unresolved.
- **70 affected checks pass**,one binary at a time,--test-threads=3:
  fixed_storage24,primitive_storage11,interfaces7,string_addition10,
  fixed_number_storage4,nested_records14. Watchdogs15.585/3.126/1.566/4.717/
  7.302/4.672s,max540,944KiB. Ownership,escaping snapshots,formatting and existing
  failure/unwind witnesses pass. Release compiler/LSP209.651s/1,530,384KiB.
- Candidate profile **2026-11-02T06:33:56.197Z**,5.706s/289,872KiB:
  interfaces **10,238→8,190 allocations (-2,048,20.0%)**,reallocations9 unchanged,
  requested994,855→955,943 (-38,912 =2,048×19),during-work frees2,048→0.
  Retained743,367 unchanged;peak743,386→743,367;all output/input drops0.
  Three samples identical/checksum473727250. All three numeric control allocation
  dictionaries remain byte-identical and zero;all nine timing controls' emitted
  AND instrumented Rust source remains byte-identical to parent.
- Official quiet candidate **2026-11-02T06:34:22.692Z**,
  13.986s/311,568KiB,maxload2.686 versus parent2.120,no override,original93
  samples/engine: interfaces **.201083→.159583ms,1.260×**,Node
  .116708→.120833 (normalized1.305×). All3 native process medians
  .160792/.157958/.159625 beat parent. Numeric record controls remain exact.
  Byte-identical labeled/concat controls drift .283916→.302333/.456542→.475750;
  neither contains changed code,so diagnose before attributing a regression.
- Focused official follow-up **2026-11-02T06:35:22.108Z**,
  8.799s/298,144KiB,maxload2.155: interfaces **.152417ms,1.319×** versus parent,
  Node .118666 (normalized1.341×);native process medians .151750/.154541/.152417.
  Labeled .285208,concat .458000 are within0.5% of parent,confirming phase drift.
  Preserve both official runs unchanged;public selection uses the latest run.
- Ingested26 candidate histories;verified46 parent/candidate entries for exact
  rounded median/mean rates/93counts and SQLite integrity. Manifest,toolchain,
  workload sources,oracle,adapters,harness,drivers and lock match;only interfaces
  generated code changes. Saved standard Rust reference timings reused.
- Compiler/LSP all-target check53.306s/1,341,968KiB passes;existing decode_plain
  dead-code warning only. Whole-project formatter4.126s/201,856KiB,zero source/
  snippet/fingerprint changes. Three website tests pass;66-page build
  3.650s/932,144KiB and links/assets/anchors in116pages pass. Existing duplicate
  404 warning only. Frontend/editor signatures unchanged;no full matrix rerun.
- Pruned20 closed/obsolete raw,probe and pre-change test artifacts
  (201,952,892bytes) after import verification. Current70-check binaries and latest
  candidate allocation/timing remain;rebuild other consumers from commits when
  needed. No CI,push/deployment;all owned runners terminal.
- Next: profile the remaining escaping five-string row allocation. A fixed-width
  Rust array could remove each inner Vec allocation,but requires proved result
  shapes and compatible callers/observers;do not globally reduce four-element
  inline storage or assume all returned rows have fixed width. Establish a
  generated-consumer upper-bound diagnosis with the original full-output oracle
  before deciding whether that larger specialization is justified. Preserve
  generic array mutation,ownership,ABI/export behavior and small-array wins.

**Object construction isolation and exact chain sizing — profiled/rejected, 2026-11-02:**
- Parent **2bcb344d**, initially clean. Reused existing string-free
  generic_record_nested_int and generic_function_record_pipeline construction/
  field-access workloads,plus record_mutation. No new workload,dependency or
  speculative numeric-record harness. These aggregate results can be optimized
  away; they establish this pipeline's behavior,not universal struct optimality.
- Fresh native profile **2026-11-02T06:03:14.322Z**,4.666s/256,160KiB:
  all three numeric controls have0 allocations/reallocations/requested/retained/
  peak bytes. Checksums10,011,200 for both construction controls and
  10,000,700,000 for mutation;three deterministic samples each,cleanup0.
  Generated code is nested typed structs/direct fields,not a dynamic object map.
- Matched standard Rust allocation profiles **2026-11-02T06:04:50.734Z**,
  1.561s/256,960KiB: identical zero-allocation numeric controls;reference hashes
  exactly match public pins. Interfaces generated10,238 allocations/9 reallocs,
  requested994,855,retained743,367,peak743,386;reference16,385/2,057,
  requested634,784,retained505,856,peak505,876. All drop counters return to0.
- Refreshed layout attribution1.041s/155,856KiB,three exact samples:
  generated104-byte row slots,outer212,992/inner245,760/strings284,615 bytes;
  standard Rust24-byte slots,outer49,152/inner245,760/strings210,944.
  Current extra237,511 retained bytes split163,840 row layout+73,671 strings.
  This supersedes the older pre-digit-cache memory gap without erasing it.
- Fresh matched system-allocator CPU probes2.648s/235,504KiB preparation,
  16.510s/32,736KiB combined sampling;20,000 original observed create/drop
  batches,checksum473727250,original2,048 iterations/seed7/input. Native
  diagnostic wall5.982s/reference10.480s are not benchmark timings. Of2,134
  main-thread samples,top-of-stack includes372 allocator free,282 tiny malloc,
  184 tsr_push_int,170 memmove,136 UTF-8 validation and85 tsr_int_text. Retained
  string/row allocation,formatting and cleanup remain concrete costs.
- Official quiet current-parent timing **2026-11-02T06:05:27.292Z**,
  14.522s/351,136KiB,maxload2.120,no override,original3processes/8warmups/
  31samples each. Construction controls **.009666/.015958ms** and
  **.009666/.019625ms** Tsrust/Node (1.65×/2.03×). Saved Rust references
  .009667/.009917ms reused,not retimed. Mutation .098666/.432375ms,
  saved Rust .098667. Interfaces .201083/.116708ms,saved Rust .413167;
  generated is already faster than that reference despite using more bytes.
  These are workload results;do not infer general object or JSON superiority.
- Trial changes only generated-consumer integer chain capacities from24 to the
  exact unsigned_abs().checked_ilog10() digit count plus sign,using the existing
  String.concat sizing expression. It does not repeat the rejected constant20
  capacity trial. Full original instrumented source hashes verified;original
  Rust flags/path and driver retained;restored sources after each compilation.
  Only interfaces and labeled_control contained affected chains in the selected
  seven cases. Full observed checksum/exactBits agree with parent.
- First paired3process×31sample trial,maxload2.181,3.092s/259,312KiB:
  interfaces **.201541→.212208ms (5.3% slower)**,Node .116875;
  labeled .282000→.279000,Node .274208. Follow-up maxload1.992,
  3.111s/245,952KiB: interfaces .205625→.210458 (2.4% slower),
  labeled .286833→.275541,Node .123125/.281667 respectively. A small ingestion
  ran during the follow-up;first clean paired slowdown independently rejects it.
- Trial layout profile0.520s/880KiB,three identical checksum-correct samples:
  allocations10,238/reallocations9 unchanged,requested816,552,
  retained565,064,peak565,083;string capacity106,312. Saves178,303 bytes
  (24.0% retained),but duplicates integer digit work and loses throughput.
  **Rejected before compiler edits.** No runtime behavior,ownership,limits or
  generated unsafe change;no claim of accepted performance gain.
- Ingested20 official current-parent histories;exact rounded median/mean rate/
  93counts and SQLite integrity verified. Public pin uses only official current
  compiler data,not rejected diagnostic timing. Reused all saved Rust timings.
  Whole-project formatter4.126s/201,584KiB,zero changes;three website tests pass,
  66-page build3.655s/891,776KiB and links/assets/anchors in116pages pass.
  Existing duplicate404 warning only. Compiler source unchanged;reuse preceding
  54 affected checks and all-target agreement rather than rebuild identical code.
- Pruned19 closed trial/CPU/layout artifacts (3,782,615bytes),including an
  obsolete temporary source snapshot. Current native/reference allocation profiles
  and official parent timing remain for next comparison;all owned runners terminal.
  No compiler tests rerun,no CI,push or deployment for this profiling checkpoint.
- Next: trace existing borrowed record/view emission before removing any local
  record string copy. Plain Player.name clones the owned input each iteration,
  but only borrowed describe calls consume it before the local dies. Establish
  whether current view/ownership machinery can represent this without changing
  escaping values or retaining source allocations. Row-wrapper storage remains
  a measured lead;104bytes are required by the current four-inline-string type,
  so shrinking it requires a proved storage choice,not a smaller heap allocation.
  No arena,RC backing or reduced inline threshold is approved by these profiles.

**Finite number strings — accepted locally, parent ac1590eb:**
- Fresh six-workload allocation profile **2026-11-02T05:36:37.662Z**,
  6.216s/318,384KiB; original deterministic samples and cleanup validated.
  Numeric-literal CPU probe uses original input/seed/2,048 iterations and
  independently checked checksum251352074,10,000 create/drop repetitions.
  Preparation1.559s/287,440KiB; sample5.683s/33,360KiB. Of2,122 main-thread
  samples,15 top-of-stack samples remain in label,35 in the number formatter,
  261 in grisu shortest conversion. Allocation/free/copy still dominate;
  this dispatch removal is a small optimization, not a formatter rewrite.
- Fresh quiet parent **2026-11-02T05:38:54.224Z**,11.912s/348,896KiB;
  original3processes/8warmups/31samples each,focused Node controls only.
  Preliminary original-driver generated-consumer comparisons rotate parent,
  direct call and Node,verify full original source hashes and exact outputs.
  First pass: numeric literals .397292→.393208ms,nullable .036792→.035375,
  mixed conversion .191917→.193875 with a slow first process for both direct
  and Node; maxload1.808. Follow-up maxload1.565,4.146s/275,968KiB:
  literals .396708→.389459,nullable .036500→.035584,mixed .193417→.189208.
  These are diagnostics; neither substitutes for the official candidate run.
- Compiler now calls the existing tsr_console_number formatter directly for
  finite String(number), avoiding a TsrConsoleValue and label dispatch.
  Digits/nonfinite cache paths remain unchanged. Traced direct,union,view-union,
  numeric-enum and indirect Object.hasOwn consumers. Added console support
  selection for the direct helper and a formatted discarded finite conversion
  witness. No new formatter,dependency,limit,unsafe,public signature or ownership
  policy. The existing helper becomes public inside the generated runtime so
  split runtime-module imports can reach it; this is not a Tsrust API addition.
- Initial test build196.394s/1,373,280KiB; numeric conversion caught two failures
  because the helper was private across runtime-module boundaries. Fixed at the
  shared helper,rebuilt197.199s/1,439,280KiB and reran all9 affected binaries.
  **54 checks pass:** numeric conversion6,number formatting5,ties2,console14,
  Object.hasOwn4,root spread5,enums6,addition10,concatenation2. Individual
  watchdogs3.138/6.721/1.034/12.081/1.036/2.606/1.043/4.741/1.031s,
  max698,736KiB. Existing1,024 random-float/edge and ownership/allocation
  witnesses pass at O0/O3. Frontend/signatures unchanged; editor agreement reused.
- Release177.876s/1,463,104KiB. Candidate allocation profile
  **2026-11-02T05:57:14.950Z**,36.235s/587,776KiB: all six complete sample
  dictionaries exactly equal parent,including checksum,allocation/reallocation,
  requested/retained/peak bytes and zero live bytes after drop. Interfaces
  emitted/instrumented source is byte-identical; all five numeric consumers change.
- Official quiet candidate **2026-11-02T05:57:51.202Z**,
  26.419s/351,552KiB,maxload3.792 versus parent2.266,no override:
  numeric literals **.398250→.392875ms,1.014×**,Node .173375→.173958;
  mixed conversion **.192625→.188750,1.021×**,Node .203125→.203750;
  nullable **.036959→.035666,1.036×**,Node .051500→.051833;
  pop **.207500→.198542,1.045×**,Node .220666→.223125;
  enums .036167→.035958,Node .035334→.035750. The byte-identical interfaces
  control drifts .205584→.206625 (+0.5%),Node .117084→.117958 (+0.7%);
  normalized control is1.002×. No altered workload regresses. Numeric-literal,
  mixed,nullable and pop Node-normalized gains1.017/1.024/1.043/1.057×.
- Ingested both official runs:24 exact history entries verified against rounded
  medians/mean rates/93sample counts;SQLite integrity passes. Original manifest,
  toolchain,oracle,harness,drivers,adapter and lock identities match. Saved
  handwritten Rust measurements reused;public pin selects official candidate.
- Compiler/LSP all-target check52.303s/1,492,112KiB passes;existing
  decode_plain dead-code warning only. Whole-project formatter4.130s/202,064KiB,
  zero source/snippet/fingerprint changes. Three website tests pass;66-page
  build3.636s/898,240KiB and links/assets/anchors in116pages pass;existing
  duplicate404 warning only. Pruned15 closed artifacts/39,521,196bytes;latest
  official candidate timing/allocation retained. All owned runners terminal.
  No CI,push or deployment. No full regression rerun for this emitter-only change.
- Next: return to object/string memory profiling with the current compiler.
  Investigate the measured104-byte Array<String> row-layout cost against the
  accepted small-array wins before choosing a representation change. First
  establish string-free record construction/access coverage,as planned below;
  record_mutation alone covers mutation,not construction. Do not repeat rejected
  integer writers/capacities or adopt an arena/RC policy without evidence and an
  owner design decision.

**Standalone nonfinite number strings — accepted locally, parent 76612b94:**
- Generated String(int) fallback already uses the same std to_string operation
  as the handwritten interfaces reference. Do not reimplement its formatter
  speculatively. Profile standalone String(number) special text instead.
- Fresh profile **2026-11-02T05:09:28.408Z**,6.242s/302,960KiB:
  nullable_number_null1,027 allocations/10 reallocations;
  numeric_string_conversion3,893/9. Finite controls: pop4,609/12,
  numeric_literals13,825/2,057,enums2,049/9,interfaces10,238/9.
  All checksums/cleanup/count determinism validated. Initial profiler selection
  named an unregistered undefined workload and was rejected before any probes;
  corrected against manifest IDs.
- Current nullable CPU probe uses original driver/independent checksum522694246,
  exact iteration count/seed7,100,000 observed-output creation/drop repetitions.
  Preparation1.033s/289,712KiB; full sample5.668s/32,464KiB. Initial10,000-loop
  diagnostic ended before the3s sample window; reran only diagnostic repetitions.
  Array push,Float label/format/tie and allocation/free work visible; diagnostic
  rather than formal timing. Original benchmark samples remain unchanged.
- Fresh quiet parent **2026-11-02T05:11:46.710Z**,11.941s/326,416KiB,
  original93samples/engine across six selected workloads,focused Node controls;
  standard Rust reference measurements reused.
- Candidate keeps the digit cache first,then reuses existing number_format_special
  only for nonfinite values; finite fallback keeps the original label formatter.
  A single is_finite test separates the slow path. Immutable static literals
  preserve independent mutation via the existing Cow behavior. No RC,dependency,
  runtime helper,limit,signature or public documentation change.
- Traced direct/union/numeric-enum conversion and indirect Object.hasOwn key
  conversion consumers. Extended native allocation witness for quiet/signaling/
  negative NaN payloads,±infinity,owned conversion and mutation; kept finite/
  random fallback checks. Expanded checked-TSR union/record mutation and single
  evaluation checks,plus discarded special conversion support selection.
  Seven initial affected binaries selected: numeric_string_conversion,
  number_formatting,number_text_ties,console_output,object_has_own,
  root_builtin_spread,enums. Build194.738s/1,499,216KiB; all42 checks pass:
  6/5/2/14/4/5/6; individual watchdogs3.105/6.732/1.030/11.954/1.039/2.626/
  1.036s,max663,088KiB. Quiet/signaling/signed NaN and mutation checks pass at
  O0/O3; finite/random label fallback remains exact. Release208.513s/1,470,976KiB. Candidate profile
  **2026-11-02T05:22:47.950Z**,6.718s/310,256KiB: nullable allocations
  1,027→514 (-513),realloc10 unchanged,requested402,429→399,009,
  retained/peak206,013→202,593 (-3,420 bytes). Numeric conversion allocations
  3,893→2,463 (-1,430),realloc9 unchanged,requested474,982→467,632,
  retained261,342→253,992,peak261,586→254,236 (-7,350 bytes).
  Every sample deterministic/checksum-correct,cleanup0; all four controls'
  allocation/reallocation/byte/cleanup samples unchanged.
- Quiet candidate **2026-11-02T05:23:12.059Z**,11.378s/399,280KiB,
  max load3.832 versus parent2.449,no override,original93samples/engine:
  nullable **.040834→.036750ms,1.111×**,Node .052083→.051958 (normalized1.108×);
  numeric conversion **.207250→.190708,1.087×**,Node .203208→.204917
  (normalized1.096×). All three native medians for each affected workload beat
  parent global. Pop .209333→.206916,enums .036291→.036208,interfaces
  .205333→.203167; no regression in these controls. Finite numeric_literals
  .394375→.403875,Node .174000→.174375 required diagnosis.
- Paired original-driver finite-literal diagnosis reconstructs full parent source
  by removing only the special branch; exact instrumented parent hash verified.
  Same source path/compiler/flags,rotate parent/candidate/Node,3processes after
  8warmups/31samples each,exact output and quiet checks. Parent .400792ms
  versus candidate .397792,Node .173916,max load1.699,watchdog2.073s/320,864KiB.
  No repeatable finite-control regression; do not claim a finite optimization.
  Preserve official measurements unchanged; do not claim a finite optimization.
- All-target compiler/LSP check52.769s/1,445,120KiB passed; existing decode_plain
  dead-code warning only. Twenty-four imported histories verified for exact
  rounded medians/mean rates/93counts,paired source/protocol/compiler identities,
  quiet policies and SQLite integrity. Integer interfaces control's emitted and
  instrumented source remains exactly identical. Saved Rust baselines reused.
  Public pin selects final official run; diagnostic timings are not substituted.
- Refreshed the two coupled addition suites because their allocation reference
  calls String(number): build4.158s/369,568KiB,addition10 and concatenation2 pass,
  watchdogs6.238s/503,280KiB and1.567s/206,576KiB. **54 total affected checks
  passed.** Frontend/signatures unchanged; existing editor agreement reused.
- Whole-project formatter zero files/snippets/fingerprints changed. Three website
  tests pass;66-page build3.620s/913,376KiB;links/assets/anchors in116pages pass.
  Existing duplicate404 route warning only. Current release compiler/LSP contains
  accepted source. No new runtime helper/table/dependency/unsafe/limit/ownership
  policy or public signature. Static literal reuse follows existing Cow rules.
  No CI,push/deployment; all owned runners terminal.
- Pruned 32,458,037 bytes across7 closed raw/CPU/paired-probe artifacts.
  Current candidate timing/allocation and54-check test binaries retained for
  next parent. No separate handoff/source snapshots created.
- Next: profile the remaining finite String(number) path. It still constructs
  TsrConsoleValue::Number and dispatches label although the domain is already
  known. Investigate calling the existing numeric formatter directly,including
  isolated support selection,only if fresh CPU evidence justifies it. Avoid
  reimplementing std integer formatting or repeating rejected append writers.
  Preserve object/string allocation plan and104-byte row-layout lead.


**Fractional string addition — accepted locally, parent e50398b5:**
- Final form: keep ordinary whole-number handling,then reuse existing TsrJoinText
  for all other numeric appends. Numeric enums whose declared members all meet
  the existing exact-integer formatting precondition use the existing int writer
  directly. Covers binary,chain and compound appends,including checked Number
  conversion. Integer writers,capacity allowances and template guard unchanged.
  Facts are precomputed once per declaration. No new runtime helper,dependency,
  unsafe,limit,public API or ownership policy. Historical trial details follow.
- Fresh allocation profile **2026-11-02T04:10:29.509Z**,5.179s/296,624KiB:
  concat22,522 allocations/2,057 reallocations,checksum909615361. Enum,interfaces
  and already-optimized template controls included; all samples/cleanup validated.
- Native CPU diagnosis uses original workload/driver/independent checksum,
  n2048/seed7/exact manifest input/10,000 observed creation/drop repetitions:
  preparation2.062s/284,432KiB; sample6.702s/33,728KiB,checksum909615361.
  Float formatting,allocator/free/copy costs visible; diagnostic,not formal timing.
- Fresh quiet parent **2026-11-02T04:14:01.372Z**,10.891s/320,976KiB,
  original93samples/engine across concat,enums,interfaces,template and Unicode
  chain control. Standard Rust reference timings reused.
- Added one formatted native witness covering binary/chain/compound and numeric
  enum fractions,one destination allocation,2,048 random f64s/edge values versus
  the original conversion path,UTF-8 prefix offsets,normal cleanup,and no added
  allocations (including scientific/special values). Existing order/unwind/tie
  witnesses remain. All six affected binaries building under watchdog:
  string_addition,string_concatenation,number_text_ties,array_join,templates,enums.
  Initial focused build194.661s/1,477,008KiB; all28 affected checks passed:
  string_addition10,string_concatenation2,number_text_ties2,array_join4,templates4,
  enums6; individual watchdogs5.765/1.575/1.048/2.069/2.611/1.042s,
  maximum537,968KiB. New2,048-random-float/allocation witness passes at O0/O3.
  Release210.600s/1,536,624KiB. Candidate profile
  **2026-11-02T04:22:27.342Z**,4.666s/306,800KiB: concat allocations
  22,522→21,498,reallocations2,057 unchanged,deallocations6,137→5,113,
  requested1,278,843→1,270,651 (8,192 saved),retained/peak922,016 unchanged.
  All samples identical,cleanup0; every control allocation sample unchanged.
- First quiet candidate **2026-11-02T04:22:52.559Z**,10.352s/353,440KiB,
  max load2.445 versus parent2.225,no override,93samples/engine. Concat native
  .471250→.465458ms (1.012×),Node .139750→.165500; enums .039084→.041792,
  Node .035292→.036041. Enum process medians .040625/.040042/.062417 vary
  substantially. Three unchanged controls' full emitted/driver hashes match
  parent; their timing drifts too. Bounded recheck of both changed workloads
  active before acceptance; no accepted speed claim yet.
- Recheck **2026-11-02T04:25:36.308Z**,7.750s/321,008KiB,max load1.732,
  no override: concat .459625/Node .135708,enums .041166/.035708.
  Resolve with paired original-driver diagnosis: reconstructed full parent
  instrumented source by reverting numeric code and removing newly selected join
  support; require exact official parent hash before compiling. First diagnostic
  rejected an incomplete registration removal before timing; corrected its
  generic-impl detection, then full parent hashes matched.
- Paired diagnosis rotates parent/guarded/direct-writer/Node,original3processes/
  8warmups/31samples,exact output and quiet guard. Parent→guarded→direct:
  concat .467708→.451959→.444000ms; enums .039792→.039792→.040167;
  Node .138166/.035500. Max load2.527,watchdog7.787s/360,496KiB.
- Simplified candidate: reuse existing writer for every non-whole Number append.
  Addition's old scientific label path uses two formatted strings, unlike the
  template baseline, so its guard may be unnecessary. Retain the template guard.
  Existing full-range allocation witness must prove the difference. Added an
  isolated fractional/scientific/nonfinite append with no console use to check
  support selection. Build195.820s/1,371,440KiB; all28 checks pass again:
  10/2/2/4/4/6; individual watchdogs5.207/1.569/1.042/2.086/2.595/1.033s,
  max492,160KiB. Full-range no-extra-allocation and isolated support witnesses
  pass at O0/O3,justifying scientific/special helper reuse in addition while
  retaining the independently justified template guard. Release build active;
  Direct-writer release210.736s/1,352,784KiB; profile
  **2026-11-02T04:38:34.787Z**,4.149s/345,680KiB,counts/bytes identical to
  guarded candidate. Official run **2026-11-02T04:38:54.354Z**,9.835s/339,744KiB,
  max load3.958,no override: concat .481792/Node .138333,enums .040583/.035875;
  unchanged controls also slower. No acceptance from this shifted-machine run.
- Final paired actual-binary diagnosis verifies retained parent binaries' hashes
  and original full source identity,candidate instrumented-source and executable
  identities; rotate parent/candidate/Node with original sample protocol and
  exact-output checks. Concat .472958→.457458ms (1.034×),Node .136917;
  enums .038750→.040417 (4.3% slower),Node .036208. Max load2.509,
  watchdog3.610s/329,472KiB. Thus the whole-enum regression is real; fix before
  retaining fractional savings.
- Current repaired candidate precomputes exact-whole numeric-enum facts once per
  declaration (no per-use member rescans). It recognizes direct enum values or
  checked Number(enum) conversion,then uses the existing int writer where every
  member satisfies the existing whole-format precondition (integral,abs<2^53).
  Applies to binary,chain and compound appends; fractional/wide enums retain the
  f64 path. No new limit or rounding. Added negative/zero/exact-boundary and
  fractional/wide enum witnesses. Private string-chain helpers now receive the
  emitter to read those facts. Affected rebuild active; final candidate unaccepted.
  Enum-proof build194.686s/1,451,936KiB; all28 checks pass again:
  10/2/2/4/4/6,watchdogs5.279/1.560/1.040/2.085/2.598/1.034s,
  max547,488KiB. Exact-boundary/negative/zero enum and fractional/wide fallback
  checks pass at O0/O3. Final release build active before official remeasurement.
  Final release209.139s/1,686,032KiB. Final profile
  **2026-11-02T04:54:06.706Z**,4.148s/284,960KiB: same1,024 allocation/8,192
  requested-byte savings; all control samples and cleanup remain unchanged.
- Final quiet official run **2026-11-02T04:54:23.307Z**,9.832s/341,792KiB,
  max load2.308,no override,original93samples: enums .039084→.036291ms
  (1.077×),Node .035292→.036375,normalized1.110×; all three candidate process
  medians .036208/.036709/.036208 beat parent global. Enum regression fixed.
  Concat .482584/Node .139875 in this phase,while all three unchanged controls
  drift about5% versus the earlier parent. Keep this official timing honestly.
- Final concat executable SHA and complete instrumented-source hash match the
  actual-binary paired1.034× winner exactly. Reuse that controlled A/B rather
  than rerunning identical artifacts until a favorable official phase appears.
  All three paired concat candidate process medians .459917/.455000/.452833
  beat every parent process median .464167/.489375/.470125. Fractional savings
  have a verified faster matched consumer; whole enums improve independently.
- All-target compiler/LSP check52.749s/1,284,080KiB passed with existing
  decode_plain dead-code warning only. Frontend/signatures unchanged; reuse
  existing editor agreement. Forty-four formal histories ingested and verified
  for exact rounded medians/mean rates/93counts,all paired source/workload/
  protocol/compiler identities,quiet policies,control hashes,allocation samples
  and SQLite integrity. Saved hand-written Rust baselines reused. Public pin
  selects final official run only; rejected variants and diagnostic timings are
  not substituted into the published table.
- Whole-project formatter zero files/snippets/fingerprints changed. Three website
  tests pass;66-page build3.632s/945,776KiB;links/assets/anchors in116pages pass.
  Existing duplicate404 route warning only. Release compiler/LSP contains the
  accepted source. No CI,push/deployment. All owned runners terminal.
- Pruned 88,992,496 bytes across16 closed raw/CPU/paired-probe artifacts.
  Keep final official timing/allocation for next parent; current28-test binaries
  retained. Controlled diagnostic summary/identities above remain reproducible
  from commits; do not preserve separate source or handoff snapshots.
- Next: reprofile standalone owned numeric string conversion now that append
  paths no longer need their ordinary formatting temporaries. Compare actual
  allocation/CPU work with existing reference before choosing another change;
  do not repeat rejected integer-writer variants. Keep object construction and
  104-byte row-layout leads tracked,with Rust ownership/small-array wins intact.


**Fractional template formatting — accepted locally, parent b8d863c2:**
- Corrected the next-lead description: Number templates call tsr_number_to_string,
  not number.label. The latter belongs to string addition. Templates allocate
  temporary owned fractional text and copy it into their independent output.
- Parent allocation profile **2026-11-02T03:45:58.864Z**,5.175s/258,544KiB:
  template_primitive_ascii checksum769987644,4,100 allocations/3,327 reallocations,
  requested313,683/retained138,006/peak138,269 bytes,input13,cleanup0. Exactly
  2,048 fractional conversions accompany2,048 output strings. Unicode-only,
  interfaces and string-concatenation controls captured too.
- Native CPU diagnosis uses original workload/adapter/independent checksum,
  n2048/seed7/input "dynamic ASCII",10,000 observed-output/drop repetitions:
  preparation1.040s/273,520KiB; sample3.612s/33,072KiB,checksum769987644.
  2,133 main-thread samples show float formatting/tie handling and allocator/
  realloc/copy costs. Initial diagnostic used the wrong Unicode input; its
  checksum rejected it, corrected before collecting successful evidence.
- Fresh quiet parent **2026-11-02T03:47:33.358Z**,12.429s/332,176KiB,
  original93samples/engine across template primitive/Unicode,interfaces,concat,
  radix controls. Saved standard Rust reference timings reused.
- Initial candidate was one emitter-line replacement: keep tsr_push_whole_number, reuse
  existing TsrJoinText::append_join_text for remaining Number interpolation.
  Existing support detection emits the trait. No new runtime helper/dependency,
  unsafe,limit,ownership policy or public API. Parity witnesses cover ties,
  scientific ranges,specials and Unicode prefix offsets.
- Extended templates' native helper to format successful fixtures in memory;
  existing allocation/unwind witness adds common fractions,2,048 random f64s,
  extremes and scientific-allocation comparison against the old template path.
  Focused builds selected templates,number_text_ties,array_join,number_to_string,
  string_addition. The first check examined scientific formatting allocations
  because existing join uses a different temporary buffer.
- Initial build192.665s/1,517,152KiB; templates3pass/1fail. Existing panic hook
  hid assertion details; restored diagnostics around new checks (test-only rebuild
  1.560s/262,528KiB). Focused rerun proves scientific-allocation regression in
  unbounded helper reuse, not text mismatch. Retain the witness. Candidate now
  reuses the existing helper only in the existing plain decimal range1e-6..1e21;
  scientific/special fallback stays byte-for-byte the original number_to_string
  append. This is the existing format boundary, not a new limit/ownership rule.
  Guarded build was interrupted after60.208s/1,498,880KiB due a mistaken
  escaping diagnosis; direct byte inspection found source escaping already
  correct. Restarted the same guarded source under watchdog. Build196.790s/
  1,554,480KiB; all22 affected checks pass: templates4,number_text_ties2 (339
  reference values),array_join4,number_to_string3,string_addition9. Individual
  watchdogs2.615/1.042/2.088/1.555/4.712s,max598,560KiB. New2,048-bit-pattern
  parity,plain allocation andscientific no-extra-allocation checks pass at O0/O3.
  Release212.825s/1,466,480KiB. Candidate profile
  **2026-11-02T04:03:55.203Z**,4.665s/299,840KiB: template allocations
  4,100→2,052,reallocations3,327 unchanged,deallocations during work2,051→3,
  requested313,683→248,147 (65,536 saved),retained138,006 unchanged,
  peak138,269→138,237. All samples identical and live bytes reach0. All controls'
  allocation samples,emitted and instrumented source hashes match parent exactly.
- Quiet candidate **2026-11-02T04:04:17.799Z**,10.361s/357,808KiB,max load2.502
  versus parent2.249,no override,original93samples/engine: template
  **.283000→.257500ms,1.099×**; Node .168250→.171292,normalized1.119×.
  All three native process medians (.239875/.257542/.263750) beat parent global.
  Byte-identical controls drifted: Unicode .124500→.129167,interfaces
  .203625→.207083,concat .476958→.497291; their Node shifts respectively
  .026458→.026791,.116875→.118000,.136541→.142041. Radix .833958→.830959 flat.
  All paired workload/protocol/versions/oracle/driver/harness/lock hashes match;
  only compiler input emit/mod.rs differs.
- All-target compiler/LSP check52.246s/1,285,520KiB passed; existing decode_plain
  dead-code warning only. Frontend/signatures unchanged; reuse existing editor
  agreement evidence rather than rerunning the LSP matrix for private emission.
- Bounded control follow-up **2026-11-02T04:05:58.431Z**,7.763s/335,792KiB,
  max load2.475,no override,original93samples: Unicode .128833/Node .027500,
  interfaces .212292/.118750,concat .501375/.167083. Drift remains in exactly
  byte-identical programs; do not describe these as optimized or attribute their
  variation to the source change. Preserve both runs; no whole-matrix claim.
- Accepted guarded reuse: existing decimal-range boundary only; scientific/special
  fallback unchanged. No new runtime helper, dependency, unsafe, limit, API or
  ownership policy. Twenty-six parent/candidate/follow-up histories ingested;
  exact rounded medians/mean rates/93counts,source/workload/protocol/compiler
  identities,quiet policies and SQLite integrity verified. Standard Rust reference
  timings reused. Public performance pins include candidate and latest controls.
- Whole-project formatter zero files/snippets/fingerprints changed. Three website
  tests pass;66-page build3.645s/897,376KiB; links/assets/anchors in116pages pass.
  Existing duplicate404 route warning remains; rendered routes/link checks pass.
  Current release compiler/LSP contains the accepted source. No CI,push/deployment.
- Pruned 37,156,416 bytes across6 closed parent raw/CPU-probe artifacts;
  current candidate timing/allocation and control follow-up remain for next A/B.
  Current22-test binaries retained; all owned runners are terminal.
- Next: profile fractional string-addition temporaries in string_parts and binary
  numeric append paths, which still copy number.label() into the destination.
  Consider the same existing guarded writer only after matched allocation/CPU
  profiles, and preserve scientific allocation behavior and integer paths. Reuse
  this section's scientific witness and the existing tie corpus. Object plan and
  104-byte row-layout lead remain tracked; no arena/ownership change approved.

**Integer append final outcome — rejected after controlled comparison:**
- Final decimal-pair/std candidate release211.098s/1,304,928KiB; all-target
  compiler/LSP check52.268s/1,465,616KiB passes with existing decode_plain
  dead-code warning. All27 affected native checks passed as detailed below.
  Allocation profile **2026-11-02T03:34:54.726Z**,3.617s/286,912KiB: every
  allocation/reallocation/requested/retained/peak/cleanup sample matches parent.
- Official quiet candidate **2026-11-02T03:35:22.088Z**,16.043s/324,368KiB,
  maximum load2.501,no override,original93samples/engine. Against reverse parent:
  labeled .299625→.207833ms (1.442×),enum .041208→.035125 (1.173×),
  interfaces .213500→.203292 (1.050×),template .300125→.292334 (1.027×),
  function_expressions .344834→.327708 (1.052×). Node-normalized gains
  1.484/1.188/1.031/1.011/1.086× respectively. Controls stable or faster.
  But concat .503416→.519375 (3.17% slower),Node .144042→.144417; unresolved.
- Final paired concat diagnosis uses the actual official candidate executable;
  parent consumer reconstructed from committed runtime helper, full instrumented
  hashes verified against official parent/candidate, same path/compiler/flags.
  Alternate3processes,8warmups,31samples,exact output/checksum and quiet guard.
  Parent .477292→candidate .481375ms (0.86% slower),Node .138417,
  max load2.373,watchdog5.182s/403,664KiB. Preliminary five-workload diagnostic
  improvement did not establish a regression-free final gain. Reject this
  variant too; do not publish a selected subset of its wins.
- Restored runtime and trial test byte-exactly to HEAD; new behavioral witness
  was trial-only. Accepted compiler/LSP release rebuilt211.114s/1,470,448KiB.
  Restored profile **2026-11-02T03:43:26.603Z**,4.654s/283,968KiB matches parent
  compiler inputs,generated/instrumented hashes and every allocation sample.
  All102 official timing histories ingested; exact rounded medians/mean rates/
  93sample counts,paired workloads/protocols/versions/oracles/compiler identities,
  quiet policies and SQLite integrity verified. Standard Rust baselines reused.
  Whole-project formatter zero files/snippets/fingerprints changed. Pruned
  230,227,684 bytes across46 closed raw/candidate/obsolete trial-test artifacts;
  reverse-parent timing and restored allocation profile remain for next parent.
  No optimization from this section is retained; no public pin changes.
  No CI,push or deployment. No full regression rerun for byte-restored source.
  Next: change hypothesis, reprofile non-integer temporary allocations in string/
  object workloads (e.g. fractional template number.label creates owned text
  before copying into the destination). Trace ownership and profile before
  changing it. Do not repeat these integer-writer variants unchanged. Existing
  104-byte row-layout lead remains conditional on preserving small-array gains.

**Closed integer-writer trial details (historical):**

**Integer append CPU — rejected trials, parent 537437e2:**
- Fresh native interfaces CPU probe uses existing workload adapter/checksum,
  opt-level3/overflow checks, n2048/seed7/input19bytes and30,000 observed-output
  creation/drop batches. Checksum473727250; CPU preparation2.586s/310,496KiB,
  sampled execution9.306s/33,264KiB (diagnostic, not a timing comparison).
  /tmp/tsrust-integer-append-native.sample has2,107 main-thread samples:
  189 top-of-stack tsr_push_int,133 from_utf8, plus allocator/copy/drop work.
- Traced all tsr_push_int callers: binary String(int) appends, addition chains/
  compound sums, templates and tsr_push_whole_number's f64/enum path. Trial adds
  only an ASCII char push and early return for0..9; existing general integer
  writer remains unchanged. No storage policy, sizing, dependencies, API or
  limit changes; generated code stays safe Rust.
- Fresh quiet parent **2026-11-02T02:59:20.410Z**,watchdog30.593s/326,512KiB,
  original93samples per selected native/Node engine across9 workloads. Saved
  handwritten Rust timings reused. Candidate tests build under watchdog:
  string_addition,numeric_string_conversion,string_concat_integer_fusion,
  string_concatenation,templates. New formatted witness exhausts digits, signed
  extremes,1,024 deterministic integers against std, nonempty UTF-8 destination,
  no temporary/reallocation with spare capacity, f64 whole/special fallback and
  cleanup at O0/O3. All27 checks passed across the five sequential binaries:
  10/6/5/2/4; watchdogs5.182/2.083/3.143/1.035/2.602s,max511,248KiB.
  Focused test build194.122s/1,521,824KiB. Release build active under watchdog;
  candidate release211.080s/1,440,352KiB; allocation profile
  **2026-11-02T03:07:21.762Z**,4.133s/307,264KiB: all samples/bytes/counts
  unchanged from accepted parent. First quiet candidate timing
  **2026-11-02T03:07:50.581Z**,14.475s/350,752KiB,max load2.850; labeled_control
  .283250→.218500ms (1.296×),enums .038833→.032208 (1.206×),interfaces nearly flat.
  Recheck6 unresolved cases **2026-11-02T03:08:35.713Z**,12.967s/350,000KiB,
  max load2.252. String_concatenation remained .505875 versus parent .474584ms,
  while Node changed .133958→.172417ms; template .280459→.289916 (Node
  .167750→.170416). Unchanged controls drift too. No acceptance yet.
- Reverse-parent comparison in progress: removed only the four-line fast path
  (runtime source matches HEAD), retained behavioral regression witness. Release
  rebuilding under watchdog; remeasure parent to distinguish current-machine
  variation from a source regression before reapplying or rejecting candidate.
- Reverse parent release210.048s/1,502,704KiB; quiet run
  **2026-11-02T03:14:32.779Z**,15.497s/353,232KiB,max load2.351,no override,
  original93samples/engine. Runtime/compiler and emitted source hashes match
  initial parent. Native parent now template .300125,concat .503416 (Node
  .173042/.144042),versus candidate .284792/.505666. Initial apparent template
  regression was machine drift; concat difference is0.45%,within observed drift.
  Labeled parent .299625 vs candidate .218500; enum .041208 vs .032208;
  interfaces .213500 vs .201791. Reapplied the identical tested four-line
  candidate; final release build/check/timing pending, no public pin change yet.

**Integer append trial follow-up — rejected, no runtime change retained:**
- Final four-line candidate release212.264s/1,477,344KiB; all-target compiler/LSP
  check52.803s/1,396,928KiB passes with existing decode_plain dead-code warning.
  Final quiet **2026-11-02T03:20:04.416Z**,16.030s/350,208KiB,max load2.703,
  original93samples: labeled .229375ms,enums .034500,interfaces .202458,
  template .301834,concat .510333. Strong small-number gains but concat unresolved.
- Paired original-driver concat diagnostic reconstructs parent consumer by removing
  only the four-line branch and asserts instrumented hash equals reverse-parent
  record; candidate hash matches final run. Same source path/compile flags,
  alternate3processes/8warmups/31samples,checksum/exact-output checks,quiet guard.
  Parent .472750ms versus candidate .484333 (2.45% slower),Node .136708;
  max load1.504,watchdog5.183s/395,440KiB. Reject four-line-only variant.
- Tried std fmt::Write/write! as the general fallback, retaining direct one-digit
  push. Disposable verified-consumer paired diagnostic (not official timing)
  uses five workloads,original drivers/samples,quiet guard,checksum/exact output:
  parent→candidate interfaces .203000→.186959,labeled .286917→.211291,
  enum .038917→.032417,template .285000→.296166,concat .474709→.464875.
  Max load1.548,watchdog8.796s/330,704KiB. Template's int count is0..96;
  std formatting helps larger ints but costs more for frequent two-digit values.
- Current unaccepted runtime trial directly writes sign/tens/ones for magnitude<100,
  then uses std's integer formatter for larger values. This is a decimal-digit
  algorithm choice, not an ownership threshold or limit. Safe unsigned_abs covers
  MIN; no digit buffer/UTF-8 validation or temporary allocation. New witness now
  exhausts -101..101 plus extremes and deterministic random ints. Paired-consumer
  diagnosis passed before rebuilding compiler/tests: parent→candidate interfaces
  .202334→.195708ms,labeled .282750→.196750,enum .039583→.032792,
  template .284250→.274917,concat .483833→.474417. Max load1.768,
  watchdog8.823s/324,080KiB. All five faster, same checksum/exact observations.
  Affected test build193.101s/1,467,616KiB. All27 final-candidate checks pass:
  string_addition10,numeric_string_conversion6,string_concat_integer_fusion5,
  string_concatenation2,templates4; watchdogs5.226/2.085/3.170/1.036/2.081s,
  maximum524,272KiB. Final-candidate release build active under watchdog;
  official profiling/quiet timing/all-target recheck remain. No acceptance yet.

**Integer concatenation capacity — rejected, parent 4f1471fe:**
- Traced string_parts callers (addition chains/compound sum appends) and binary
  numeric appends. Trial reserves20 rather than24 bytes only for i64 operands
  (signed minimum needs20); f64/enums keep24. No digit-count calculation,
  ownership change, new limit or unsafe; exact concat/templates unchanged.
- Fresh parent profile **2026-11-02T02:42:11.807Z**,watchdog5.712s/275,136KiB;
  candidate **2026-11-02T02:50:48.978Z**,6.243s/268,032KiB. Each three samples
  identical/checksum-correct; all live bytes reach zero after cleanup.
  Interfaces10,238 allocations/9 reallocations unchanged; requested994,855→962,087,
  retained743,367→710,599,peak743,386→710,618 bytes (32,768 saved).
  Function_expressions17,703/9 unchanged, requested988,742→972,358,
  retained601,950→585,566,peak602,039→585,655 (16,384 saved).
  Labeled_control7,981/2,057 unchanged, requested1,187,492→1,130,148,
  retained/peak876,612→835,652 (57,344 requested/40,960 retained saved).
  Enum and numeric record controls exactly unchanged, including emitted/driver hashes.
- Added trial-only formatted native witness for binary/chain/compound forms,
  UTF-8 prefix, signed extremes/zero/single/multiple digits, one allocation/no
  reallocation, requested capacity and cleanup at O0/O3. All27 checks passed:
  string_addition10,numeric_string_conversion6,string_concat_integer_fusion5,
  string_concatenation2,templates4. Watchdogs5.204/2.090/3.133/1.033/2.086s,
  peak601,264KiB; test build192.774s/1,399,296KiB; release210.063s/1,558,544KiB.
- Quiet parent **2026-11-02T02:42:27.425Z**,7.793s/291,904KiB,max load2.290;
  candidate **2026-11-02T02:51:51.679Z**,56.376s/239,104KiB,max load3.985 after
  waiting for load below4. No override; original3processes×31samples after8warmups.
  Native milliseconds parent→candidate: interfaces .204750→.199125 (1.028×),
  function_expressions .321125→.329625 (.974×),labeled_control .284916→.284875.
  Corresponding Node .117875→.120458,.335334→.353250,.274208→.275875.
  Unchanged record control had a slow first process; recheck justified.
- Recheck **2026-11-02T02:52:28.669Z**,9.834s/354,080KiB,max load3.291,
  no override,original93samples/engine: interfaces native .201666/Node .116667,
  function_expressions .326167/.343000,labeled_control .285584/.272042.
  Normalized versus parent1.005/1.007/.990×. Enum .039792/.036792 versus
  parent .038583/.035542; numeric record .098667/.431167 versus .101000/.431458.
  Smaller requested memory did not establish a reliable speed gain without
  regressions. **Reject:** restored both emitter and trial test byte-exactly to
  accepted HEAD. Candidate test pass is evidence about the rejected trial only.
- store:ingest added30 histories; exact rounded median/mean rate/93sample counts,
  matching workload/protocol/versions/oracle/driver/harness identities, unchanged
  control source hashes and SQLite integrity verified. Saved standard Rust
  measurements reused. Rejected timestamps excluded from public performance pins.
- Accepted compiler/LSP release rebuilt211.189s/1,481,184KiB. Restored profile
  **2026-11-02T02:56:44.223Z**,4.154s/302,768KiB matches every parent compiler
  input, generated/instrumented source hash, allocation sample and cleanup result.
  Whole-project formatter zero files/snippets/fingerprints changed. Pruned
  119,510,514 bytes across15 closed raw/candidate artifacts; fresh parent timing
  and restored allocation profile remain. Trial-only test binaries removed;
  rebuild affected binaries if needed. No CI,push or deployment.
  No broader regression rerun needed for reverted compiler source. Next: profile
  residual integer append CPU work in interfaces before trying the existing
  tsr_push_int digit loop/UTF-8 validation, including single-digit inputs. Do not
  assume lower capacity or a replacement formatter makes it faster. The104-byte
  row-layout lead remains tracked; preserve earlier small-array wins.

**Standalone empty object schemas — accepted locally, 2026-11-02:**
- Parent **9b016b90**, clean tree. Fresh current-release witnesses show both
  `s.object({}).retain()` and plain `s.object({})` fail before retention with
  "Unknown generic record type"; watchdog .520/.523s,5,072/5,360KiB.
  Prior diagnostic-options test created a leaf schema before its empty object,
  hiding an initialization-order defect.
- Root: schema namespace preparation registers Schema<T> through leaf binding
  signatures. A standalone empty composite has no leaf signature. Collect one
  composite-builder span and explicitly intern Schema before generic processing;
  leave generic validation and runtime behavior unchanged. Scope: namespace.rs,
  schema_decode native regression, schema_editing checked completion/hover.
- New native witness compiles each plain/strip/retain policy independently,
  checks owned/source success and retained private-key extras, non-object
  validation errors and malformed-JSON errors. Uses existing format round trip.
  New editor witness independently checks all three policies without prior leaves.
- Focused compiler build passed **194.295s/1,639,584KiB**. First decoder
  run: **9 passed/one new fixture failed**,24.377s/1,085,344KiB. The fixture
  wrongly expected strict default unknown fields to succeed; runtime correctly
  rejected them. Fixed the fixture to verify default unknown-field rejection
  explicitly and test extra-field success only under strip/retain. No product
  relaxation. Retry-only build1.561s/380,848KiB; new witness live session90912,
  /tmp/tsrust-empty-schema-retry.log. Preserve nine earlier passes.
- Corrected new witness passed **20.165s/363,168KiB**, nine original decoder
  passes preserved. Remaining suites passed: diagnostic options11
  **21.761s/998,736KiB**; recursive10 **22.790s/1,078,048KiB**; literals9
  **22.789s/972,544KiB**; unions13 **55.936s/1,077,568KiB**. Total **53
  affected compiler/native invocations passed**, no skips. Whole-project format
  completed with zero files/snippets/fingerprints changed.
- LSP build passed **181.252s/1,690,800KiB**. schema_editing:16pass,
  one new fixture failed because console.log(retainedValue) inspects JsonValue,
  which is explicitly unsupported. Changed only the fixture to print a key
  count. Retry build1.565s/397,920KiB; corrected witness passed .520s/32 sampled
  KiB,16 prior passes preserved. All17 schema-editing cases now pass.
- Required editor agreement completed with no failures/skips: completion
  matrix3 **498.984s/49,760KiB**, checker agreement3
  **518.845s/100,064KiB**, editor consistency27 **14.125s/123,568KiB**.
  Together with schema_editing17, **50 editor invocations passed**. Combined
  compiler/native/editor total **103**; shared support checks are repeated across
  binaries, not103 distinct features. Each binary used three threads and its own
  5GB/30min watchdog; no resource stop.
- Compiler/LSP all-target diagnostics passed **110.795s/1,508,576KiB**.
  Only the preexisting decode_plain dead-code warning in the compiler lib-test
  target remains. Final whole-project format changed zero files/snippets/
  fingerprints. No generic validation, runtime defaults or public API changed.
  No performance claim or runtime timing: the standalone case cannot compile on
  the parent, so this is a correctness checkpoint. SQLite unchanged; test logs
  are not benchmark measurements. See latest Git checkpoint for accepted source.
- Next: refresh release compiler/LSP, then profile object/string allocation using
  the owner-approved plan below. Current debug consumers already execute the
  corrected schema behavior. No new CI, push, release or deployment.


**Single-digit String(number) — accepted locally, 2026-11-02:**
- Parent **e486ede7**, initially clean. Native parent allocation profile
  **2026-11-02T01:57:42.671Z**,5.199s/281,952KiB: enum_values4097alloc/9realloc,
  nullable_number_null1367/10,array_pop_number6145/12; three exact samples and
  zero bytes after drop. System-allocator CPU probe of actual enum workload
  (2048iterations,200,000batches) confirms label/spec_to_string/malloc costs,
  checksum **183779782**, watchdog20.666s/32,912KiB. CPU probe is not timing
  evidence; /tmp/tsrust-number-digit-parent.sample.
- Quiet parent **2026-11-02T02:01:30.230Z**,15.053s/373,328KiB,max load2.103,
  no override,93samples/engine for enum_values,nullable_number_null,array_pop_number,
  array_shift_number,number_to_string_radix,number_to_fixed_default,interfaces,
  record_mutation. Native/Node only; reuse pinned Rust. store:ingest added16
  histories, verified exact medians/rates/counts,source/compiler hashes,integrity.
- Traced direct and union StringConversion numeric/number-enum paths and
  Object.hasOwn's transitive scalar fallback. A shared emitter source expression
  snapshots f64 once, reuses tsr_int_text only for exact values0..9, otherwise
  retains the exact old TsrConsoleValue::Number.label fallback. No runtime API,
  table,dependency,unsafe,limits or editor signature changes. String(-0) remains
  "0"; console inspection's "-0" and input sign bits remain unchanged. ObjectHasOwn
  union-key path, numeric methods and concatenation's digit writer unchanged.
  Both String and TsrConsoleValue fallback markers preserve runtime selection,
  including discarded calls; no new runtime support hook is needed.
- Added formatted native witnesses for direct/copy-union/view-union/enum paths,
  all digits,fractions,nonfinite/zero spellings,cloned record/array mutation and
  single evaluation; isolated discarded call witness. Allocation witness asserts
  zero allocations/borrowed storage for digits and both zeros, native into_owned,
  mutation isolation,owned fallback,edge floats and1024 deterministic random
  f64-bit patterns against the unchanged label formatter at optimization0/3.
- Affected build passed **198.267s/1,485,872KiB**. All **51 invocations
  passed** sequentially: enums6,numeric conversion6,number formatting5,ties2,
  number.toString3,console14,hasOwn4,root spread5,concatenation2,primitive identity4.
  Watchdogs2.069/1.549/7.216/1.031/2.056/12.424/1.038/2.601/1.032/2.593s,
  max native-suite peak664,160KiB. Formatting/no-generated-unsafe assertions,
  zero-allocation digit checks and1024 random fallback patterns at O0/O3 pass.
  No frontend/signature changes; preserve existing editor acceptance.
- Release build passed **207.049s/1,583,040KiB**. Compiler/LSP all-target
  diagnostics passed **53.273s/1,582,128KiB**, only preexisting decode_plain
  dead-code warning. Private emitter-only change preserves frontend/editor facts;
  prior50 editor agreement passes remain applicable.
- Candidate allocations **2026-11-02T02:19:01.337Z**,3.626s/326,048KiB:
  three exact samples and source hashes match, zero live bytes after both drops.
  enum_values **4097→2049alloc**,requested540,268→513,632,retained/peak
  327,692→301,056. Nullable numbers **1367→1027alloc**,requested
  405,829→402,429,retained/peak209,413→206,013. Pop numbers **6145→4609alloc**,
  requested1,647,936→1,646,400,retained812,544→811,008,peak812,576→811,040.
  Reallocations remain9/10/12; no changes to during-work deallocation counts.
- Quiet candidate **2026-11-02T02:19:53.811Z**,13.456s/389,008KiB,max timed
  load2.347,no override. Original93samples/engine/case; four affected cases:

  | Workload | Parent ms | Candidate ms | Raw speed | Node ms, parent→candidate |
  | --- | ---: | ---: | ---: | --- |
  | enum_values | .067875 | .038792 | 1.750× | .036500→.035667 |
  | nullable_number_null | .044250 | .041292 | 1.072× | .051333→.051542 |
  | array_pop_number | .223000 | .208083 | 1.072× | .220250→.219459 |
  | array_shift_number | .221459 | .206750 | 1.071× | .256625→.252542 |

  Enum native process medians .038792/.039250/.038792,normalized1.710×;
  other affected normalized gains1.054–1.076×. Every affected native process
  median beats its parent global median. Unchanged controls initially varied:
  interfaces .202208→.208792, fixed .077833→.079458,radix .810291→.825708,
  mutation .098667→.098667. Their emitted and driver-instrumented source hashes
  are exactly identical across parent/candidate; no code change in these cases.
- Bounded control follow-up **2026-11-02T02:22:44.353Z**,6.224s/272,240KiB,
  max load2.091,no override,original93samples: interfaces native **.201125**,
  Node.117375; fixed native **.077125**,Node.077709. Native timings return to
  parent level; normalized1.019/.989×. This resolves the initial2–3% drift in
  byte-identical controls without rerunning affected gains or claiming a control
  optimization. Preserve both candidate runs; no whole-matrix timing claim.
- All paired manifest,driver,oracle,adapter,harness,lock,version/workload hashes
  match. No external native dependency change; only compiler input emit/mod.rs
  differs. store:ingest added16parent+16candidate+4control histories; exact
  median6/mean rate3/93counts,all source identities,quiet policies and SQLite
  integrity verified. Rust reference timings reused, not rerun.
- Public pins select accepted affected results and the latest control pairs.
  Whole-project formatter zero files/snippets/fingerprints changed. Three website
  tests,66-page build,links/assets in116pages pass; website3.636s/966,080KiB.
  Current compiler/LSP release consumers contain accepted code. No CI,push or
  deployment. Source change is nine emitter lines plus focused regression tests;
  no new runtime function, table,dependency,unsafe,limits or public signature.
- Pruned **74,835,547 bytes** across9 closed raw/probe/older test outputs after
  verification. Current51-test artifacts and latest candidate timing/allocation/
  control follow-up remain for the next parent; older int-only suites rebuild
  when needed. Next object-memory lead:
  inspect integer concatenation's conservative24-byte capacity allowance against
  i64's exact20-byte maximum, using current interfaces profiles and full min/max
  witnesses before changing sizing. Test timing rather than assuming fewer bytes
  wins. The104-byte output-row layout remains tracked; preserve small-array gains.


**Single-digit String(int) — accepted locally, 2026-11-02:**
- Parent **a984ce1f**, initially clean. Reuses the completed interfaces allocation/
  CPU profile and baseline below. Additional affected parent controls completed
  **2026-11-02T01:30:31.359Z**, **9.377s/296,016KiB**, max load2.228,no
  override: nullable_int_null,declaration_lists,array_splice_int,labeled_control,
  native+Node,93samples each. Eight histories ingested; exact medians/rates/
  counts/workload and compiler hashes/SQLite integrity verified before editing.
- Traced all String(int) emission paths: direct value_builtin_call and int members
  in union_observation. Both now call one private runtime tsr_int_text helper.
  Decimal digits0..9 borrow the existing static ASCII table through tsr_char_text;
  all other integers keep i64::to_string. No new table, dependency, generated
  unsafe, reference counting, limits or public signature. Float conversion,
  radix/default int.toString and concatenation's existing digit writer remain
  untouched. Object.hasOwn union-key conversion keeps its existing path; its
  direct scalar-key fallback reuses StringConversion and therefore the new helper.
  The object_has_own suite covers this transitive caller. Runtime support is
  selected by String types or the explicit helper marker. Generic validation/
  checker/editor code unchanged.
- New native/format-round-trip witness checks direct/copy-union/view-union
  conversion, all digits,negative/large/min/max integers, null/undefined/boolean/
  text branches, single evaluation, record/array cloning, captures and independent
  subsequent mutation. Runtime witness uses existing counting-allocator pattern
  to assert zero allocations for digits, borrowed storage, native into_owned,
  mutation isolation and owned fallback at optimization levels0/3.
- Initial affected build passed **195.226s/1,574,800KiB**. Before execution,
  review found support selection for discarded `String(7)`: emitted a helper
  call but no helper definition because no String type/text marker remained.
  Added explicit tsr_int_text support-selection marker and a minimal formatted
  native regression; no unrelated runtime selection changes. This follow-up
  invalidated the first artifact set; execution waited for the final rebuild.
- Final affected build passed **196.803s/1,606,736KiB**. Added the transitive
  object_has_own suite with a2.067s/270,096KiB incremental target build.
  All **35 invocations passed** across nine sequential suites: numeric conversion4,
  borrowed runtime2,concatenation2,construction4,int.toString4,enums6,
  primitive identity4,root spread5,hasOwn4. Watchdogs2.071/1.048/1.036/2.092/
  2.076/1.043/2.635/2.603/1.045s; maximum sampled native-suite peak532,912KiB.
  Formatting witnesses and emitted-no-unsafe assertions passed. Frontend/editor
  signatures unchanged; reuse the preceding50 editor agreement passes.
- Release build passed **209.635s/1,524,576KiB**. Compiler/LSP all-target
  diagnostics passed **53.426s/1,372,432KiB**, only existing decode_plain
  lib-test dead-code warning. Private emitter/runtime change preserves editor
  facts; preceding50 editor passes remain applicable.
- Candidate allocation profile **2026-11-02T01:47:44.234Z**,
  **3.120s/321,184KiB**. Three identical samples/source hashes and zero live bytes
  after both drops. Interfaces **14,337→10,238 allocations** (4,099 fewer,
  28.6%), reallocations9 unchanged, requested **1,072,736→994,855 bytes**,
  retained **821,248→743,367**, peak **821,267→743,386**. Exactly77,881 bytes
  saved =4,099 eliminated19-byte integer buffers. Numeric record_mutation's
  complete sample dictionary is unchanged, all allocation counters zero.
- Quiet candidate **2026-11-02T01:48:14.054Z**, **11.923s/353,104KiB**,
  max timed load **2.182**,no override. Original93samples/engine/workload.
  All six affected native medians improved; numeric/enum controls remain flat:

  | Workload | Parent ms | Candidate ms | Native speed | Node ms, parent→candidate |
  | --- | ---: | ---: | ---: | --- |
  | interfaces | .243333 | .203083 | 1.198× | .117292→.118000 |
  | type_aliases | .148458 | .123333 | 1.204× | .038875→.039791 |
  | nullable_int_null | .033000 | .022708 | 1.453× | .026833→.027375 |
  | array_splice_int | .321041 | .221209 | 1.451× | .217583→.216667 |
  | labeled_control | .366500 | .286084 | 1.281× | .272167→.273750 |
  | declaration_lists | .314875 | .275458 | 1.143× | .076750→.081500 |
  | record_mutation | .098666 | .098667 | flat | .431375→.431167 |
  | enum_values | .066833 | .066375 | flat | .036458→.035917 |

  Interfaces native process medians .202333/.207333/.199875; normalized
  improvement1.205×. Other affected normalized improvements1.214–1.483×.
  Declaration Node drifted+6.2%, so distinguish its raw1.143× from normalized
  1.214×. All affected native process medians improve over parent global medians.
  Controls normalized .9995/.992× are within noise. No measured regression in
  selected cases and no semantic failure; do not claim whole-matrix timing.
- Paired manifest,drivers,oracle,adapters,harness,lock,versions and workload hashes
  match. No external native dependencies required by these cases; dependency
  records match. Compiler input difference is exactly emit/mod.rs and runtime/
  strings.rs. store:ingest added **16 candidate histories** (plus8 extra parent
  controls earlier); exact median6/mean rate3/93counts, all current source hashes
  and SQLite integrity verified. Reused Rust baselines; no Rust retiming.
- Public selection uses accepted candidate pairs. Whole-project formatting zero
  files/snippets/fingerprints changed. Three website tests,66-page build,
  links/assets in116pages pass; website **3.645s/988,304KiB**. Current compiler/
  LSP release consumers include accepted code. No CI,push or deployment.
- Pruned **105,105,032 bytes** across25 closed raw/probe/obsolete test outputs
  after import verification. Kept the latest candidate timing/allocation profile
  as the next parent, with summaries above for closed profiles. Current accepted
  digit test executables remain; older schema/editor test artifacts will rebuild.
  See latest checkpoint for accepted source and measurements.
  Next: profile the remaining whole-number String(number) conversion in enum_values
  before considering the same character-storage idea. Preserve negative zero,
  nonfinite/fractional spelling and exact native semantics. The104-byte output-row
  layout remains a measured memory target, with prior inline-array wins protected.


**Object allocation profiling — parent baseline complete, 2026-11-02:**
- Accepted parent **2f3d0fca**. Release compiler/LSP rebuilt successfully,
  **210.971s/1,492,320KiB**. No object optimization/source change yet.
- Native allocation profile **2026-11-02T01:18:34.930Z**,3.639s/221,888KiB;
  pinned Rust reference profiles **2026-11-02T01:18:51.364Z**,1.041s/250,784KiB.
  Same existing allocation driver/ABI adapters/independent oracle; three exact
  repeated samples per engine/case. The reference source hashes match public
  pinned baselines. No new profiling dependency or Rust timing baseline.
- interfaces n2048,seed7,input19 UTF-8 bytes,checksum **473727250**:
  generated **14,337 allocations/9 reallocations**, requested **1,072,736**,
  retained **821,248**, peak821,267; reference **16,385/2,057**, requested
  **634,784**, retained **505,856**, peak505,876. Generated has fewer calls but
  uses more bytes. Every output/input drop returns to zero live bytes.
- String-free record_mutation n200000,checksum **10,000,700,000**: both engines
  **zero allocations/reallocations/requested/retained/peak bytes**.
- Matched layout attribution reuses those probes and asserts retained bytes equal
  outer slots+inner vectors+owned string capacities. Three identical samples;
  all original counts/checksums preserved. Generated row slots **104 bytes**,
  outer212,992 /inner245,760 /strings362,496. Reference slots **24 bytes**,
  outer49,152 /inner245,760 /strings210,944. Extra retained315,392 bytes splits
  exactly into **163,840 row-layout +151,552 string-capacity bytes**. Layout
  diagnostic1.037s/165,936KiB; /tmp/tsrust-object-layout.json.
- System-allocator CPU probes retain original workload bodies and black-box each
  returned output before drop; observer validates checksum once before20,000
  repeated batches. CPU samples show generated allocation/manual integer append,
  reference std formatting/reallocation. Diagnostic wall7.306/10.539s are **not
  matched benchmark timings** and must not update performance histories. Combined
  watchdog18.066s/33,424KiB; /tmp/tsrust-interfaces-{native,hand}.sample.
- Fresh quiet parent **2026-11-02T01:23:48.576Z** completed,
  **8.315s/283,232KiB**, max timed load **2.067**, no override. Original
  3processes/eight warmups/31samples each, **93 per engine/workload**. Native /
  Node medians: interfaces **.243333/.117292ms**, record_mutation
  **.098666/.431375**, type_aliases **.148458/.038875**, enum_values
  **.066833/.036458**. Interface native process medians .241917/.246250/.245417.
  Saved Rust medians remain **.413167/.098667/.178917/.179166** respectively;
  no Rust wall-clock baseline rerun. The old public .755ms interfaces figure was
  stale; this fresh current-compiler baseline is not an empty-schema speedup.
- store:ingest added **eight histories**. Exact median6/mean rate3/93 counts,
  every compiler/workload source hash, quiet policy and SQLite integrity verified.
  Public pin now selects these four current pairs, reusing all other measurements
  and pinned Rust references. Three website tests,66-page build,links/assets in
  116pages pass; website **3.640s/933,584KiB**. Whole-project formatter zero
  files/snippets/fingerprints changed. Source unchanged since **2f3d0fca**.
- Keep the raw parent timing/allocation profiles for the coming A/B. The matched
  hand/layout/CPU scripts in /tmp reuse existing observers/drivers, with exact
  hashes, accounting assertions and zero-live-byte checks; they are disposable
  diagnostics, not new production tools or dependencies.
- Next: trace standalone String(int) emission and existing static ASCII-character
  storage before choosing a change. Interfaces retains two per-row small numeric
  output strings (score and optional y). Reusing the existing one-character table
  for single decimal digits is a candidate to remove their allocations while
  preserving independent mutation through current Cow ownership. This is an
  **unimplemented hypothesis**, not a chosen new ownership policy or measured
  gain. Verify minimum/maximum/negative integers, formatting, subsequent mutation
  and native String conversion; profile first and preserve controls. Do not
  assume replacing the already-buffered digit writer with format! is faster.


**Object and string allocation — next optimization plan (owner follow-up):**

Owner reminder, 2026-11-02: retain the useful suggestions from the supplied
feedback; they are profiling hypotheses, not evidence that arenas or a formatter
replacement will help. Reuse the existing tools and measurements before adding
dependencies. Rust-default ownership remains the default.

1. **Separate construction/access from string work.** Reuse string-free
   `record_mutation` as the existing mutation control. Check existing coverage
   before adding a numeric record construction/access workload. If needed, add
   matched Tsrust/TypeScript/standard Rust sources, an independent oracle and
   observable outputs; distinguish temporary records from records that escape.
   A win on one numeric case does not prove all struct code optimal, or establish
   that strings explain every other gap. Inspect optimized work when interpreting
   a benchmark, rather than attributing its title to one operation.
2. **Compare actual generated code with the reference.** Ordinary records already
   use typed structs/direct fields, not dynamic hash maps. Regenerate interfaces
   Rust from the accepted compiler; compare with
   [the Rust reference](../benchmarks/runtime/parity/interfaces.rs). The reference
   uses std `format!()` in describe; generated describe already reserves one
   destination buffer and appends text/manual decimal digits. At the original
   profiling baseline both copied an owned input name per iteration; accepted
   c21ea2ce now borrows that nonescaping generated field. Generated code borrows
   literal "p" while the reference allocates it. Thus cloning alone was not a demonstrated explanation
   of the previously reported .413/.755ms gap. Neither version is assumed optimal.
3. **Profile matched memory and CPU work first.** Measure allocation/reallocation
   counts, requested bytes, peak live bytes and cleanup for generated/handwritten
   Rust with matching inputs, iterations and output observation. Use existing
   allocation counters and CPU sampling first; consider dhat/coz only if a missing
   attribution capability justifies another tool. Rust allocator calls and V8
   managed allocations are different units; do not equate their raw counts.
   Inspect repeated small-number string conversions, input copies, formatting,
   output growth and drop costs. Another concrete layout lead: five-string rows
   take Array<String>'s heap arm, but each wrapper reserves its four-element
   inline payload. Measure its cost and preserve accepted small-array wins.
4. **Remove proven unnecessary work while retaining Rust ownership.** Evaluate
   borrowing/moving nonescaping fields, eliminating intermediate strings/arrays,
   and direct writes into an existing destination where the output contract
   permits. The interfaces workload retains every result string and row: one
   reusable scratch buffer cannot replace those independently owned outputs.
   Compare std formatting with the existing digit helper only after profiling;
   include `itoa`/`ryu` only if number formatting is a measured residual cost,
   checking existing dependencies first and preserving exact integer/float text.
   No formatter dependency, arena or storage rewrite is preapproved by this plan.
5. **Treat arenas as a conditional design question.** Structs already have inline
   storage; placing a String/Vec header in an arena does not redirect its backing
   buffer. Arena-aware containers, escaping values, reset boundaries, destructors
   and async/thread lifetimes need explicit handling. Bumpalo's ordinary arena
   allocations do not automatically run Drop. Preserve Rust-default ownership
   and timely resource cleanup; raise an owner design decision only if measured
   residual allocation cost justifies this architectural change. Do not treat
   bump allocation as zero total cost or replace the default allocator speculatively.
6. **Apply the existing acceptance loop.** Establish fresh quiet parent timings,
   change one measured cause, validate affected native/editor suites under the
   watchdog and A/B against the accepted checkpoint with focused Node controls.
   Reuse the existing Rust reference measurement unless its source/protocol must
   change. Keep only measured performance gains without regressions, ingest into
   measurements.sqlite, format, checkpoint and record rejected trials honestly.

Status: numeric construction/access controls and matched allocation profiles are
complete (see the accepted/rejected sections above). Borrowed local string fields
are accepted in c21ea2ce; exact integer sizing and a global five-slot Array were
rejected after measurement. Fixed-width owned result rows and readonly small slices are accepted in the
previous compiler checkpoint;closed readonly small parameter slices are accepted
in the latest optimization checkpoint (1.066×,repeat1.046×);see measured sections above.
The subsequent global single-digit append branch and three formatter/wrapper
variants were rejected:interfaces1.153× did not offset string-iteration.978× and
iterable-call-spread.970× regressions. The original writer remains. Text-only
assignment-concatenation reuse is now accepted:2.307× (repeat2.288×),allocations
-86.2%,retained heap unchanged;see current section above. Numeric/mixed bindings
keep the original fused path after measured regressions;future short/long numeric
builder work remains separate. Reprofile residual
string costs before extending storage hints or changing formatting. Destination reuse
is limited to values whose ownership permits it; escaping strings remain distinct
owned outputs. Arena allocation and new formatting dependencies remain conditional,
unapproved design options, not scheduled implementations. The old .413/.755ms
comparison is historical, not a current optimization target.

Residual-string follow-up is now accepted above: safe stack character views remove
8,192 transient allocations and improve Unicode iteration1.171× (repeat1.147×)
without the repeated ASCII regression of the rejected source-slice prototype.
Retained/peak heap stays unchanged;owned copies,calls,captures and unknown uses
retain ordinary storage. Profile residual escaping output and clone costs before
extending this proof;no arena,cache,RC or public lifetime change is approved.


**Source JSON retention — correctness repair accepted, 2026-11-02:**
- Parent runtime **df3ca7f0**; benchmark support checkpoint **b611fecf**. Fixed
  source::Node::retain once, replacing direct Serde Value decoding with existing
  parse_plain. Ordinary `$serde_json::private::RawValue` properties previously
  replaced objects with scalars or panicked for non-JSON string contents. The
  shared fix covers ordinary/pipeline retained extras and dynamic JSON callback
  input. Public APIs, Rust defaults, dependencies and generated unsafe unchanged.
- New owned source-node regression covers bounded/plain documents, private-key
  payloads "0", "true", "not JSON", nested objects and input drop. Public native
  regression checks all three callers, exact retained payload/count/Map size.
  The first witness correctly failed ownership checking after capturing/reusing
  expected; fixed the fixture with a fresh input literal, without weakening rules.
- Added **native_schema_json_source_retain** without changing old DOM-retention
  workload semantics. safeParseJson retains/serializes nested extra data and
  observes all missing-count issue fields. Matched TS and independent standard
  Serde/Rust corpus-specific references; n=2000, seed=7, two operations/iteration,
  checksum **seed+5*n**. New oracle, guards, actual reference ABI and report
  registration; **39 support tests pass**. Untimed verification passed all three
  engines/five correctness cases, checksum10007, **23.853 s/367,456 KiB**.
- One-time parent **2026-11-01T23:16:59.787Z**: Tsrust **1.735208 ms**, Node
  **1.574083 ms**, hand-written Rust **1.212250 ms**. Original three processes,
  eight warmups,31 samples/process, **93/engine**; max load3.503, no override,
  **86.257 s/349,120 KiB**. Native process medians2.064250/1.694833/1.735208
  show first-process drift. Rust source hash
  **aa744dd4a3acd91b8c04e0313aad7fca343f30595e9852e26d938a608f92c306**.
  Public page now has **933 comparisons**; reuse this Rust baseline.
- Profiled actual retention before implementation: 4M iterations, checksum
  **20,000,007**, diagnostic3784.293ms; source field indexing/materialization/
  serialization visible. First repair diagnostic3903.210ms. These CPU samples
  are profiling evidence, not comparative benchmark claims.
- Final retained code is byte-identical to first correctness candidate's parser/
  source hashes, verified against measured compilerInputs after rejecting two
  refinements. Its **126 affected invocations passed**: decoder9 (eight first
  pass+one repaired-fixture retry), pipeline20, recursive10, plain parser3,
  scalar backend19, JSON library65. Shared backend cases appear in more than
  one binary; not126 distinct features. Initial build455.359s/1,654,064KiB;
  suite watchdogs17.094+7.753 retry/51.245/22.799/7.781/.524/1.052s,
  max native-suite peak1,077,840KiB. Shared target, incremental off, one build
  job, individual5GB/30min watchdogs, three test threads; no resource stop.
- Correctness candidate **2026-11-01T23:43:03.346Z**, **82.871s/414,160KiB**,
  max load3.845, original93samples and identical dependency objects:
  **Tsrust1.735208→1.761291ms (+1.503%)**, Node1.574083→1.567875ms
  (-.394%), normalized .981×. **This is a mandatory accuracy fix, not an
  accepted speed optimization.** Public selection now uses this measured fix.
- Parent and all three candidates' allocation dictionaries match exactly:
  **18 allocations/iteration**,0reallocations,5,344,000 requested bytes/2000
  iterations, peak1,523, checksum10007, zero live/retained bytes after drop.
- Rejected specialization skipping already-validated final document checks:
  **2026-11-02T00:11:25.347Z**, retention1.804708ms,Node1.579125ms,
  normalized .965×. Scalar .089209ms vs .086833 control; string .073500
  vs .073334. Max timed load3.944, no override,183.961s/374,608KiB.
  All126 affected tests passed (build453.102s/1,929,568KiB).
- Rejected further removal of unused float tracking from trusted retention:
  **2026-11-02T00:29:57.191Z**, retention1.760125ms,Node1.568041ms,
  normalized .982×. Scalar .091542ms/Node.141875; string .079042/Node.143000,
  versus pre-refactor .086833/.134708 and .073334/.147208. Max timed load3.817,
  no override,148.715s/436,240KiB. All126 affected invocations passed,
  build473.136s/1,699,296KiB; suite watchdogs22.666/48.931/22.166/7.209/.518/
  1.043s, max1,089,312KiB. Both compile-time options and trial-only numeric
  assertions removed; existing public parser restored byte-for-byte.
- Original parent/first repair, pre-refactor parser-control baseline
  **2026-11-01T23:51:24.252Z**, and both rejected trial histories ingested with
  store:ingest. Exact median6/mean rate3/93counts, source hashes, matching native
  dependency objects and SQLite integrity verified. No full Node/Rust reruns.
  Rejected trial timestamps remain excluded from public pins.
- Whole-project formatting passed with **zero** file/snippet/fingerprint changes.
  Three website tests,66-page build,links/assets in116pages pass;
  website watchdog **3.657s/927,200KiB**. No LSP signature/checker changes;
  previous complete editor acceptance remains valid. No CI/deployment/push.
- Final compiler/LSP release rebuild passed **212.995s/1,634,000KiB**.
  Original native reproducer now prints the complete private-key object rather
  than scalar0; smoke passed **7.737s/356,176KiB**. Pruned **128,093,218 bytes**
  across35 closed raw/profile/native/stale-test outputs after verified imports.
  Selected test executables will rebuild next time; current release consumers
  remain. Local correctness checkpoint is ready; no speed acceptance claimed.
  Next independent confirmed defect:
  `s.object({}).retain()` rejects "Unknown generic record type"; plain empty
  object schemas already have a diagnostic-options witness. Investigate/fix
  retained empty output without weakening generic-record validation.

**Plain JSON visitor — accepted locally, 2026-11-01:**
- Parent **a50ff90a**, initially clean. Fresh actual recursive CPU sample used four
  million iterations, seed 7, checksum **16,000,007**, diagnostic consumer time
  **5,874.634 ms**, watchdog **6.194 s/46,800 KiB**. parse_plain/holds_float are
  still visible after slot removal work; no comparative claim from CPU sampling.
- Found a real defect: parse_plain checks floats in the final DOM, so a later
  duplicate field can erase evidence of an over-range integer. Accepted release
  reproducer /tmp/tsrust-duplicate-range.tsr printed **ok / ok / ok** for positive
  overflow overwritten by 0, negative overflow overwritten by null, and overflow
  inside an overwritten array. Native watchdog **8.812 s/349,536 KiB**. Bounded,
  typed/source validation already track the original tokens and do not have this
  final-tree gap. Syntax/trailing-data precedence must stay intact.
- Reuse the existing strict DOM visitor with
  a const bounded/unbounded switch; the plain path tracks saw_float while decoding
  and keeps Serde's default recursion guard. This removes holds_float, preserves
  plain Rust limits, and validates overflow even in discarded duplicate content.
  Scope: shared JSON parser, native backend/public JSON.parse regression witnesses,
  affected JSON/schema suites and focused existing workload timing support.
  Public API/editor signatures stay unchanged; no new limits/dependencies/unsafe.
- Fresh eight-workload parent timing completed **2026-11-01T22:00:45.183Z**,
  watchdog **150.694 s/390,096 KiB**. Original three processes × 31 samples after
  eight warmups, no busy override, max load **3.752**. store:ingest added **16**
  parent histories; exact medians/mean rates/93 counts, both source hashes and
  SQLite integrity verified. Raw result remains for the candidate comparison.
  Includes previous five schema controls plus native_json_roundtrip/scalars/
  string_reads, with focused Node controls. No unrelated Node/Rust reruns.
- Candidate parser.rs now reuses Seed/KeySeed with const BOUNDED. Plain decoding
  disables explicit budget bookkeeping, tracks every f64 and rechecks integer
  tokens after syntax/end validation. Bounded behavior retains its existing guards.
  Removed holds_float; no second DOM walk. Added backend witnesses for five
  overwritten-value/escaped-key cases with exact offsets, syntax/trailing error
  precedence, scalar/string/duplicate equivalence and 0..130 nesting comparisons
  against Serde. Extended the existing public JSON.parse native regression.
  The backend also preserves ordinary keys matching Serde internal tokens.
- Confirmed another plain-path defect: with Serde's raw_value feature, a first
  property `$serde_json::private::RawValue` is interpreted as a private wrapper.
  Accepted release maps the valid objects with string payloads "0" and "true"
  to scalar outputs **0 / true**, losing the object/property. Corrected native
  probe passed **25.303 s/273,424 KiB**. Initial probe's second nested-string
  fixture had an escaping error (first valid case still printed 0); corrected
  it before relying on the two-case result. No resource stop.
  New visitor treats ordinary property names consistently with the bounded path.
  Added four backend cases for numeric/boolean/non-JSON string contents and key
  position. Source inspected locally in Serde Value visit_map/KeyClassifier.
  Don't claim equivalence to this unintended private-key interpretation.
- Compiler test build passed **921.177 s/2,076,944 KiB**. Long library phase
  was confirmed in LLVM ThinLTO imports; no resource stop or duplicate restart.
  Existing decode_plain dead-code warning only. All selected artifacts built;
  --list confirms the late-added private-key backend test is registered (19
  scalar-backend tests including shared runtime witnesses), so no rebuild needed.
- All **152 test invocations passed across nine binaries**: json_parse_plain 3,
  scalar backend 19, container backend 15, json_typed 8, json_namespace 9, json 4,
  schema_recursive 10, schema_pipelines 20, JSON library 64. This includes shared
  runtime witnesses compiled in multiple binaries; don't call them distinct
  feature tests. New overflow/private-key/0..130 nesting checks passed. All
  catalog examples executed through the native CLI and matched documented output.
  Highest suite peak **1,156,448 KiB**; one binary at a time, three threads,
  5 GB/30-minute watchdog each. Full suite log /tmp/tsrust-json-visitor-tests.log.
  JSON/catalog **294.152 s/1,074,400 KiB**, recursive **23.378 s/1,077,632 KiB**,
  pipelines **50.428 s/1,022,064 KiB**, library **1.057 s**. No resource stops.
- Candidate public JSON.parse probe now preserves both private-key objects,
  **10.452 s/313,968 KiB**. Initial schema-retention probe with s.object({}) hit
  an independent empty-shape generic-record checker limitation (.526 s/3,680 KiB),
  not a runtime result. Revised it to the existing supported named-field shape;
  native probe confirmed scalar **0**, **9.924 s/311,712 KiB**. This remaining
  caller defect is separate from the now-passing plain-parser regressions.
- LSP build passed **189.482 s/1,367,120 KiB**; all **16 schema_editing tests
  passed**, **.519 s**. Total **168 passing test invocations** across the selected
  compiler/LSP binaries. No full-regression or CI claim for this helper change.
- Release build passed **217.023 s/1,567,616 KiB**. Allocation profile
  **2026-11-01T22:37:25.434Z** passed **81.302 s/359,520 KiB**. All three
  samples/case identical, correct checksums, zero reallocations/retained/live
  bytes after drop. Recursive remains **40 allocations/iteration**, requested
  **8,344,000 bytes/2,000 iterations**, peak **1,898** (unchanged from a50ff90a).
  Candidate roundtrip/scalars/string_reads: **6 / 1 / 1 allocations/iteration**,
  requested bytes **1,790,000 / 10,000 / 20,000**, peaks **895 / 5 / 10**. No
  fresh parent allocation profiles for those three; don't claim measured equality.
- First quiet candidate completed **2026-11-01T22:42:34.270Z**,
  **299.322 s/367,536 KiB**, max timed load **3.9995**, no busy override. The
  guard waited for preparation load to fall; no timing accepted while busy.
  All eight dependency evidence objects exactly match the parent. All emitted/
  instrumented source hashes change due to the shared parser, so these are not
  byte-identical-code controls. Original five correctness cases/93 samples each.
- First candidate native medians parent→candidate (ms): recursive
  **2.803542→2.611834 (1.073×)**, roundtrip **.630333→.579250**, scalars
  **.097875→.088167**, string reads **.081500→.085542**, union
  **.347708→.334667**, discriminator **1.086084→.991208**, pipeline
  **.710792→.689166**, collections **1.911500→1.873791**. Their Node movements:
  +1.710%, -3.995%, -.664%, -5.957%, -3.662%, -1.458%, -2.460%, -1.321%.
  **Not accepted:** string reads regress **4.959%**, normalized speed .896×.
  Parent string process medians **.080875/.081334/.083458**, candidate
  **.085541/.084292/.086625**: every candidate slower than every parent.
- store:ingest added **16 diagnostic histories**, **32** total with the parent.
  Exact medians/mean rates/93 counts, both source hashes and SQLite integrity
  verified. Public pins unchanged; first-candidate date is excluded. Raw results
  remain for refined comparison; no acceptance claim for mixed results.
- Profiled that regression before refining: current string workload ran 120M
  iterations, checksum **1,800,000,007**, diagnostic **5,022.727 ms**, watchdog
  **5.201 s/46,416 KiB**. Sample /tmp/tsrust-json-string.sample shows generic
  Seed::deserialize under parse_plain; this is not comparative timing.
- Refinement: string roots use Serde's direct String deserialize; they contain no
  numeric tokens/object keys requiring the shared float flag. Composite/numeric
  input keeps the strict visitor. Syntax/end checks and error offsets still use
  the original decoder/input. Added root-string integer-text/whitespace/escaped
  Unicode/backslash parity plus invalid escape, trailing-data and invalid leading
  whitespace checks. No thresholds, limits, dependencies or unsafe added.
- Focused refinement build passed **200.975 s/1,505,632 KiB**. Both affected
  binaries passed sequentially: json_parse_plain **3**, scalar backend **19**,
  watchdogs **8.816 / .522 s**, peaks **768,768 / 32 sampled KiB**. New root-string
  whitespace/Unicode/escape/error cases passed alongside duplicate overflow,
  literal private keys and default Serde nesting checks. These 22 are reruns,
  not new distinct feature counts. Prior 168 passes remain for unchanged bounded,
  catalog, composite/runtime and editor behavior; no heavy matrix restart.
- Refined release build passed **215.657 s/1,467,840 KiB**. String allocation
  profile **2026-11-01T22:54:47.588Z**, **22.296 s/340,032 KiB**, has three
  identical samples: checksum 30007, 2,000 allocations/2,000 iterations,
  requested 20,000 bytes, peak 10, zero reallocations/retained/live bytes after
  drop. Exactly equal sample metrics to the first candidate; root routing adds
  no allocation or retained source storage.
- Refined quiet candidate **2026-11-01T23:00:34.692Z** completed,
  **316.958 s/400,752 KiB**, max timed load **3.703**, no busy override. Original
  five correctness cases/three processes × 31 samples after eight warmups; all
  16 workload/engine groups retain **93 samples**. All eight dependency evidence
  objects exactly match the parent, including manifest/lock/config/rlib byte hashes.
  Shared parser changes emitted/instrumented source hashes in all eight; don't
  label these byte-identical-code controls. All timing sources have no unsafe.
- **Accepted:** all eight native medians and every per-process native median
  beat their corresponding parent medians. Recursive **2.803542→2.647250 ms**,
  **1.059×**, Node **1.430375→1.388416** (-2.933%), normalized **1.028×**.
  Parent native process medians **2.873833 / 2.793667 / 2.790667**, refined
  **2.711750 / 2.623292 / 2.659000**. String reads **.081500→.073542 ms**,
  **1.108×**, Node **.155292→.143416** (-7.648%), normalized **1.023×**.
  Refined string medians **.074792/.073500/.071417**, each below every parent.
  This resolves the first candidate's measured regression while fixing both
  plain-parser correctness defects. Allocation/ownership behavior is preserved.
- Other native medians (ms): roundtrip **.630333→.581625**, scalars
  **.097875→.088625**, union **.347708→.316833**, discriminator
  **1.086084→.949958**, pipeline **.710792→.670625**, collections
  **1.911500→1.888875**. Corresponding Node changes -5.699%, -1.878%, +2.080%,
  -3.363%, -4.873%, -1.217%; normalized gains **1.022 / 1.084 / 1.120 / 1.105 /
  1.008 / 1.000×**. Collection movement tracks Node; don't attribute every raw
  gain to the parser or claim universal hand-written parity. Only focused Node
  controls reran; all hand-written Rust/unrelated Node baselines are reused.
- store:ingest added **16 refined histories**, **48 total** across this section's
  parent/diagnostic/refined runs. Exact medians/mean rates/93 counts, both source
  hashes and SQLite integrity verified. Public selection includes only the
  refined candidate; diagnostic date excluded. Whole-project formatting changed
  zero files/snippets/fingerprints. Three website tests, 66-page build and links/
  assets in 116 pages passed, website watchdog **3.666 s/975,008 KiB**. No new
  full-regression, CI or deployment claim.
- Follow-up caller review: source::Node::retain still directly uses
  serde_json::from_str. Callers are emit/schema/decode.rs, decode_pipelines.rs
  (unknown retained subtrees) and decode_input.rs (JSON callback inputs). Its
  private-key behavior is now confirmed by the named-field native probe above.
  Repair this caller in the next coherent correctness
  section using the fixed shared parser and add a source-retention witness.
  **Benchmark correction:** existing native_schema_json_object_retain uses
  JSON.parse+safeParse, not safeParseJson/Node::retain. It is a context/control
  workload, not a measurement of that caller. Before timing the repair, add a
  matched source-retention workload (Tsrust safeParseJson, TypeScript counterpart,
  independent oracle, source guards/ABI support and one-time hand-written Rust
  reference), preserving the old workload identity and baselines. Do not claim
  the current plain-parser fix already covers source-retained schema values.
- Closed raw timing/allocation records, CPU probes, first-candidate allocation
  consumers and obsolete test binaries were pruned. Current focused tests and
  refined timing consumers remain in the shared Cargo target. All owned runners
  are terminal. The unresolved retention fixture remains at
  /tmp/tsrust-json-retained-private-key.tsr for the next correctness section.
  Also investigate the independent empty-object schema checker rejection noted
  above; don't silently classify it as intended behavior.

**JSON object slots — accepted locally, 2026-11-01:**
- Parent **7e0b9a8c**, clean accepted source. Fresh system-allocator CPU sample
  ran the actual recursive workload, four million iterations, seed 7, checksum
  **16,000,007**. Consumer **6,397.237 ms** (diagnostic, not comparative timing);
  build/sample watchdog **28.483 s/392,848 KiB**. BTreeMap remove_entry and its
  remove_leaf_kv/memmove descendants are visible under recursive object trials.
- Preserve keys/tree slots in stripping/rejecting JSON object trials,
  move values out using Value::take, and replace them on rollback. Retaining
  schemas keep removal so extras remain exact. No change to plain prepared JSON
  projection yet. Scope: pipeline object emitter, focused rollback/default/null
  witness, affected schema suites and existing five-workload timing support.
  No public API/editor/catalog change, new limits/dependencies or generated unsafe.
- Fresh parent execution-only baseline plus focused Node controls completed:
  **2026-11-01T21:30:44.494Z**, **115.587 s/365,520 KiB**, max load **3.831**,
  no busy override. Original three processes, 31 samples, eight warmups.
- Candidate changes pipeline_object_trials.rs only: stripping/rejecting JSON
  objects use get_mut/Value::take, retain existing key storage and replace each
  present raw value on rollback. Retain keeps remove_entry/insert. Added a public
  std:testing witness in schema_pipelines.rs for real null versus temporary null
  slots, absent default removal, supplied value presence and unchanged extras,
  through both JSON.parse+safeParse and safeParseJson. Existing partial-field
  rollback/commit/error-path suites also passed.
- Affected compiler test build passed **451.243 s/2,112,000 KiB**. All **158**
  compiler checks passed: pipelines 20 (including new null/default witness),
  recursive 10, collections 5, unions 13, diagnostic options 11, super-refine 5,
  schema library 94. One binary at a time, three threads, one Cargo job, shared
  target, incremental off; highest suite peak **1,083,824 KiB**. Exact suite
  watchdogs in /tmp/tsrust-schema-slots-tests.log. Existing decode_plain dead-code
  warning only; no generated lifetime/type errors or resource stops.
- LSP build passed **180.124 s/1,831,568 KiB**; all **16 schema_editing tests
  passed**, watchdog **.521 s**. Total **174 unique affected checks passed**.
  Release build passed **212.065 s/1,738,592 KiB**.
- Candidate allocation profile **2026-11-01T21:51:05.174Z** passed,
  **61.667 s/345,728 KiB**. Recursive stays **40 allocations/iteration**,
  requested bytes over 2,000 iterations **8,408,000→8,344,000**, peak requested
  **1,898** unchanged. Pipeline **10** and collections **33** allocations/iteration;
  requested/peak bytes unchanged. Three identical samples each, exact checksums,
  zero reallocations, retained bytes and live bytes after output/input drop.
- Quiet A/B candidate **2026-11-01T21:52:54.801Z**, **102.079 s/367,920 KiB**,
  max load **3.911**, no busy override. Both runs retain original five correctness
  cases and **93 samples per engine/workload**. All five dependency evidence
  objects, including manifest/lock/config and rlib byte hashes, are exactly equal.
- Recursive Tsrust median **2.937209→2.825958 ms (1.039×)**. Node
  **1.423000→1.440625 ms (+1.239%)**, normalized speedup **1.052×**. Parent
  process medians **2.894125 / 2.973917 / 2.932125**, candidate
  **2.800708 / 2.833666 / 2.844125 ms**: every candidate beats every parent.
  Accepted for measured end-to-end improvement with unchanged behavior/peak.
- All four controls retain byte-identical emitted/instrumented source and paired
  dependencies. Native medians (ms): union **.350709→.351084**, discriminator
  **1.063542→1.058584**, scalar pipeline **.681917→.715250**, collections
  **1.922041→1.948125**. Their Node changes are +.587%, +2.947%, +4.617%, +.129%.
  Pipeline's roughly 4.9% native drift accompanies the 4.6% Node drift with the
  exact same compiled inputs; normalized movement is effectively flat (.997×).
  Do not claim all medians improved. Only focused Node controls reran; existing
  hand-written Rust and unrelated Node baselines are reused.
- store:ingest added **20 histories**. Exact median/mean-rate/93-sample counts,
  both benchmark source hashes and SQLite integrity verified before deleting raw
  records. Public selection uses the accepted candidate only. Whole-project
  formatting changed zero files/snippets/fingerprints. Three website tests,
  66-page build and links/assets in 116 pages passed. Website watchdog
  **3.106 s/1,158,304 KiB**. Emitted recursive source has four slot extractions,
  zero remove_entry calls and no unsafe blocks; allocation instrumentation is
  confined to its diagnostic driver. No full-regression/CI/deployment claim.
- Closed raw timing/allocation results, diagnostic consumers and temporary CPU
  probes were pruned; current accepted timing/test consumers remain in the shared
  target. All owned runners are terminal. Next: fresh profile remaining advanced
  JSON materialization or prepared projection costs. Do not repeat rejected
  inverse-storage rewrites without new evidence. This gain does not establish
  general hand-written Rust parity or fuse advanced parsing and validation.

**Typed object-field restoration — rejected locally, 2026-11-01:**
- Parent **f307d518**. Fresh recursive CPU sample ran 4M iterations, checksum
  **16,000,007**, watchdog **6.194 s/46,848 KiB**. Object trial, sequence,
  BTreeMap removal and allocator work remain visible. Profile first
  **2026-11-01T20:46:41.665Z**, **60.673 s/350,688 KiB**: recursive/pipeline/
  collections **40 / 10 / 33 allocations per iteration**.
- Tried typed Present/Absent field inverses plus prepared-only map_restoration,
  retaining the enclosing object's existing erased inverse. Exact missing/default/
  present identity, callback commitment and ownership were preserved; no API,
  limits, dependencies or generated unsafe. Changes in trial.rs,
  pipeline_object_trials.rs and the added schema_pipelines witness are now
  restored exactly to f307d518. **The candidate is not retained or published.**
- The new unit test initially named nonexistent Issues::default; corrected it to
  the existing constructor. Stopped only the owned doomed build (160.843 s,
  1,598,448 KiB), not a resource watchdog stop. Actual runtime modules compiled
  **1.045 s/197,824 KiB**, **32 tests passed**. Full test build then passed
  **462.706 s/1,947,728 KiB**. All **175 unique affected tests passed**: pipelines
  20, recursive 10, collections 5, unions 13, diagnostic options 11, super-refine
  5, schema library 95, editor 16. The isolated 32 are a subset, not extra passes.
  Native std:testing witness verified dropping synthetic defaults and preserving
  supplied values during readonly union fallback. No generated type/lifetime
  errors. One binary at a time, three threads, one job, shared target, incremental
  off; executable peaks at most **1,110,752 KiB**.
- Candidate release build **211.525 s/1,756,512 KiB**. Allocation profile
  **2026-11-01T21:13:58.110Z**, **61.099 s/405,360 KiB**: recursive **40→35**
  allocations/iteration, requested bytes over 2,000 iterations
  **8,408,000→8,248,000**, peak requested **1,898→1,866**. Pipeline and collection
  samples exactly unchanged; all three samples/case match checksums, have zero
  reallocations and zero live bytes after drop. Fewer allocations alone do not
  establish an end-to-end speedup.
- Quiet A/B, parent **2026-11-01T20:50:29.774Z**, candidate
  **2026-11-01T21:16:22.617Z**. Paired dependency evidence is exactly equal,
  including both rlib byte hashes. Original five correctness cases, three
  processes × 31 samples after eight warmups, 93 samples/engine/workload, system
  allocator. Recursive median **2.866334→2.869375 ms** (+.106%, effectively flat);
  Node **1.399500→1.476584 ms** (+5.508%). Node-normalized movement suggests
  1.054× but does not establish the required absolute runtime improvement.
  Parent native process medians **2.824666 / 2.893916 / 2.866334 ms**; candidate
  **2.874959 / 2.856875 / 2.951375 ms**, overlapping rather than consistently
  faster. Rejected under the keep-only-faster/no-regression rule.
- Native controls parent→candidate (ms): union **.343709→.354042**,
  discriminator **.997958→1.071834**, scalar pipeline **.690583→.705084**,
  collections **1.865459→1.951584**. Their Node movements were +1.559%, +4.426%,
  +1.928%, +3.285%. Runtime helper additions change emitted source in all five
  programs; do not label these as byte-identical-code controls. Max loads
  **3.464 / 3.465**, no busy override. Watchdogs **114.607 / 98.942 s**, peaks
  **441,440 / 417,232 KiB**. No hand-written Rust baselines reran.
- store:ingest added **20 histories**. Exact medians/mean rates/counts, both
  source hashes and SQLite integrity verified. Public pins remain unchanged;
  rejected timing is not selected. Closed raw profiles/results and rejected native,
  allocation and test executables were pruned after import verification.
- Accepted source is restored. Release rebuild completed **210.171 s**,
  peak **1,811,552 KiB**. Accepted debug CLI/LSP rebuild completed
  **196.297 s/1,692,800 KiB**. All runners are terminal. Rejected executables
  and temporary probes were removed; subsequent work uses the restored tools.
  Whole-project formatting changed zero files/snippets/fingerprints. Regenerating
  the public performance page after ingestion produced identical bytes; both
  focused readability/highlight tests passed. No new broad regression or CI run.
- Next: profile JSON materialization and BTreeMap extraction/restoration before
  another inverse-representation change. Keep f307d518's measured Map/Set gain.

**Identity-preserving Map/Set schemas — accepted locally, 2026-11-01:**
- Parent **8da8e64d**, implementation checkpoint **32cdfb23**. Plain matching
  string/int/number/boolean Map/Set schemas borrow existing scalar preparation
  checks and return the owned input unchanged. No entry/output/rollback vectors
  or collision scan are needed when validation cannot change any entry. Pipeline,
  coercion, wrapper and scalar-choice schemas keep their existing trials. Bounds,
  finite-number checks, first-error order, ownership and union fallback remain.
  No new limits, unsafe, dependencies, public signatures or built-ins.
- Scope: collection emitter, shared temporary-field checker, standalone editor
  constructor recovery, compiler/editor witnesses, benchmark dependency setup,
  minimal measurements, public performance selection and this handoff. Existing
  docs/catalog signatures remain correct. New std:testing witness verifies bounds,
  key/value/index paths, deletion/reinsertion order, readonly fallback, NaN,
  boolean/empty collections and temporary size reads. Temporary size writes stay
  rejected by the existing unsupported-storage rule.
- Found/fixed a real defect during validation: `.parse(input).size` rejected
  temporary Map/Set outputs when the pre-check receiver hint was unavailable.
  The shared field checker now uses the resolved receiver type. Standalone and
  header editor recovery lacked Map/Set schema constructor output inference;
  extended the existing array helper. Checked project facts were already correct
  and were not changed. Hover/completion now agree in standalone, header and
  checked-project tests. Initial new-witness failure and later negative-diagnostic
  expectation mismatch were repaired; passing tests were retained.
- **168 unique affected checks passed**: collections 5, pipelines 19, recursive
  10, unions 13, diagnostic options 11, schema-library 94, schema editor 16.
  Binaries ran one at a time, --test-threads=3, shared Cargo target, incremental
  disabled, one job. Highest executable-suite peak **1,159,312 KiB**. Debug build
  after the field fix **463.179 s/1,983,648 KiB**; filtered negative retry
  **1.039 s/2,784 KiB**. One unchanged test-only decode_plain dead-code warning;
  no unrelated warning suppression. No full-regression/CI claim for this change.
- Profile first: **2026-11-01T19:35:23.933Z**, recursive/pipeline/collections
  **40 / 10 / 43 allocations per iteration**. Fresh collections CPU sample ran
  5M iterations/checksum **40,000,007**, **6.703 s/32,432 KiB**, showing sequence,
  allocator and transform work. Candidate profile **2026-11-01T20:14:23.592Z**:
  collections **43→33**, requested bytes over 2,000 iterations
  **7,242,000→6,042,000**. Peak requested bytes **649** unchanged; pipeline and
  recursive samples exactly unchanged. All three samples/case checked oracle
  checksums, zero reallocations and zero live bytes after drop. Profile watchdogs
  **44.615 / 43.087 s**, peaks **1,151,280 / 900,288 KiB**.
- Initial timing suggested 1.260× but a byte-identical discriminator control
  drifted. Found measurement defect: random exported source directories changed
  Serde rlib identities between runs. The dependency helper now uses an exclusive
  stable scratch path, still removed in finally, and restores one Cargo job after
  environment sanitization. Real offline dependency tests prove repeated exports
  have identical library paths/hashes and preserve occupied directories on EEXIST.
  Final five tests passed without skips, **43.037 s/383,408 KiB**, including the
  final one-job setting; the initial run also passed (34.244 s/722,512 KiB).
- Kept **22 diagnostic histories** from **2026-11-01T19:40:21.459Z**,
  **20:16:43.106Z** and **20:18:20.785Z**, excluded from public selection. Rebuilt
  the parent emitter from 8da8e64d after checkpointing, verified all five emitted
  and instrumented hashes match that initial parent exactly, then restored the
  candidate emitter from 32cdfb23. No source snapshots. Controlled parent/candidate
  builds **210.820 / 211.869 s**, peaks **1,807,680 / 1,811,488 KiB**.
- Controlled A/B: parent **2026-11-01T20:29:50.102Z**, candidate
  **2026-11-01T20:35:23.593Z**. Original three processes × 31 samples after eight
  warmups, five correctness cases, system allocator. All ten workload/engine
  groups retain 93 samples each. Paired manifest/lock/config and both dependency
  libraries are byte-identical for every workload: serde
  **0658f7c40d32d58e412ece6be7ed32de61fed46da6df06fbebfbc0efaf3184bf**,
  serde_json **ce7bb19427156bb765beee10ddc3f306e158864b9c74c2dfd3cddb52f3dfee2a**.
- Collection medians: Tsrust **2.491750→1.953750 ms (1.275×)**; Node
  **.869458→.884875 ms (+1.773%)**, normalized speedup **1.298×**. Parent process
  medians **2.488167 / 2.475375 / 2.505583**, candidate
  **1.987875 / 1.933666 / 1.951333 ms**; every candidate median beats every parent.
  Max loads **3.108 / 2.797**, no busy override; watchdogs **101.182 / 100.641 s**,
  peaks **379,024 / 375,200 KiB**. Only these focused Node controls reran; all
  hand-written Rust and other Node references reused.
- Four controls retain byte-identical emitted/instrumented sources and paired
  dependencies. Native parent→candidate medians (ms): recursive
  **2.967917→2.964750**, union **.349833→.357958**, discriminator
  **1.060041→1.082208**, pipeline **.723667→.704708**. Their roughly ±3% timing
  variation is present with identical compiled inputs; no control code changed.
  Do not claim all control medians improved or universal hand-written parity.
- store:ingest added **20 controlled histories**, alongside the 22 diagnostics.
  Exact medians/mean rates/93-sample counts, both source hashes and SQLite
  integrity verified before disposable raw/probe cleanup. Local public selection
  uses only the controlled candidate; published v0.1.1 remains unchanged.
  Whole-project formatting changed zero files/snippets/fingerprints. Three
  website tests, the 66-page build and links/assets in 116 pages passed. Generated
  collection benchmark has no unsafe blocks/FFI. Removed closed raw results,
  allocation consumers, probes and obsolete schema test binaries; current tested
  executables remain in the shared target. All active runners are terminal.
- Next: fresh CPU/allocation profile of remaining advanced JSON composite/choice
  restoration using stable dependency identities. Reassess old rejected trials
  only from new evidence; record/recursive JSON costs are not solved here.

**Current owner request — microbenchmarks first, stronger CPU example:**
- Follow-up: the Mandelbrot HTTP throughput gap was too modest for the headline.
  Selected the existing **local_closures** microbenchmark: **15.94× Node throughput
  for Tsrust**, **16.20× for hand-written Rust**. Shows the exact workload function
  inline with its 200,000-iteration/seed-7 input and three-engine results.
  Source/counterpart hashes are pinned in performance-runs.json; source changes
  fail generation instead of silently keeping old highlight numbers. This is
  explicitly a selected callback microbenchmark, with overall results/losers kept.
- All eight full language/library benchmark groups now precede **HTTP benchmarks**.
  HTTP retains all existing rendering, p99, CPU/RSS and I/O observations; also
  exposes the actual rendering function and Output type in a collapsed example.
- Exact displayed function re-executed at n=200000/seed=7: Tsrust, TypeScript and
  hand-written Rust all returned **20001700007**, agreeing with an independent
  BigInt oracle. Untimed check under watchdog: **3.098 s**, peak **257,568 KiB**.
  No timing reruns or new measurement histories were needed. Website **three
  tests pass**, including exact-code/selected-ratio and section-order guards;
  66 pages build and 116-page links/assets pass. Desktop screenshot verified
  section order, compact 3-column comparison and full displayed function.
- Previously removed all visible date columns, combined
  batch-time cells and redundant ratios. Secondary HTTP engine/req-s/p99/
  CPU-per-request/RSS is a five-column table, with responsive engine cards on
  narrow screens. Old I/O results remain in collapsed one/16-connection
  four-column tables. All 932 language/library comparisons remain visible in
  compact groups; timestamps/source pins stay in repository metadata.
- New independently authored Tsrust/Node/Rust handlers render a 96×64 Mandelbrot
  tile (up to 256 iterations/pixel), based on varying IDs 1..128. Independent
  Python complex-number oracle runs outside servers/timing. One process/one
  executor worker per engine; Tokio worker count fixed to one. No hidden
  multicore advantage, no unsafe in generated source, no new compiler dependency.
- Original three-process/eight-warmup/31-sample protocol, 128 requests/batch.
  Each engine/case has **93 batch samples and 11,904 actual request latencies**.
  Separate warmed resource pass completes another **11,904 requests** per case:
  macOS ps CPU time (10 ms resolution), peak sampled RSS (50 ms polling).
  Individual request p50/p99 never derive from batch durations. Resource polling
  does not overlap throughput/latency sampling. RSS includes runtime/shared pages,
  excludes the client; it is not heap usage or guaranteed high-water memory.
- Initial verify caught JSON field order only; canonicalized Node/Rust/oracle
  output without weakening byte checks. Corrected three-server live verify
  passed **32.676 s**, peak **453,216 KiB**, 260 requests per engine. Seven HTTP
  guards and **22 storage tests** pass. Minimal histories now add only aggregate
  request p50/p99/count and CPU-ms/request/resource count/peak RSS/sample count.
- Quiet run **2026-10-31T18:50:04.215Z** completed all six groups, **461.971 s**
  including initial quiet wait, peak **128,992 KiB**. System indexing initially
  raised load to 16; waited without bypassing the guard or stopping system tasks.
  Accepted maximum load **3.413**, busy override false. Nine processes each
  preflighted 260 requests; every timed/resource response fully checked.
- **One connection**, Tsrust / Node.js / hand-written Rust: mean req/s
  **916.465 / 844.806 / 919.633**; p99 ms **1.208792 / 1.286541 / 1.201542**;
  CPU ms/req **1.022345 / 1.117272 / 1.018145**; peak sampled RSS MiB
  **8.0 / 62.609375 / 3.703125**. Tsrust is near Rust throughput; sampled RSS
  is about 7.8× smaller than Node for this kernel, not a universal speed claim.
- **16 connections**, same one-worker budget: req/s
  **988.083 / 980.029 / 988.328**; p99 ms **17.451083 / 30.704500 / 19.408291**;
  CPU ms/req **1.012265 / 1.024866 / 1.011425**; RSS MiB
  **8.203125 / 65.0 / 3.921875**. Closed-loop same-machine latency includes
  queueing and is not a production SLA. No prior I/O throughput controls reran.
- store:ingest added **six histories**; exact medians, mean rates, p50/p99,
  request/resource populations, CPU/RSS summaries, source/oracle/protocol hashes
  and SQLite integrity verified before raw results/verify-file cleanup. Public
  pins are separate from the 932 library workloads; no inflated summary counts.
- Website builds 66 pages, two website tests pass, links/assets in 116 pages pass.
  Final visual checks passed: desktop metric table **632 px**, no overflow or
  date headers; at 390 px mobile, **358 px engine cards** have no horizontal
  overflow. Screenshots /tmp/tsrust-performance-cpu.jpg and the -mobile.jpg
  counterpart. Removed both obsolete 675 MiB exported vendor packages, keeping
  only reusable server executables/client output in the shared target. Existing preview
  127.0.0.1:4321 is user-owned; do not stop it. No later release/deployment has
  been requested; the live v0.1.1 page still has the previous layout/results.
- Next after finishing this checkpoint: resume profiling normalization/table
  lookups or advanced schema restoration. Published v0.1.1 remains unchanged.

**Owner-requested promotion — published/deployed, 2026-10-31:**
- Owner explicitly approved **publishing v0.1.1**. Compiler **0.1.1**, independent
  LSP/VSIX **0.2.1**; version 979b906a, notes 045efdb7. Staging PR #8 merged
  **fa2bd1a9** after CI **41219037206** and Documentation **41219037199** passed
  (standard subset, not full). Used staging's squash/linear-history method and
  authorized owner review bypass; all required checks passed, no policy changes.
- Promotion https://github.com/Forever-Frameworks-LLC/tsrust/pull/9 merged main
  **9c294876**. **Release 41226915583 passed** all four platform archive builds,
  distribution smokes and editor packaging. **Promote and publish 41229001147
  passed**, publishing https://github.com/Forever-Frameworks-LLC/tsrust/releases/tag/v0.1.1
  with four archives, VSIX and SHA256SUMS. **Documentation 41229051188 passed**
  build/deploy. Live public performance page returned HTTP 200 with three-way
  HTTP results and the frozen release snapshot's **930 Rust references**.
- Downloaded actual macOS ARM archive/VSIX/checksums and verified GitHub API
  digests, SHA256SUMS, compiler/LSP/VSIX versions and clean release manifests.
  Synthetic build source **45e5ae2f** differs from main's final merge SHA; both
  have exactly the source tree and parents pinned by promotion.json. Do not
  require identical commit IDs. Actual published pair smoke passed CLI scaffold/native run/shared-cache JSON,
  documented quickstart/testing/reporters, and LSP initialization/shutdown:
  **9.325 s**, peak **405,808 KiB**. Scratch verification package is disposable.
- Trusted publisher uses its pre-merge checkout's release notes. Added the already
  reviewed optimization intro to the published notes while preserving generated
  changelog text; assets/tags unchanged. Initial local verification-only attempts
  hit Python 3.9's missing tar extraction filter and then an incorrect identical
  merge-SHA assumption; corrected both before published-pair execution. No
  product/package failure was found by those attempts.
- Main/stag ancestry was reconciled against 9981279e/27ca587e in e0fd0169;
  retained minimal measurements, owner-directed archive removals and newer docs.
  Later local **0a68a4f5** completes **932/932** Rust baselines; the normalization
  candidate below is also later local work, outside the frozen v0.1.1 release.
  Do not claim those later local results were already deployed.

**Cached normalization classes — accepted locally, 2026-11-01:**
- Parent **ee108cdc**. Cache the eight-bit combining class alongside the 21-bit
  Unicode scalar in the existing u32 decomposition buffer. Ordering reads that
  class; composition/output mask the scalar bits. Composed starters need only
  their scalar for later composition. No wider tuple, extra vector, unsafe,
  dependency, public API/ownership/table-version change or new limit. Scope is
  the private emitted helper, existing native conformance harness, measurements,
  public pin and this handoff; all callers route through that helper. Signatures,
  catalogue, examples, exports and benchmark sources/oracles remain unchanged.
- Profile first, n=2048/seed=7: **87,625** decomposed scalars, **87,625** class
  computations during segment scanning, **11,410** in sort comparisons and
  **56,036** during composition: **155,071** total. Candidate probe verifies
  **87,625**, exactly one per scalar (**67,446** computations removed).
  Both probes compare complete outputs with their uninstrumented consumer,
  checksum **332162546**. Watchdogs **2.599 / 2.598 s**, peak sampled group RSS
  **174,320 / 177,920 KiB**. A (u32,u8) tuple would widen the scalar buffer;
  packing uses the existing unused high bits instead.
- Three allocation samples before **2026-11-01T19:21:34.717Z** and after
  **2026-11-01T19:30:39.963Z** are exactly equal: **47,105 allocations**,
  **15,349 reallocations**, **1,877,118 requested bytes**, **780,086 retained**,
  **780,104 peak requested**. Every checksum matches and live bytes after drop
  are zero. Watchdogs **3.624 / 3.622 s**, peak **282,976 / 356,320 KiB**.
  These are allocation-byte counts; process RSS is a separate sampled metric.
- Added a **4,096-mark** leading sequence with differing and equal classes to
  the existing native conformance harness; all four forms must preserve tied
  order while sorting distinct classes. Full **399,300** official conformance
  and **1,101,784** identity checks, previous **324** boundary assertions and
  ownership/error tests pass. Compiler **9 passed**, **3.647 s**, peak
  **503,600 KiB**; LSP hover/completion/signature/native catalogue **2 passed**,
  **1.037 s**, peak **132,368 KiB**. Built the two selected test binaries together
  (--no-run), then executed one at a time under watchdogs with --test-threads=3.
  Shared target, one build job, incremental disabled. Test build **206.045 s**,
  peak **1,507,472 KiB**; release build **214.706 s**, peak **1,519,360 KiB**.
- Original quiet A/B protocol retained: three fresh processes × 31 samples after
  eight warmups, all five correctness cases, n=2048/seed=7. Parent
  **2026-11-01T19:22:20.264Z**, Tsrust/Node medians **3.713417 / 1.377917 ms**;
  candidate **2026-11-01T19:31:04.850Z**, **3.571625 / 1.396208 ms**. Tsrust
  **1.040× faster**, Node duration **+1.327%**, normalized speedup **1.054×**.
  Parent process medians **3.712459 / 3.710042 / 3.715500 ms**; candidate
  **3.556750 / 3.566708 / 3.610792 ms**. Every candidate median beats every
  parent median. Max accepted loads **2.190 / 2.645**, no busy override;
  watchdog **3.639 / 4.141 s**, peak **292,976 / 303,520 KiB**. This is a modest
  improvement; no Rust parity claim. Only this affected Node control reran;
  all other Node and Rust baselines reused.
- store:ingest added **four histories**. Exact median/mean throughput, counts,
  both source hashes and SQLite integrity verified before raw/probe cleanup.
  Whole-project formatting changed zero files/snippets/fingerprints; three
  website tests, the 66-page build and links/assets in 116 pages passed.
  Generated benchmark consumer contains no unsafe. Removed closed allocation
  consumers and superseded intelligence test binaries, retaining the tested one.
  Local public performance pin selects this candidate; published v0.1.1 remains
  unchanged. Next: profile advanced schema restoration/record copying against
  the independent Rust reference; do not add more normalization complexity
  without a fresh profile showing a worthwhile remaining cost.

**Normalization table-bound fast paths — accepted locally, 2026-11-01:**
- Parent **23abf642**. Skip combining-class/decomposition binary searches for
  scalars below the corresponding pinned table's first entry. Bounds come from
  the tables, not hard-coded character ranges or heuristic limits. The default
  class is zero; these scalars have no decomposition. Five runtime lines; no new
  allocation, API, ownership, Unicode version, dependency or generated unsafe.
  Scope: shared emitted helper, existing conformance harness, measurement store,
  public result selection and this handoff. No caller/signature/catalog/export
  changes; benchmark sources and independent oracles remain unchanged.
- Profile first on the parent's actual emitted consumer, n=2048/seed=7:
  **155,071** class searches, **46,697** below the first CCC entry (U+0300);
  **108,840** decomposition searches, **28,956** below its first entry (U+00A0).
  Complete output matched the uninstrumented parent, checksum **332162546**.
  Watchdog **2.591 s**, peak sampled group RSS **175,664 KiB**. Candidate probe
  confirms **108,374 / 79,884** actual class/decomposition searches, exactly
  **75,653** searches removed. Complete candidate output also matched;
  **1.029 s**, peak sampled group RSS **18,896 KiB** (short probe; sampling is
  not a guaranteed high-water measurement). Diagnostic probes are disposable.
- Added **324** mixed-string boundary assertions to the existing native Rust
  conformance harness: U+0000..U+00A0 and U+02FF followed by out-of-order combining
  marks. They bypass the whole-string ASCII shortcut, verify canonical ordering,
  preserve identity below the decomposition boundary and check U+00A0's
  compatibility mapping. Compiler **9 passed**, including **399,300** official
  normalization checks and **1,101,784** identity checks. **198.227 s**, peak
  **1,532,304 KiB**. LSP hover/help/completion and executable catalogue examples
  **2 passed**, **188.325 s**, peak **1,475,680 KiB**. Tests one binary at a time,
  --test-threads=3, shared Cargo target, incremental disabled, one build job.
  Release compiler/LSP build **214.528 s**, peak **1,488,864 KiB**.
- Original quiet A/B protocol retained: three fresh processes × 31 samples after
  eight warmups, five correctness cases, n=2048/seed=7. Parent
  **2026-11-01T19:07:11.225Z**, Tsrust/Node medians **4.572000 / 1.382459 ms**;
  candidate **2026-11-01T19:18:15.430Z**, **3.892959 / 1.440792 ms**. Tsrust
  **1.174× faster**, Node duration **+4.220%**, normalized speedup **1.224×**.
  Parent process medians **4.256500 / 4.633875 / 4.636875 ms**; candidate
  **3.877833 / 3.901292 / 3.931041 ms**. Every candidate median beats every
  parent median. Max accepted loads **2.301 / 2.548**, no busy override;
  watchdog **4.168 / 4.670 s**, peak **297,056 / 302,480 KiB**. Only this affected
  Node control reran; all other controls and hand-written Rust baselines reused.
- store:ingest added **four histories**. Exact median/mean throughput, 93-sample
  counts, both workload source hashes and SQLite integrity verified before raw
  cleanup. Whole-project formatting changed zero files/snippets/fingerprints;
  three website tests, the 66-page build and links/assets in 116 HTML pages
  passed. Generated benchmark consumer contains no unsafe. Public local pin
  selects the candidate; live v0.1.1 is unchanged.
- Next: profile repeated class computation between ordering/composition, including
  the memory cost of retaining classes, or resume advanced schema restoration.
  Do not assume a cached-class representation is faster before measuring it.

**Normalization in-place composition — accepted locally, 2026-11-01:**
- Parent **07d40f08**. Composition now writes into the already decomposed/ordered
  scalar vector and truncates it, removing the second composition vector. The
  write position never exceeds the read position; starter indices refer only to
  the written prefix. No API, ownership, Unicode table, limits, dependency or
  generated-code unsafe changes. Affected scope: shared emitted helper, focused
  compiler regression, measurement store, public result selection and this note;
  editor signatures/catalog/exports and benchmark sources remain unchanged.
- Profile first on the accepted parent's actual emitted consumer, n=2048/seed=7:
  **155,071** combining-class lookups (**46,697** below the table's first code),
  **74,172** decomposition entries, **44,993** composition attempts and **9,216**
  composition-vector allocations for **56,036** scalar visits. Instrumented and
  uninstrumented complete outputs agreed, checksum **332162546**. Watchdog
  **1.034 s**, peak **184,960 KiB**. Initial diagnostic compilation put counters
  before Rust inner attributes; corrected the probe before collecting evidence.
- Existing three-sample allocation profiler **2026-11-01T19:03:41.674Z**:
  allocations **56,321 → 47,105**, reallocations unchanged **15,349**, requested
  bytes **2,101,262 → 1,877,118**. Retained/peak requested bytes unchanged
  **780,086 / 780,104**, all checksums correct and zero live bytes after drop.
  **2.594 s**, peak **306,528 KiB**. The removed 9,216 allocations match the probe.
- Compiler **9 passed**, including the new leading-mark/blocked-composition/
  repeated-contraction witness, **399,300** official normalization checks and
  **1,101,784** identity checks. **198.115 s**, peak **1,167,856 KiB**. LSP
  hover/help/completion and native catalogue examples **2 passed**, **185.353 s**,
  peak **1,443,040 KiB**. One binary at a time, --test-threads=3, shared Cargo
  target, incremental disabled. Release compiler build **182.114 s**, peak
  **1,411,968 KiB**; runner's default LSP build **33.065 s**, **612,736 KiB**.
- Original quiet A/B protocol: three fresh processes × 31 samples after eight
  warmups, five correctness cases, same n=2048/seed=7 workload. Parent
  **2026-11-01T18:52:36.852Z**, Tsrust/Node medians **4.413458 / 1.367916 ms**;
  candidate **2026-11-01T19:03:55.850Z**, **4.290250 / 1.402209 ms**. Tsrust
  **1.029× faster** (2.79% less time); Node **2.507% slower**, normalized speedup
  **1.055×**. This is a modest gain, not parity with Rust. Parent process medians
  **4.407917 / 4.434041 / 4.405792 ms**; candidate
  **4.276584 / 4.308500 / 4.234917 ms**. All candidate medians beat all parent
  medians. Accepted max loads **2.744 / 2.453**, no busy override. Watchdog
  **5.164 / 4.141 s**, peak **339,776 / 309,344 KiB**. Only this affected Node
  control reran; all other Node and Rust baselines reused.
- store:ingest added **four minimal histories**. Verified exact median/mean rate/
  93-sample counts and SQLite integrity before disposing results and probes.
  Whole-project formatter changed zero files/snippets/fingerprints; three website
  tests, the 66-page build and links/assets in 116 HTML pages passed.
  Public local performance pin selects the candidate; published v0.1.1 remains
  unchanged. Next: profile reducing repeated combining-class searches, including
  the provably empty range below the first pinned CCC entry, before another edit.

**Normalization scratch reuse — accepted locally, 2026-10-31:**
- Parent **0a68a4f5**; change is after the frozen/published v0.1.1 source. Reuses
  one growable decomposition scratch Vec per normalization call instead of
  allocating one for each input scalar. No new limits, unsafe, dependencies or
  public API changes; owned output/default behavior and Unicode 16 tables remain.
- Profile first: baseline **2026-10-31T18:09:59.001Z** versus candidate
  **2026-10-31T18:22:39.956Z**. Each three-sample profile returned the complete
  oracle checksum and zero live bytes after drop. Allocations **116,157 → 56,321**,
  reallocations **28,802 → 15,349**, requested bytes **2,383,822 → 2,101,262**;
  retained bytes **780,086** and peak requested bytes **780,104** unchanged.
  Watchdog peaks **287,600 / 238,928 KiB**. Removed closed diagnostic JSON after
  retaining these measurements here; no raw profiles imported into the store.
- Affected compiler **8 passed**, including **399,300 official Unicode
  conformance checks** and **1,101,784 identity checks** plus the new repeated
  mixed-decomposition witness. Compiler test run **197.859 s**, peak
  **1,337,216 KiB**. LSP hover/help/completion and native documentation examples
  **2 passed**, **184.321 s**, peak **1,482,848 KiB**. One binary at a time,
  three threads; release rebuild **211.095 s**, peak **1,545,776 KiB**.
- Quiet A/B, 93 samples/engine after eight warmups per process, all five result
  cases retained. Parent **2026-10-31T18:10:28.708Z**: Tsrust **5.793750 ms**,
  Node **1.366417 ms**. Candidate **2026-10-31T18:25:01.488Z**: Tsrust
  **4.402167 ms**, Node **1.369583 ms**. Tsrust **1.316× faster**; Node duration
  **+0.232%**, normalized speedup **1.319×**. Max loads **1.984 / 1.667**;
  watchdog peaks **227,600 / 231,760 KiB**, elapsed **5.716 / 4.154 s**, no busy
  override/stops/failures. Only this affected Node control reran; other controls
  and all hand-written Rust baselines reused.
- store:ingest added **four histories**. Exact medians, mean rates, counts, source
  hashes and SQLite integrity verified before deleting disposable results.
  Candidate selection added to the local performance page; published v0.1.1
  retains its earlier snapshot. Whole-project formatting changed zero files;
  the 66-page website build passed. Accepted checkpoint **2c538120**.
- Separate SQLite HTTP phase profile validated 4,096 responses per mode. Mean
  acquire/prepare/query microseconds: c1 **7.763 / 7.465 / 8.859**, c16
  **607.541 / 8.445 / 9.928**. Concurrent latency sums are not elapsed throughput.
  No SQL runtime change: metadata preparation enforces documented parameter
  count/type checks; skipping it would change behavior. Closed probes removed.
- Release-note provenance repair below is complete locally. Next: profile the
  remaining normalization table lookup/ordering costs or advanced schema
  restoration before selecting another candidate.

**Release-note provenance repair — validated locally, 2026-10-31:**
- Observed v0.1.1's trusted publisher reading its pre-merge checkout's older
  release notes. The current published notes were corrected without changing
  tag/assets or generated changelog. Future fresh publications now fetch notes
  from the exact validated merged commit through the GitHub contents API;
  executable orchestration still comes from trusted main. File/encoding checks
  fail closed. No extra checkout or execution of staged source is introduced.
- Extended the actual publication fixture with a fresh-release branch whose
  checkout deliberately contains stale notes. It verifies the pinned merge ref,
  exact validated note text, generated changelog flag and draft publication;
  existing partial-draft recovery/merge/artifact checks remain intact.
  **Nine release-policy tests passed**, under the watchdog. This fix is local,
  after the published release; no second release/tag or CI dispatch was created.
- Removed the verified temporary download/extraction directory after successful
  smoke. The release URL/checksums remain the source for consumers.

**Validation:**
- GitHub CI dispatched today ran only the default subset, because
  `gh workflow run CI --ref <branch>` without `-f full_regressions=true` skips the
  complete `cargo test --workspace` step. Do not call such a run "full".
  The repository is public, so Actions minutes are free, but the owner now
  prefers local runs (other projects stopped to free memory).
- Full local regression (one test binary at a time, memory-guarded) was stopped
  at the owner's request after 430 of 491 test files (all 399 compiler files
  plus 31 of 92 editor files; editor files after `completion_matrix`
  alphabetically were not run, nor the `--lib` unit tests at the end).
  Results: one real failure, fixed in `4bf84f84` (`primitive_storage` expected
  the removed 1 GiB inline-record limit). Environment-only failures when test
  binaries run outside cargo: `native_cache` (passes under `cargo test`),
  `native_dependency_bundle` and `shared_sources` (need `CARGO_TARGET_DIR`).
  `shared_sources::portable_export_remains_toolchain_free_and_rebuilds_after_relocation_offline`
  still fails locally with `CARGO_TARGET_DIR` set: "Portable sources are not
  prepared. Run `tsrust setup --portable --yes`". Likely machine setup (CI
  prepares portable sources); confirm before treating it as a defect.
- Validation resumed locally on `3a625efe`: run all 77 current LSP integration
  targets alphabetically after `completion_matrix`, then both crates' `--lib`
  targets. The earlier 31/92 and 61-remaining counts do not match the current
  file inventory, so target names determine coverage. Each Cargo invocation runs
  alone with `--test-threads=3`, `CARGO_INCREMENTAL=0`, one build job and
  `CARGO_TARGET_DIR` set to the repository target. A process-group watchdog polls
  RSS every 0.5 seconds and kills the group above 5,000,000 KiB or 1,800 seconds.
  Disposable current-run script/results: `/tmp/tsrust-validation-watchdog.py` and
  `/tmp/tsrust-validation-results.json`. All 77 resumed LSP integration targets
  passed: 798 tests, zero failures, three existing ignored tests. The catalog
  target passed in 757.76 seconds; peak RSS across the resumed targets was
  1,164,768 KiB (including the initial dependency rebuild). Compiler `--lib`
  passed 1,292 tests with eight existing ignores; LSP `--lib` passed 20 tests.
  Total resumed validation: 2,110 passed, zero failures, 11 existing ignores,
  no watchdog stops. Overall peak RSS was 1,611,840 KiB. Compiler library build
  reported the existing unused `decode_plain` warning in `stdlib/json/typed.rs`.
  This requested validation stage is complete; the accepted optimization and
  remaining work are recorded below.
  Remote validation is not needed for this local optimization stage.

**How to run tests safely (8 GB Mac):** a test once grew to 30 GB when a compiler
limit was removed (non-terminating instantiation). Always run test binaries
under a watchdog that kills the whole process group past a memory/time budget,
e.g. a script that starts the binary with `set -m`, sums RSS for its process
group with `ps -A -o pgid=,rss=` every 0.5 s, and `kill -9 -$pid` past ~5 GB or
30 min. Use `--test-threads=3`. Run each binary from its crate directory
(`crates/tsrust-compiler` or `crates/tsrust-lsp`) or under `cargo test`, and set
`CARGO_TARGET_DIR=$PWD/target`, or several tests fail for environmental reasons.
Compiler and editor crates share some test names (for example `api_project`):
pick binaries per crate, not by name alone. Use bash (zsh does not split
unquoted variables and mangles `$var:path`).

**How to measure:** time only on a quiet machine (the runner refuses above load
4). A/B by checking out the baseline commit, running
`node benchmarks/runtime/run.mjs --only=<id> --engines=native,node-tsc` per
workload, then the change; compare median sample durations with the Node.js
control (a control far from 1.0 means the machine moved). Add
`rust-hand` where `benchmarks/runtime/parity/<id>.rs` exists. Afterwards run
`npm run store:ingest` in `benchmarks/runtime` (adds runs to
`benchmarks/measurements.sqlite`); raw result files are disposable after that.
`npm run checkpoint` prints latest native/Node ratios. For allocations,
`node benchmarks/runtime/profile-native.mjs --only=<id>`; for per-statement
counts, wrap a copy of the generated program with a counting global allocator.
`sample <pid> 3` works (Developer Mode is enabled).

**Accepted Map snapshot optimization — 2026-10-30:**
- Baseline: validation checkpoint `6b8f62b1`. A three-second CPU sample of the
  depth-80 nesting probe identified repeated `contains_scoped_map_storage`
  walks beneath Map epoch snapshot collection. The six snapshot sites now use
  one helper that classifies each distinct type once per snapshot. Its cache
  cannot outlive record-layout changes; snapshot ordering remains `BTreeMap`.
- The new native witness exposed stale optional-selection union/payload metadata
  after Map entry narrowing. Retargeting now aligns the selected payload before
  selecting callback dispatch, fixing a generated `TsrCallbackError` versus
  `TsrStorageError` mismatch. Present/missing entries and capture release pass.
  Initial development failures were a helper `HashMap`/`BTreeMap` return-type
  mismatch and an invalid nullable assertion; both were corrected before acceptance.
- Final affected validation: **347 passed, zero failures** across compiler
  `scoped_record_composition` (204), `optional_chaining`,
  `borrowed_optional_fallback`, `exception_return_moves`, `maps`,
  `exception_handling`; LSP `maps`, `exception_handling`; compiler library
  `map_` filter (59). One binary at a time, three test threads, one Cargo job,
  incremental disabled, shared target, 5,000,000 KiB/1,800-second watchdog.
  Peak process-group RSS: 2,088,960 KiB. The earlier 203 passing scoped-record
  cases were retained, then that affected suite was rerun for the metadata fix.
- A/B protocol: existing `nesting-discovery/run.py` scoped-map-catch generator;
  fresh compiler library-probe processes on the ordinary test worker stack;
  two warmups and 11 samples per depth, load average at most 4 before each.
  Native check/emit medians (baseline → candidate): depth 32 **460.521 →
  180.750 ms (2.55×)**; 64 **3,189.947 → 818.285 ms (3.90×)**;
  80 **6,067.294 → 1,402.972 ms (4.32×)**. Node/TypeScript strict no-emit
  controls: **155.646 → 157.441**, **161.296 → 160.239**,
  **161.154 → 159.901 ms**, all within 1.2%. These are frontend costs with
  different language contracts, not runtime throughput measurements.
- `npm run store:ingest -- /tmp/tsrust-map-baseline-timings.json
  /tmp/tsrust-map-candidate-timings.json` added all 12 history entries to
  `benchmarks/measurements.sqlite`; integrity check returned `ok` and all six
  workload/engine rows contain both 11-sample runs. Engine names are
  `native-check-emit` and `node-tsc-check` to avoid conflating runtime data.
- Whole-project formatting with the validated CLI changed zero files, catalog
  snippets or benchmark fingerprints. The native witness uses the shared
  formatting round-trip helper, including idempotence and formatted execution.
- Next: profile the remaining cost of scoped Maps across catch regions before
  another optimization. Repeated subtree traversal and binding scans still
  exist; this checkpoint does **not** claim to eliminate cubic scaling.
  Existing input generator and `/tmp/tsrust-time-map.py` can reproduce the
  measurement. The temporary scripts/profile are disposable, not new handoffs.
  Cleanup removed 577 older duplicate test executables (8,673,106,648 bytes),
  4,490 completed-build intermediate objects (3,980,355,832 bytes), and 16 old
  native program cache entries (79,347,359 bytes). The newest executable per
  test source, current CLI/LSP binaries, latest native program, root locks and
  shared runtime/Cargo dependencies remain. Target size is 20 GB; no rebuild
  was started to refill caches. Ingested raw timing files were deleted.
  No push, CI dispatch, merge, release or dependency change was made.

**Accepted callable-boundary Map epochs — 2026-10-30:**
- Baseline commit `260d1781`, clean tree at start. A fresh depth-128 CPU sample
  identified repeated Map epoch planning beneath `finish_block`, including
  snapshot filtering/hashing and nested body replay. Native Map epoch planning
  now runs at callable boundaries or the existing post-check parameter-carrier
  replay. Child-block carrier joins and ownership checking remain unchanged.
  The depth-16 native witness also executes inside a nested callable, checking
  its separate lexical boundary and present/missing entries after catches.
- Affected validation: **347 passed, zero failures**, using the same nine-target
  selection and watchdog as the preceding checkpoint. Peak process-group RSS:
  2,007,600 KiB; no watchdog stops. Compiler library `map_` contributed 59 tests.
  Whole-project formatting with the validated CLI changed zero files, snippets
  or fingerprints. The expanded witness verifies formatting round trips and
  generated native execution through the existing helper.
- A/B protocol unchanged: two warmups, 11 samples per depth/engine, fresh probe
  processes, load at most 4 before each sample. Timing initially refused while
  Spotlight indexed build outputs; measurements began only after load fell.
  Native check/emit medians (baseline to candidate): depth 32 **170.383 to
  65.181 ms (2.61×)**; 64 **784.422 to 170.969 ms (4.59×)**; 80 **1,375.769
  to 250.132 ms (5.50×)**; 128 **4,631.181 to 563.738 ms (8.22×)**.
  Node/TypeScript controls: **153.102 to 154.681**, **156.083 to 157.235**,
  **160.085 to 159.522**, **168.284 to 170.127 ms**, all within 1.1%.
  Fourfold depth now costs 8.65× instead of 27.18× in this probe; this measures
  the removed repeated epoch walk, not a formal bound on all compiler passes.
- `npm run store:ingest` added 16 run entries to `benchmarks/measurements.sqlite`.
  Integrity returned `ok`; all eight workload/engine rows contain both new
  11-sample runs. Engine names distinguish frontend costs from runtime data.
  Ingested raw timing files were deleted after verification.
- Cleanup removed 129 completed-build objects (1,118,442,264 bytes). No obsolete
  test or native executables remained. No push, CI dispatch, merge, release,
  dependency change, generated unsafe or limit change was made.
- Next: profile schema trial restoration allocations, ranked target 2. Map
  compile-time optimization is accepted; further compiler changes need a new
  profile and measured benefit rather than more speculative pass restructuring.

**Accepted typed schema restoration adapters — 2026-10-30:**
- Baseline `cea19c4c`. Fresh allocation profiling found 96,000 allocations for
  2,000 collection-workload iterations (48 each). `sequence`, `pair`, `inject`
  and `remap_input` boxed each intermediate restoration. They now retain typed
  restoration state; generated function and heterogeneous branch boundaries
  still erase types. No public signatures, ownership rules or limits changed.
- Initial native collection tests found an opaque-return lifetime leak in the
  candidate: `pair` retained the validator type and its temporary mutable path
  borrow. `PairRestoration<L,R>` now stores only actual rollback state. The
  new library witness composes all adapters, reuses callback-local state, then
  rejects a readonly refinement and verifies original payload pointers/values.
- Final affected validation: **209 passed, zero failures**: all 11 compiler
  schema integration targets (100), LSP schema editing (15), compiler library
  `stdlib::schema::` (94). One binary at a time, three test threads, one Cargo
  job, shared target, incremental disabled; 5,000,000 KiB/1,800-second watchdog.
  Peak RSS 2,016,624 KiB, no watchdog stops. Only the existing `decode_plain`
  unused warning appeared. Native fixtures retain formatting round trips.
- Candidate allocation profile: **86,000 allocations / 2,000 iterations = 43
  each**, down 10.4%; zero reallocations, correct checksum and zero live bytes
  after drop. The allocation profiler is diagnostic, separate from timing.
- A/B keeps the original protocol: eight warmups, 31 samples in each of three
  processes (93 samples per workload/engine), system allocator, quiet load <4.
  Initial baseline/candidate maximum load was 3.483/3.864. Native medians in ms:
  collections **2.502834 to 2.253125 (1.1108×)**, recursive 3.001375 to 2.958833
  (1.0144×), owned_array 0.411333 to 0.409334 (1.0049×), json_object 0.838459
  to 0.809334 (1.0360×), json_union 0.326541 to 0.331375 (0.9854×), pipeline
  0.678959 to 0.696208 (0.9752×). Collections' Node control improved 1.05%.
- Apparent small regressions were investigated, not discarded. Focused repeat
  recovered pipeline to 0.674875 ms (1.0061× baseline; Node within 0.1%).
  json_object repeated at 0.829792 ms; its Node control was 7.3% slower, so no
  precise speedup is claimed there. json_union repeated at 0.332334 ms, prompting
  a source checkpoint and fresh rebuild of `cea19c4c`, then the candidate.
  Paired json_union medians were **0.328250 / 0.322959 ms**, with Node
  **0.367583 / 0.371667 ms**; maximum loads 3.847/3.779. Its emitted source
  outside the unused trial module and all **286,144 executable text bytes**
  were byte-identical between commits; there are no external trial-module
  references in that workload. The apparent slowdown was measurement variation.
- `npm run store:ingest` preserved all **34** initial/repeat history entries,
  including slower samples. SQLite integrity is `ok`; each ingested entry has
  93 samples. Raw timing/profile files were deleted after verification.
  Whole-project formatting changed zero files, snippets or fingerprints.
- Cleanup removed 130 completed-build objects (1,127,130,016 bytes) and 22 old
  native runtime program-cache entries (2,293,088,437 bytes). Latest executables,
  root locks and shared Cargo/runtime build dependencies remain. Both current
  debug and release compilers were built from the accepted candidate source.
  No push, CI dispatch, merge, release, dependency change or generated unsafe.
- Next: profile mixed-type small arrays (`type_aliases`), ranked target 3.
  Further schema allocation work needs a new profile and measured benefit.

**Accepted inline scalar-union arrays — 2026-10-30:**
- Baseline `5083586a`, clean tree. Previous goal turn made progress by committing
  the accepted typed schema restoration optimization.
- Fresh `type_aliases` allocation profile: 12,289 allocations and nine
  reallocations for 2,048 iterations, with correct output/drop accounting.
  The workload's two-element `int|string` array uses Vec despite the existing
  safe four-slot Array representation used for homogeneous scalar arrays.
- Candidate adds immutable scalar-member metadata alongside UnionId's existing
  Copy property, including snapshot/reinterning paths. Rank-zero arrays of
  canonical scalar unions use the existing Array representation. Record,
  collection, callable, recursive and deeper array layouts keep indirection.
  Conservative aggregate-size accounting includes the scalar union's tag.
  No new runtime container, dependency, language limit or unsafe code.
- New witnesses cover snapshot identity and recursive exclusion, plus native
  clone independence, crossing the four-slot growth boundary, moving arrays
  through functions, shrinking/mutation, nested-array storage, enum and literal
  unions. Existing fixtures cover array methods, equality and ownership.
- Baseline runtime run `results/2026-10-31T03-16-17-317Z.json`: original eight
  warmups, 31 samples across three processes (93 per workload/engine); maximum
  load 2.361. Native/Node medians: type_aliases 0.147041/0.039000 ms;
  union_array_values 0.557500/0.294208 ms. Those raw profile/timing files were deleted after the ingestion recorded below.
- First witness used unsupported enum-union includes(); changed it to supported
  iteration. That exposed a real emitter defect: data-union text registrations
  referenced Enum types without emitting their TsrJoinText adapters when an
  unrelated array used join(). The emitter now registers both numeric and
  string enums through their existing scalar text behavior. The native witness
  covers both enum kinds; the enum acceptance suite joins affected validation.
- Validation completed via `/tmp/tsrust-union-array-validation.py`: 12 compiler
  integration targets (union arrays, aliases, nullable/data unions, recursive
  storage/schema, primitive storage, call/array spread), three LSP targets
  (type aliases, generic aliases, switch statements), then both full --lib
  suites because checked union identity is shared across the compiler.
  One binary at a time, three threads, one Cargo job, shared target,
  CARGO_INCREMENTAL=0, 5,000,000 KiB/1,800-second process-group watchdog.
  All 12 compiler integration targets passed (102 tests), all three LSP targets
  passed (7), compiler --lib passed 1,294 with eight existing ignores, and LSP
  --lib passed 20: **1,423 passed, zero failures, eight existing ignores**.
  Peak RSS 1,910,736 KiB, no watchdog stops. The library build reported only the
  existing unused decode_plain warning.
- Candidate profile: type_aliases **10,241 allocations** (2,048 fewer), nine
  reallocations. Initial timing: union_array_values **0.557500 → 0.510917 ms
  (1.091×)**, Node 0.294208 → 0.287833 ms. type_aliases **0.147041 →
  0.156500 ms**, Node 0.039000 → 0.041291 ms; normalized speed 0.995×.
  Candidate max load 2.359. This initial pair was insufficient for acceptance;
  the final paired comparison below determined the decision.
- Added a matched twenty-element union_array_heap control to cover spilling,
  heap iteration and array methods. It shares the existing protocol, independent
  arithmetic oracle and guarded sources. Both registration/oracle tests and
  untimed native/Node verification pass. No new runtime machinery.
- Final paired comparison rebuilt release compilers from commits: parent
  `393bceea` contains exactly the `5083586a` compiler plus the new benchmark;
  candidate `a1fd6a0d` contains the validated storage change and enum text fix.
  All three workloads used the original eight warmups, 31 samples × three
  processes (93 samples per engine), system allocator and exact-result oracles.
  Maximum measured loads were 1.976 / 3.500; the runner waited while candidate
  post-build load exceeded four. Watchdog peak was 1,862,784 KiB, no stops.
- Native baseline → candidate medians: union_array_values **0.597458 →
  0.488333 ms (1.223×)**; union_array_heap **1.239000 → 1.204375 ms
  (1.029×)**; type_aliases **0.155750 → 0.147333 ms (1.057×)**.
  Node controls respectively: **0.287000 → 0.290500**, **0.812541 →
  0.777583**, **0.041000 → 0.038750 ms**. Control-normalized speeds are
  1.238× / 0.984× / 0.999×. Keep the small-union improvement (also 1.091×
  in the initial pair); do not claim a reliable speedup for type_aliases or
  heap arrays. The latter's normalized 1.6% variation is within the control
  movement; neither repeat native median regressed. Allocation reduction is
  measured independently, and all affected correctness checks pass.
- `npm run store:ingest` added all **20** timing entries, including initial
  slower samples. SQLite integrity returned `ok`; each entry has 93 samples.
  Raw results/profiles were deleted after checking stored timestamps/counts.
  Whole-project formatting passed before both checkpoints: token-preserving
  formatting of the new benchmark, then zero changes on subsequent runs.
  The new benchmark passed native/Node exact-result verification and both
  registration/oracle tests. Compiler witnesses include formatted execution.
- Cleanup removed **204** completed-build objects (**1,168,179,056 bytes**).
  Each native program-cache root already retains only its latest executable;
  shared Cargo/runtime dependencies remain. Debug and release compilers both
  reflect the accepted candidate. No push, CI dispatch, merge or release.
- Next: profile single-pass schema JSON decoding, ranked target 4. Establish
  a fresh allocation/CPU profile before changing subtree cursor behavior.


**Accepted schema JSON parser-default repair — 2026-10-31:**
- Previous goal turn made progress: accepted scalar-union storage and persisted
  measured results in `610c2bc0`. Tree was clean before this section.
- Baseline `610c2bc0`: native allocation profiles report 26,000 allocations
  for native_schema_json_fused and 86,000 for native_schema_collections across
  2,000 iterations (13/43 per iteration), no reallocations or retained leaks.
- Fresh baseline timing used release/native programs built before source edits,
  three processes × 31 samples, eight warmups, Node-after-tsc controls, maximum
  measured load 3.706 (runner waited for load below four). Native/Node medians:
  fused 0.876209/0.673375 ms; ordinary JSON object 0.816417/0.671375 ms;
  collections 2.243541/0.813458 ms. Raw result:
  `results/2026-10-31T03-54-42-594Z.json`, pending paired timing/ingestion.
- Three-second CPU sample of baseline generated fused workload collected 2,132
  samples. Source field BTreeMap construction, RawValue scans, validation and
  allocation/free paths are visible. Existing field-index cursors reparse each
  subtree; callbacks require complete syntax validation, last-duplicate-wins
  resolution and schema field order. Any streaming replacement must preserve
  those behaviors rather than validating callbacks in source-field order.
- Found a concrete default-limit defect: schema safeParseJson still imposed
  16 MiB / depth 64 / one-million-entry/node caps, contrary to owner direction
  and ordinary JSON.parse's Rust defaults. Fix in progress: reuse the existing
  syntax validator with unbounded count budgets and Serde's recursion guard;
  avoid repeating UTF-8 validation for an existing str. Explicit bounded JSON
  entry points keep their existing behavior. This is a correctness repair,
  not a claim that the ranked single-pass decoder is implemented.
- Added native witnesses above the old byte/entry/node/depth caps, malformed
  input before callbacks, recursion agreement with serde_json, duplicate
  invalid content, and exact numbers. Updated safeParseJson IntelliSense.
  Existing issue/path retention budgets remain unchanged.
- Affected validation complete via `/tmp/tsrust-schema-defaults-validation.py`:
  JSON library tests 62, compiler schema_decode 7 / collections 4 / recursive 10
  / unions 13, LSP schema_editing 15: **111 passed, zero failures/ignores**.
  Shared target, incremental off, one job/target at a time, three test threads,
  5,000,000 KiB / 1,800-second watchdog. Peak group RSS 2,006,640 KiB; no stops.
  Catalog examples and native fixtures passed formatted execution.
- First candidate built/formatted successfully. Native baseline → candidate /
  repeat medians: fused 0.876209 → 0.860208 / 0.857625 ms; ordinary object
  0.816417 → 0.827459 / 0.796292 ms; collections 2.243541 → 2.286083 /
  2.284541 ms. Node baseline → candidate / repeat: fused 0.673375 →
  0.676667 / 0.665583; object 0.671375 → 0.712459 / 0.664208;
  collections 0.813458 → 0.806083 / 0.813666 ms. Max loads 3.706 / 3.610 /
  3.637. The repeated collections ~1.8% slowdown is not dismissed as noise.
- All 18 run entries were ingested (93 samples each). This correctness repair
  is checkpointed provisionally, not accepted as a performance improvement.
  Next: specialize the existing validation visitor at compile time so unlimited
  parsing does not maintain count/depth budgets. Recheck explicit bounded
  behavior and plain decoding, then repeat affected timings before acceptance.
  Single-pass schema decoding remains outstanding after this repair.
- Provisional source checkpoint is `11dc24cf`; its 1.8% collections slowdown
  remains unaccepted. Specialization checkpoint `4cf6c13c` uses one const-generic
  visitor with BOUNDED=true for explicit-limit APIs and false for schema's
  default path. False removes node/entry/depth counter maintenance and checks;
  syntax, float detection, exact integers and Serde recursion remain intact.
  Specialization validation passed via `/tmp/tsrust-schema-defaults-specialization.py`:
  62 JSON library tests and seven schema_decode tests, zero failures/ignores.
  Sequential watchdog peak 1,985,056 KiB; no stops. Other 42 already-passed
  schema/editor tests remain valid; no emitter/API changes since that complete
  111-test run. Specialized release build and whole-project formatting passed.
- Specialized timings: fused **0.839375 ms**, ordinary object **0.791625 ms**,
  collections **2.280334 ms**; Node controls 0.679834 / 0.719750 / 0.807166 ms.
  Maximum load 3.376, watchdog peak 1,671,168 KiB, 93 samples per row.
  Fused improves 2.2% over 11dc24cf (4.4% over 610c2bc0); collections remains
  about 1.6% behind the original baseline, despite being marginally faster than
  11dc24cf. All six specialized entries ingested, including this residual signal.
- Final rebuilt A/B: original `610c2bc0` versus specialized `4cf6c13c`.
  Fused **0.864458 → 0.832166 ms (1.039×)**; collections **2.311375 →
  2.281542 ms (1.013×)**. Node controls: fused 0.683500 → 0.708000 ms;
  collections 0.813042 → 0.810500 ms. Original baseline collections itself
  moved from 2.243541 to 2.311375 ms across rebuilds (3.0% variation), while
  specialized repeats stayed at 2.280334 / 2.281542 ms. The earlier residual
  slowdown did not reproduce in the fresh pair. Accept the parser-default fix
  and the specialization; claim only a small fused-decoding speed improvement.
- Unchanged ordinary JSON object control: **0.804375 → 0.824542 ms**; Node
  **0.672208 → 0.731834 ms**. Its native measurements ranged 0.791625–0.827459
  across runs, and this workload does not call safeParseJson. Control movement
  limits precise speed claims; it is not treated as an optimization result.
  Fresh measured loads max 3.715 / 3.298; watchdog peak 1,942,112 KiB, no stops.
- `npm run store:ingest` retained all **36** timing entries across six runs,
  including every slower observation. SQLite integrity `ok`; each entry has
  93 samples. Raw timings/allocation profile deleted after verification.
  Whole-project formatting passed with zero changes. Cleanup removed 110
  completed-build objects (**1,112,962,864 bytes**); native program caches
  already retain only their latest executable. Current debug/release compilers
  reflect the accepted specialization; requested branch restored, no remote work.
- Next: implement direct DeserializeSeed decoding for plain schema outputs,
  retaining the current decoder for unsupported plans and detailed failure
  reporting. Fast decoding must execute no callbacks; only accept after full
  input consumption and exact integer-token checks. Missing/unknown fields,
  constraints, duplicate replacement, invalid Unicode and nested invalid values
  must retain existing semantics through fallback. Profile larger nested cases
  too; this repair does not complete ranked target 4.



**Accepted plain-schema direct decoding — 2026-10-31:**
- Previous goal turn completed the parser-default repair in `a74da73f`; this
  turn began with a clean tree on the requested branch. Benchmark-only
  checkpoint `0a2443b8`
  adds a four-group/sixteen-integer nested JSON control and matching TypeScript
  implementation, independent arithmetic oracle, routing, guards and report
  category. All 36 schema benchmark tests pass. Whole-project formatting proved
  the new fixture's token preservation and idempotence before checkpointing.
- Fresh accepted-code profiles: fused 26,000 and collections 86,000 allocations
  across 2,000 iterations, zero reallocations or retained leaks. The nested
  profiling attempt exposed that profile-native.mjs always rebuilds Cargo: its
  internal 120-second build timed out on uncommitted candidate source before
  recording any profile or timing. Session 93445 is terminal. Its confirmed
  orphan rustc (PID 68916) exited before the termination request (no such
  process); no overlapping work is permitted.
  Retried correctly from `0a2443b8`, with an explicit release build under the
  external watchdog before invoking the profiler/runner. Baseline completed;
  requested branch restored. Nested profile: 26,000 allocations, no reallocations
  or retained leaks for 2,000 iterations. Timing file:
  `results/2026-10-31T04-59-04-347Z.json` (93 samples per workload/engine).
  Native/Node medians (ms): fused 0.825708/0.670500; nested
  6.566417/1.823125; ordinary object 0.816333/0.685959; collections
  2.290833/0.807750. Maximum load 3.772.
  Peak group RSS 1,458,992 KiB; no watchdog stops.
- Candidate emits callback-free DeserializeSeed visitors for plain scalar,
  array and object schemas. Typed output is built directly, without per-object
  BTreeMap indexes or repeated RawValue subtree scans. Fields overwrite their
  slots on duplicates; unknown/missing fields and unsupported plans fall back.
  No callback executes during the speculative attempt. Full consumption and
  exact integer-token checking precede accepting output. Pipelines, choices,
  recursive/retained outputs and detailed failures retain the existing decoder.
- New witnesses cover nested arrays, constraints, escaped duplicate keys,
  invalid earlier/final duplicates, malformed ignored content, large exact
  integers, source-specific diagnostics and callback order. Runtime helper tests
  cover trailing data, overflow, precision and negative zero. Generated output
  is checked for the direct visitor and absence of unsafe. No public API added.
- Candidate checkpoint `680ff0d0` plus the fixes below is accepted. Inspection of Serde's
  implementation found that IgnoredAny does not enforce its ordinary recursion
  guard. The candidate now uses CheckedIgnored, reusing our syntax visitor with
  default guard behavior, for unknown fields and unsupported runtime choices.
  Added unit/native witnesses for ignored excessive nesting and Unicode.
- Initial validation: all **64 JSON library tests passed**. schema_decode had
  seven passes and one failure: the new no-unsafe text assertion found existing
  JSON Lines unit-test FFI copied into emitted source (under cfg(test)), plus a
  comment containing the word unsafe. This was not direct-decoder unsafe code.
  JSON runtime emission now excludes that trailing test-only module; compiler
  unit tests remain intact. The witness checks actual emitted unsafe block/FFI
  spellings, allowing comments. No performance acceptance claim yet.
- Resumed validation passed: schema_decode 8, JSONL owned 6/borrowed 5/writer 5,
  collections 4, pipelines 18, recursive 10, unions 13, LSP schema_editing 15.
  Including the completed 64 JSON library tests: **148 passed, zero failures**.
  One target at a time under the 5 GB/30-minute watchdog, three test threads,
  shared target and incremental compilation disabled. Peak RSS 1,712,736 KiB;
  no watchdog stops. No passing suite was restarted.
- Release build and whole-project formatting passed; zero files, snippets or
  fingerprints changed. Exact-result checks passed for all four workloads.
  Allocation profiles over 2,000 iterations: fused **26,000 → 22,000**
  (13 → 11 each); nested **26,000 → 10,000** (13 → 5 each); collections
  unchanged at 86,000 (43 each), all with zero reallocations.
- Paired A/B against `0a2443b8`: eight warmups, 31 samples × three processes,
  93 samples per workload/engine, system allocator. Runner waited for post-build
  load below four; candidate maximum measured load 3.984. Watchdog peak
  1,545,856 KiB; no stops. Native baseline → candidate medians (ms):
  fused **0.825708 → 0.710917 (1.161×)**; nested **6.566417 → 2.103542
  (3.122×)**; ordinary object 0.816333 → 0.791583; collections
  2.290833 → 2.266125. Node controls respectively: 0.670500 → 0.739417,
  1.823125 → 1.845167, 0.685959 → 0.693417, 0.807750 → 0.828208.
  Fused Node moved 10.3% slower, so the exact fused speedup is less precise;
  its movement cannot explain the native gain. Nested Node moved only 1.2%.
  No native median regressed; no optimization claim for unchanged controls.
- `npm run store:ingest` added all **16** timing history entries. Verified
  SQLite integrity (`ok`) and all timestamps/sample counts (93 each), then
  deleted disposable raw timings/profiles. Cleanup removed 122 completed-build
  objects (**1,123,992,192 bytes**); both native program-cache roots already
  keep only their latest executable. Shared dependencies remain.
- Plain scalar/array/object decoding is accepted; advanced schemas still use
  the existing decoder. Next profile remaining advanced-schema cost before
  expanding this optimization. The later owner decision keeps Rust-default
  string ownership; see the accepted sizing pass below. No push, CI dispatch,
  merge or release.


**Current milestone validation — 2026-10-31 (complete):**
- Full local inventory reconciled: all 399 compiler integration files, all 91
  LSP integration files, and both complete library targets: **492 targets,
  5,321 passed, 19 ignored, zero unresolved failures, zero watchdog stops**.
  Compiler library: 1,297 passed/eight ignored; LSP library: 20 passed.
  Nine previously validated integration targets contributed 84 passes exactly
  once. Rerun targets replaced earlier results; partial failures were not
  double-counted. Peak process-group RSS was 1,888,848 KiB. Every target ran
  sequentially with three test threads, one build job, shared `target`,
  `CARGO_INCREMENTAL=0`, and a 5,000,000 KiB/1,800-second watchdog. No CI claim.
- Portable-export test initially lacked `TSRUST_SOURCE_PACK_DIR`. Its explicit
  local prepared source pack repaired the environment; the failed test passed
  on filtered retry, preserving five original passes. No product change.
- Retained UI ExampleCard diagnostics intermittently consumed an earlier empty
  publication. Isolated retry passed. Fixed the shared test wait to match URI
  and tracked document version, retaining unversioned closed-file publications.
  Added a deterministic queued stale/current-publication regression. All six
  worker tests and every suite using the affected wait passed; full UI: 31/31.
  This is test-support correctness work, with no runtime speed claim or timing
  store entry. Generated runtime implementation is unchanged since `a2f94dee`.
- Long gates passed: checker/editor agreement 515.18 s; completion matrix
  499.49 s; intelligence/native backend catalog 644.42 s; compiler library
  including build 325.86 s; LSP library including build 5.19 s.
- All runners are terminal. Disposable ledgers/logs remain under
  `/tmp/tsrust-schema-milestone-*`; final runner session 61290 exited zero.
  Whole-project formatting with current debug CLI passed: zero changed files,
  zero documentation snippets, zero fingerprint updates. Diff whitespace clean.
- Fresh three-second collections CPU sample before this milestone included
  substantial transform/trial/allocator work; field-index visitor work was
  smaller. Do not expand the advanced-schema decoder without an isolated
  representative profile. No new performance claim from that diagnostic sample.
- Owner resolved string storage: Rust defaults. Borrow slices where lifetimes
  permit; copy independently owned strings. No reference-counted backing or
  automatic copy/share size threshold. Known byte lengths may inform capacity;
  any extra sizing pass must prove an end-to-end benefit in fresh paired runs.
- Next: profile existing string workloads, starting with replacement first/all,
  plain/token/empty-pattern cases and slice/concat controls. Runtime concat,
  padding, addition and borrowed-to-owned append/reserve already allocate known
  byte capacity; repeat delegates to Rust after checked sizing. Token replacement
  starts at source byte capacity and may reallocate on expansion. Profile before
  changing it. Existing replacement histories contain 93 samples per engine;
  use fresh baseline/current comparisons, not historical times for acceptance.

**Accepted single token-replacement sizing — 2026-10-31:**
- Source checkpoint `ab83377b` is accepted after repeated parent/candidate timing
  against `1e4d2547`. `.replace()` computes checked byte capacity using the same
  token parser that writes the result. `.replaceAll()` retains one-pass growth;
  plain replacements retain Rust's standard operations. Independent owned
  strings still copy; no shared backing storage, unsafe code or new size limits.
- Profile first: over 2,000 iterations, first/token reallocations fell 1,012 ->
  12, first/empty-pattern 3,012 -> 12; allocations remain 2,003. Slice and concat
  controls unchanged. Three allocation samples each checked independent output
  checksums and zero live bytes after drop. Allocation profile peak 315,392 KiB.
- All 14 affected tests passed (string_construction, generic_default_builtins,
  string_concat_padding, string_substring), including native debug/release output,
  Unicode/empty/unknown tokens, ownership, operand evaluation and no generated
  unsafe. Sequential 5,000,000 KiB/1,800 s watchdog, three threads; peak
  1,602,912 KiB, no stops. Product behavior and IntelliSense unchanged.
- Fresh repeat A/B, 8 warmups and 31 samples x three processes per workload and
  engine (93 samples); quiet maxima 2.238 parent/2.292 candidate, limit four:

  | Workload | Native parent -> accepted ms | Node parent -> accepted ms |
  | --- | --- | --- |
  | first/plain | .067458 -> .069083 | .075292 -> .076584 |
  | first/tokens | .113916 -> .115042 | .175542 -> .173291 |
  | first/empty pattern | .158125 -> .110292 (**1.434x**) | .222584 -> .220042 |
  | all/plain | .075459 -> .074291 | .191500 -> .190292 |
  | all/tokens | .257250 -> .251833 | .617792 -> .619875 |
  | all/empty pattern | .391875 -> .379292 | 1.129583 -> 1.149291 |

  Target improvement is 30.25%, with Node control -1.14%. Other native medians
  range -3.21% to +2.41%; their process medians overlap and do not establish a
  repeatable regression. No speed claim for unchanged controls or first/tokens.
  Earlier single-only trial independently showed first/empty 22.39% faster.
- Rejected full pre-sizing: all/tokens .235500 -> .265375 ms (+12.69%), versus
  Node +5.59%, despite fewer reallocations. The extra scan was not worthwhile.
  Retained only the measured single-replacement improvement. Initial broad-run
  control drift prompted the fresh comparison above rather than early acceptance.
- All five runs (baseline, rejected full scan, initial single-only candidate,
  repeated parent and accepted candidate) ingested: 60 minimal workload/engine
  histories, 93 samples each; verified in SQLite with integrity check. Removed
  all five disposable timing files and three allocation profiles, 47 obsolete
  object files (345,268,704 bytes), and six older native build caches.
  Whole-project formatter: zero files/snippets/fingerprints changed.
- Repeat runner session 52120 exited zero; requested branch restored. Release
  rebuilds took 210.060/209.754 s, peaks 1,401,808/1,531,520 KiB; timed passes
  9.334/9.847 s. No active job; cleanup complete. This acceptance checkpoint
  closes ranked target five. Full milestone plus affected string suites are green.

**Accepted typed JSON pipeline restoration — 2026-10-31:**
- Baseline `1d3eb962`. Allocation/CPU profiles identified JSON pipeline leaf
  adapters boxing already typed `RestoreJson` state before callbacks. The main
  pipeline adapter appeared in 391/2,093 sampled stacks including allocator work.
  Uninstrumented 20M-iteration pipeline/recursive checksums verified 140,000,007
  and 80,000,007. Recursive sample also showed dynamic JSON/BTreeMap costs.
- JSON leaves now retain concrete restoration through existing `Trial::remap_input`
  and `impl FnOnce + use<'a>`. Refinements and supplied non-branching wrappers
  preserve that type; choices, recursion, presence alternatives, consuming
  stages and composite adapters retain their existing boxing. No new runtime
  machinery, unsafe, arbitrary limits, API or IntelliSense behavior change.
- New chained-refinement witness verifies exact rollback before a consuming
  fallback and successful reuse. All **70 affected tests passed**: pipelines 19,
  collections 4, recursive 10, unions 13, diagnostic/default options 11,
  super-refine 5, decode 8. Three threads, sequential targets, shared Cargo target,
  incremental disabled; 5,000,000 KiB/1,800 s watchdog. Peak 1,498,528 KiB,
  no stops. A nonexistent queue entry was corrected without rerunning passes.
- Three allocation samples over 2,000 iterations, checked checksums/no retained
  bytes: pipeline **24,000 -> 20,000** (12 -> 10 each), recursive **86,000 ->
  80,000** (43 -> 40 each); union 10,001, discriminator 30,001, collections
  86,000 unchanged. Zero reallocations throughout.
- Fresh quiet A/B against parent, 8 warmups and 31 samples x three processes,
  93 samples per workload/engine; max load 3.552 baseline/3.741 candidate:

  | Workload | Native parent -> accepted ms | Node parent -> accepted ms |
  | --- | --- | --- |
  | pipeline | .687542 -> .636250 (**1.081x**) | .341041 -> .362625 |
  | recursive | 2.944292 -> 2.840458 (**1.037x**) | 1.346416 -> 1.338250 |
  | collections | 2.318583 -> 2.285125 | .803125 -> .819959 |
  | discriminator | .980667 -> .965583 | .532209 -> .542417 |
  | union | .321000 -> .325959 | .362125 -> .366167 |

  Pipeline native improved 7.46% while its Node control slowed 6.33%; recursive
  improved 3.53% with Node -0.61%. Union's +1.54% tracks Node +1.12%, within
  noise; no material control regression. No speed claim for unchanged controls.
- All 20 entries ingested and verified (93 samples), SQLite integrity clean.
  Whole-project formatter: zero files/snippets/fingerprints changed; diff clean.
  Deleted two raw timings, two allocation profiles and 53 obsolete object files
  (351,826,064 bytes). Native caches already retain one latest build each.
  Release build 209.432 s, peak 1,510,944 KiB; profile 74.746 s; timing including
  preparation/quiet wait 178.313 s. Sessions 69994/42876/16299 are terminal.
- Next: reprofile remaining composite pipeline restoration boundaries before
  expanding opaque returns. Do not rerun the complete milestone for this routine
  accepted gate. Keep original rollback/consuming-callback/lifetime witnesses.

**Lazy sequence rollback storage — rejected, 2026-10-31:**
- CPU sampling accepted `28a3409c`: collections/recursive 20M iterations each,
  checksums 160,000,007/80,000,007; 53.765 s, peak 32,608 KiB. Sequence buffer
  allocation/free appeared in consuming collection paths.
- Tried starting `Trial::sequence` restoration Vec empty, reserving the original
  input length only on the first reversible element. This avoided unused rollback
  storage after first-element commitment, preserving Rust ownership and capacity
  on prepared paths. 70 affected integration tests + 95 schema library tests
  passed under sequential 5 GB/30 min watchdog, three threads. Peak test RSS
  1,630,176 KiB. Candidate and added witness have now been removed.
- Allocation profile: collections 43→41/iteration, recursive 40→39; pipeline
  10 unchanged; discriminator 30,001 and union 10,001/2,000 unchanged. No reallocs.
- First quiet five-workload run (93 samples/engine, max load 3.390) against
  accepted parent measurement 2026-10-31T09:00:23.547Z: collections
  2.285125→2.190500 ms (-4.14%, Node -2.29%); recursive -.49% (Node -2.48%);
  discriminator -.32%, union -1.07%; pipeline .636250→.653917 ms (+2.78%,
  Node -6.72%). Candidate timestamp 2026-10-31T09:23:22.784Z.
- Fresh rebuilt parent/candidate pair confirmed pipeline regression: pipeline
  .633334→.651750 ms (+2.91%) while Node .345458→.342333 (-.90%). Collections
  2.307125→2.265334 (-1.81%), Node .830458→.819250 (-1.35%). Same 93 samples,
  max loads 3.684/3.984, timestamps 2026-10-31T09:29:12.559Z and
  2026-10-31T09:34:40.161Z. Fewer allocations did not justify slower pipeline.
- Rejected under the no-regression rule. Source restored exactly to `28a3409c`.
  All runners terminal zero: 34359, 53107, 22746. `npm run store:ingest` added
  all 18 run entries from these three timing runs; verified all 93-sample entries
  and SQLite integrity. Removed three raw results and allocation profile. Whole
  formatter changed zero files/snippets/fingerprints. Pruned 69 obsolete object
  files (762,892,448 bytes).
- Build executables currently reflect the rejected candidate; rebuild from current
  source before any further compiler/native measurement. Do not mistake binaries
  for accepted source. Next optimization requires a fresh profile; investigate
  composite pipeline/collection costs without this lazy sequence allocation.

**Concrete JSON composite restorations — rejected, 2026-10-31:**
- Profiled accepted collection/recursive workloads, 20M iterations each, exact
  checksums 160,000,007/80,000,007; 54.776 s, peak 32,704 KiB. Recursive samples
  showed boxed restoration allocation inside sequence callbacks.
- Tried concrete closure returns for JSON arrays/records. Initial four E0700
  failures exposed validator-type overcapture in sequence's opaque return;
  candidate used SequenceRestoration like existing PairRestoration. Repaired
  candidate passed 71 integration + 94 schema library tests (165 total), peak
  1,718,560 KiB. No watchdog stops. Source and added witness now reverted.
- Release build 215.344 s/1,823,328 KiB; allocation profiles 75.073 s. Recursive
  allocations 40→38/iteration; collections 43, pipeline 10, union/discriminator
  unchanged. Native-only timing completed, 93 samples each, max load 3.511;
  result `results/2026-10-31T10-06-12-839Z.json`, no Node rerun.
- Against accepted baseline 2026-10-31T09:00:23.547Z: recursive 2.840458→2.781000
  ms (-2.09%); collections 2.285125→2.321375 (+1.59%); pipeline .636250→.646292
  (+1.58%); discriminator .965583→1.000375 (+3.60%); union .325959→.321500
  (-1.37%). Mixed result fails acceptance; not a proven general improvement.
  Ingest added five native entries; all entries and SQLite integrity verified.
  Raw timing/profile removed. Do not publish this rejected run.
- All candidate validation/measurement sessions terminal. Rust baseline attempt
  63472 ended before timing on harness ABI parameter names; fixed reference
  names to value0/value1/value2. Retry 32691 hit runner's 120-second automatic
  compiler rebuild timeout after source revert. No Rust measurement produced.
- Child monitor 36545 finished zero; accepted-source rebuild session 7730
  finished zero. Rust-only baseline final runner 29812 completed zero in 46.451 s,
  peak 1,095,200 KiB. All compiler binaries now match accepted source again.

**Hand-written Rust reference expansion — current, 2026-10-31:**
- Owner requested independent, ordinary Rust comparisons alongside **all 932**
  public workloads. **All 932 now have measured Rust references.** This is
  measured workload coverage, not universal hand-written-Rust parity. Next:
  profile the HTTP SQLite gap and resume ranked compiler/runtime optimization.
  All native-json and native-schema workloads now have Rust baselines. No compiler candidate is active.
- Reuse Node and Rust baselines instead of timing them for every optimization.
  Refresh on workload/protocol/reference-source/toolchain/hardware changes or
  when a noisy close comparison needs a fresh control. Tsrust parent/candidate
  A/B remains required for optimization changes. Reference-only additions do not
  establish Tsrust speedups and do not require another full compiler/LSP matrix.
- Every reference below passed independent result oracles and the original timing
  protocol: three processes, eight warmups, 31 timed samples each (**93 samples**).
  Rust-only batches used the sequential 5,000,000 KiB/1,800-second watchdog and
  quiet load at most 4. Rust edge tests used three threads. No Node/Tsrust timing
  reruns were needed for these reference-only additions.
- After each batch, store:ingest imported minimal histories into SQLite. Stored
  medians/sample counts, source hashes and database integrity were verified before
  deleting disposable raw results. Public selection pins each Rust timestamp and
  source SHA256 in website/tools/performance-runs.json. SQLite is the measurement
  source of truth; commits retain full implementation and checkpoint evidence.

| Completed reference family | Added | Checkpoint |
| --- | ---: | --- |
| Initial existing Rust references | 2 | d860cd34 |
| Schema text decoding/validation | 9 | f3ed77d6 through 63a3d145 |
| Integer operators | 16 | 0e5520b8 |
| Compound arithmetic | 11 | 64abfd1d |
| Boolean operations | 6 | 8b8e3405 |
| Number 32-bit operations | 6 | 38f5ec06 |
| Conversions/nullable values | 8 | b0a95868 |
| Nullable transport | 7 | 94304bcc |
| Scalar nullish/optional | 9 | 0e18ee2d |
| Owned nullish | 6 | 008562fd |
| Nullable fallback variants | 4 | a741f447 |
| Owned optional access | 5 | e3bc12a9 |
| String trim/concat | 12 | fef96456 |
| String search/repeat | 9 | 9d81dfdc |
| String slice/substring | 12 | eef97aea |
| String find/reverse-find | 6 | bad5076c |
| Scalar charAt | 5 | 646c6646 |
| Remaining string construction/indexing | 6 | f24af7a2 |
| String split/padding | 9 | cba5ad6b |
| String replacement | 6 | 637056fc |
| Updates/assignment values | 34 | b7f3df01 |
| Primitive values/conditionals/integer formats | 8 | d0aae614 |
| parseFloat | 6 | 8ecaf077 |
| Scalar floating-point/radix parsing | 20 | f484969a |
| Numeric formatting | 7 | 17e4a731 |
| Exact/approximate math | 35 | c77580ac |
| Conversions/literals/numeric arrays/binary16 | 13 | e06e0219 |
| Foundational array search/copy/selection | 25 | c57c8be5 |
| Stable array sorting | 28 | 3675fa30 |
| Range/copying array mutations | 40 | aec6d22a |
| Array removal/optional selection | 40 | 2ed02865 |
| Array insertion/splice | 16 | 65ca3406 |
| Array concatenation | 25 | 4de10d13 |
| Short-circuit/index/map/filter callbacks | 32 | bf661272 |
| Ordered reductions/owned accumulators | 28 | d3ede86e |
| forEach/recursive/discarded callbacks | 16 | 1088bca8 |
| Array flat/flatMap/of | 24 | 907bee6c |
| Array join/toString/locale/lastIndexOf | 21 | d0110399 |
| Array from/keys/values/entries/basic ownership | 15 | 91059aa6 |
| Array/call/iterable spread | 25 | 259614a6 |
| Default/rest parameters | 20 | e7ab431a |
| Nested functions/escaping closures | 6 | e2e2d217 |
| Generic function environments/replacement | 5 | e7bde0f0 |
| Nullable callable ownership/forwarding/factories | 10 | 5fcd9045 |
| Optional callbacks/function expressions | 6 | 5505c5ba |
| Remaining scalar control/operators | 20 | cc25c212 |
| String construction/ownership/scalar split | 19 | 8df76859 |
| String identity/literals/templates/HTML wrappers | 25 | ff52175b |
| Record ownership/recursive data/retained arrays | 14 | f0f8e8cb |
| Callback contracts/owned callable storage | 14 | f9afd1ec |
| Higher-order scalar callbacks/factories | 16 | ef303e0d |
| Higher-order owned callback results | 8 | 0d18cc7e |
| Declared functions/callbacks/owned values | 12 | f3744386 |
| Ordered Map/Set/grouping/Unicode iteration | 11 | cff3e51c |
| Native bytes/deque/heap/ordered collections | 10 | fa6d9cea |
| Borrowed byte-view records/callbacks/containers | 11 | b62879ca |
| Owned Result/errors/cleanup completion | 5 | 754255de |
| Scoped/fallible records/generic value adapters | 3 | 8d67d28c |
| Caller-borrowed callback record stores | 1 | bc1ca527 |
| Full scoped-record/map composition | 1 | 5ce1bd51 |
| Paths and OS identity | 12 | dd8c100d |
| Standard file I/O and buffered readback | 6 | 49d80b96 |
| JSON values/views/typed records/raw encoding | 17 | d8c411ae |
| JSONC and owned/borrowed JSONL streams | 5 | dde5d289 |
| Scalar schemas/refinement/owned arrays/parse | 14 | aa3a2b32; also refreshes integer bounds |
| Schema object policies/presence/defaults/literals/export | 8 | e94d7def |
| Standard language/string/record/tuple primitives | 13 | 6241c366 |
| Object enumeration/membership and union arrays | 8 | c6e11476 |
| Patterns/control/assignments/testing assertions | 13 | 28d30640 |
| Recursion/environment/clock/import/module state | 5 | 6329bc83; details below |
| RNG and Unicode normalization | 2 | Current checkpoint; details below |
| **Total** | **932** | **all public language/library cases covered** |

**Reference correctness and interpretation:**
- Sources are independently authored Rust using standard-library operations and
  established crates where needed, not generated Tsrust code with a new label.
  They preserve each measured corpus's ownership, checks and observed outputs;
  they are workload programs, not general JavaScript API reimplementations.
- Approximate math required 920 genuine operation probes plus 115 accumulated
  correctness cases across 23 references. c77580ac adds Rust engine point probes
  from the actual parity sources; scalar timing requires accepted MPFR probes.
  No numerical allowance or protocol was widened. Affected harness suites passed;
  report regeneration revalidated the actual observations before raw disposal.
- Durable edge tests in parity/*_edges.rs cover the custom boundary behavior:
  integer overflow/radix signs, nullish/default ownership, UTF-8/scalar indexing,
  split limits and replacement tokens, exact whitespace and numeric parsing,
  binary16 ties/underflow/overflow, decimal grouping and lazy defaults, relative
  array indices, deep copies, temporary ownership and stable sorting.
- Standard Rust numeric formatting matches the measured corpus. Tie rounding
  differs outside that corpus; do not claim a general JavaScript formatter.
  Numeric radix parsing uses bounded u128 input where required, decimal grouping
  uses shortest decimal text, and fractional radix text covers exact eighths in
  even bases. Corresponding source comments and tests state the measured domain.
- string_char_at_ascii_long is an **out-of-range control**: all 128 measured
  positions are negative. 646c6646 corrected its description without changing
  code, timing or protocol. Add a separate positive-index workload if needed.

**Performance page — current, 2026-10-31:**
- Generator selects the latest accepted Tsrust measurement per workload and
  Node independently, excluding rejected experiments. There are **932 distinct**
  comparisons. Each engine has its own date because reference baselines are reused.
- Public labels say **Tsrust**, **Node.js** and **hand-written Rust**. The page
  explains Tsrust source → Rust → executable, named ratio denominators and that
  unmeasured Rust rows are not losses. Never label Tsrust simply "Native".
- Current summary: Tsrust faster than Node.js in **613/932**, median throughput
  ratio **1.52×**, range **0.04×–189.48×**. Hand-written Rust faster than Node.js
  in **626/932 (932 measured)**. The ratios use mean throughput, not median time.
- Website generation/build passes locally; no deployment, push or CI is implied.
  Full-project formatting has consistently changed zero files/snippets/fingerprints.

**Final RNG/Unicode Rust references — verified, 2026-10-31:**
- Completes **932/932** independent Rust language/library baselines. The random
  workload performs two actual rand::random::<f64>() calls per iteration, scales
  the first to a dice roll, and retains all four property checks. The Unicode
  workload computes all seven normalization results at runtime, including both
  default paths and dynamic form selection; never substitutes corpus outputs.
- Reference-only **rand 0.10.3** and **unicode-normalization 0.1.25** (Unicode
  **17.0.0**) use an isolated locked parity-dependencies manifest, untimed offline
  preparation and the one shared Cargo target. Compiler/Node dependency paths
  remain unchanged. Actual Cargo artifacts are selected by exact library name,
  with hashes of library/manifest/lock recorded before minimal-store ingestion.
  One attempt to select non-workspace dependency packages through root Cargo
  panicked before compilation; the isolated reference manifest avoids that path.
- **Five Rust edge tests passed**, three threads under the watchdog: 65,536
  genuine RNG draws check range/diversity/distribution, retained row/roll checks,
  canonical/compatibility/Hangul/combining-order normalization (including inputs
  outside the corpus), idempotence, defaults, owned results and invalid form.
  Elapsed **4.643 seconds**, sampled peak **210,288 KiB**. No generated unsafe.
- Affected harness **13 passed, one existing opt-in native integration skipped**,
  peak **176,752 KiB**. Live dependency linking is exercised by the Rust edges
  and final runner. Fixed stale pending metadata/tests for these two measured
  families; their two registration suites reran successfully. No compiler suite
  rerun was needed for reference-only changes. Local release compiler/LSP version
  rebuild passed in **216.775 seconds**, peak **1,093,904 KiB**.
- Rust-only baseline **2026-10-31T18:02:59.219Z** passed all five correctness cases
  and **93 samples per workload** (three processes, eight warmups, 31 samples).
  Medians: RNG **0.039084 ms**, normalization **1.636000 ms**. Max load **2.520**,
  busy override off, elapsed **4.639 seconds**, peak **207,680 KiB**. Existing
  Tsrust and Node baselines reused; no fresh Node timing. Different RNG algorithms
  and normalization-library versions are disclosed without general parity claims.
- store:ingest added two histories; exact median/mean-rate/count/source identities
  and SQLite integrity verified before raw disposal. Public pins now select all
  932 Rust baselines: Rust faster than Node in **626/932**, Tsrust **613/932**,
  unchanged Tsrust/Node median ratio **1.52×**. All 66 website pages build locally.
  This later reference coverage is local and outside the frozen v0.1.1 promotion.
- Removed 6,682 obsolete benchmark outputs/cache directories (about 1.064 GB of
  top-level files), retaining only the current two workload builds. No archived
  sources, duplicate Cargo targets or extra handoff files were introduced.
- Next: profile the HTTP SQLite gap before considering an optimization. Static
  inspection shows SQL AnyPool preparation/metadata validation before each query;
  establish CPU/allocation or round-trip evidence and retain exact parameter/type
  diagnostics before changing it. Parent/candidate A/B and affected acceptance
  remain required. Also keep monitoring release 41226915583 and Pages promotion.

**HTTP request benchmarks — verified, 2026-10-31:**
- Owner requested actual HTTP text, JSON, parsed/transformed JSON, and parsed
  JSON plus SQLite lookup responses. Added independently authored Tsrust, Node
  HTTP/node:sqlite, and Rust Axum/serde/SQLx servers in benchmarks/http/requests.*.
  Four scenarios at one and 16 keep-alive connections yield **eight HTTP cases**,
  kept separate from the **932 language/library workloads** and their summary.
- One common Node HTTP client prepares payloads and independent expected response
  bytes outside timing. Every request checks status, content type and the entire
  body; malformed JSON/types/Unicode, invalid IDs and missing rows are preflighted.
  Each batch completes 128 requests. Database setup, builds and startup are
  excluded; parsing, serialization, queries, transport and response validation
  are included. This measures end-to-end loopback behavior, not isolated server
  saturation. All servers bind loopback port zero and shut down gracefully.
- SQLite uses a private in-memory database with 128 distinct seeded rows and one
  connection. Tsrust/Rust use asynchronous SQLx; Node uses a synchronous prepared
  statement. SQLite versions: Tsrust/Rust **3.51.3**, Node **3.50.4**. These are
  warm reads, with no disk/write throughput claim. The independent Rust server
  links the same prepared dependencies/features as Tsrust; no dependencies were
  added. Initial Axum Json feature mismatch was fixed using serde_json encoding
  and a response header, preserving the existing dependency features. Generated
  Rust contains no unsafe blocks/functions/implementations.
- **Four harness tests passed** under the watchdog. All three real servers passed
  final untimed verification (1,029 requests each), **1.041 seconds**, sampled
  peak **142,080 KiB**. Node **v24.11.1**, TypeScript **5.9.3**, Rust **1.96.1**.
- Quiet-machine baseline **2026-10-31T16:02:46.335Z** completed all three engines:
  three processes each, eight warmups then 31 samples per process/workload
  (**93 samples per engine/case; 2,232 timed batches total**). Engine order
  rotates; no busy override. Max load **3.841**, elapsed **89.312 seconds**,
  sampled watchdog peak **196,736 KiB**; no watchdog stops or correctness failures.
  Existing language/library Node/Rust baselines were reused, not rerun.
- Mean requests/second at **16 connections**, Tsrust / Node.js / hand-written Rust:
  text **47,279 / 48,502 / 48,527**; JSON **49,154 / 48,525 / 50,136**;
  parsed JSON **46,474 / 46,449 / 46,944**; SQLite JSON
  **24,886 / 46,516 / 32,220**. The page also shows one-connection results,
  median 128-request batch times, ratios and per-engine dates. These are new
  application baselines, not a claimed compiler optimization. SQLite exposes
  a measurable Tsrust-versus-Rust gap suitable for later profiling.
- store:ingest added **24 histories**. Stored medians, mean rates, 93-sample
  counts, source identities, quiet load and SQLite integrity were checked before
  deleting disposable results. Public selections pin source hashes and the full
  HTTP protocol. Whole-project formatting and the 66-page website build pass
  locally; rendered HTTP results verified. No push, deployment or CI.
- Next: finish the two remaining language/library Rust references
  (math_random_range and string_normalize_forms), then resume ranked profiling.
  The optimization goal remains active; HTTP implementation and baseline are complete.

**Rust recursion/host/module references — verified, 2026-10-31:**
- Five independent references add actual factorial/parity/string/depth/Fibonacci
  recursion, mutable captured steps, std environment reads, process-epoch monotonic
  clock, typed compiled JSON import data and exclusive module counter state.
  Standard Rust recursion performs each recursive call; environment reads
  distinguish missing/present-empty and reject invalid Unicode. LazyLock/Instant
  supplies actual clock readings in milliseconds. Typed SETTINGS constants mirror
  the checked imported JSON data; the body reads base and scans all three values,
  never substitutes a precomputed checksum.
- Added narrowly scoped moduleStateReferenceABI/driver support for an ordinary
  Rust function taking &mut i64. One caller counter is initialized outside all
  warmups/samples and retained across calls. Compiler moduleStateABI and its
  fallible generated-function guards remain unchanged/strict; references never
  manufacture compiler function/error wrappers. The adapter forwards reference
  mode only for that engine path.
- Five complete independent output checks and **93 samples per workload** passed
  (three processes, eight warmups, 31 samples), Rust-only run
  **2026-10-31T15:37:09.263Z**. Medians: recursion **0.569292 ms**, environment
  **1.973333**, clock **0.531250**, typed import **0.001250**, module counter
  **0.000750**. Max timing load **2.227**, sampled watchdog peak **328,624 KiB**,
  total **7.758 seconds**, busy override off. Node/Tsrust baselines reused.
- **13 harness tests passed**: adapters five, module-state three, typed import
  one, environment two, monotonic time two. Repaired a new readonly-owner guard
  witness that initially mutated the wrong parameter. Also fixed two stale
  registration-test expectations from pending-s09-family to the existing measured
  manifest status; source/operation/oracle guards retained. Reran only affected
  checks; no compiler/product changes or additional Node timing.
- Five host_state_edges.rs checks passed under the watchdog, three threads:
  recursive base cases/push order/results, real isolated child environment,
  finite nondecreasing clock reads, pinned compiled import data/array scan,
  repeated module-owner calls and checked overflow. No unsafe environment mutation
  or new dependencies. Edge compilation sampled peak **95,680 KiB**.
- store:ingest added five histories. Stored medians/counts, accepted hashes,
  timing load and SQLite integrity checked before raw disposal. Public pins
  refreshed; all 66 website pages build locally. No push/deploy/CI.
  Coverage **930/932**, **two remain**: math_random_range and string_normalize_forms.
  Next: inspect installed established Rust RNG/Unicode libraries and add the
  minimum reference-only dependency preparation if needed. Preserve compiler/
  Node dependency paths, exact workload work and independent range/normalization
  edge evidence; never precompute corpus outputs. All 932 required.
- Prior **28d30640** adds thirteen pattern/control/assertion references and thirteen
  edge checks; corrected finite close-comparison precision semantics, refreshed
  only that reference (0.093833 → 0.123667 ms; accuracy fix, not Tsrust speedup).
  Standard language/objects **6241c366/c6e11476** add 21 references/21 checks;
  schemas **aa3a2b32/e94d7def**, JSON **d8c411ae/dde5d289**, files **49d80b96**,
  paths/OS **dd8c100d**. Native-only references stay std-only.
- Prior composition checkpoint **5ce1bd51** covers the complete 2,985-line fixture
  and 136 observed columns, seven edge tests and 93 samples (18.420917 ms).
  Its public **Tsrust/Rust ratio 0.15×** exposes generated overhead for later
  profiling. It is a measured reference, not a Tsrust optimization.
- Harness acceptance retained: f3744386 adds verifyDeclaredReferenceABI for
  independent owned results while the compiler path keeps all declared-target,
  wrapper, capture and no-Clone checks. All 12 declared references passed real
  oracles/timing, 37 harness tests and eight Rust edge tests. New reference path
  never requires counterfeit generated Closure structs.
- Prior correction retained: 8df76859 fixes UTF-8 byte .length in 27 callable/
  nested/generic references and refreshes all their source hashes and Rust runs.
  Those measured length inputs were ASCII, so historical corpus outputs remain
  valid. Unicode edge expectations were corrected; scalar splitting still uses
  chars. That checkpoint passed all 46 oracles/93 samples and 37 affected tests.

**Owner rules learned this session:** match Rust's defaults for limits (no count
caps; depth bound only where rustc has `recursion_limit = 128`); generated code
must not contain `unsafe` (tests assert it); one handoff note only (this file);
fix defects when found; keep only the latest artifacts; ask the owner only for
merges or design decisions.

**Measured today (vs Node.js after tsc, M3):** single-character ASCII table and
direct `text.at(i)` (`string_at_characters` 30×, now 3.7× Node);
schema small-collection duplicate scan and identity restorations (1.26×);
`String(int)` via `to_string` (1.04–1.18×); `String(record.field)` moves a dead
record's field (`interfaces` 1.15×). Rejected with numbers: safe inline
small-string type (chains up to 47% slower, re-validating UTF-8 without
`unsafe`); `String(number)` whole-number path (up to 7% slower). Details in the
optimization log below.

**Ranked next targets:**
The next object/string allocation pass follows the detailed **Object and string
allocation — next optimization plan** in the current-state section above.

1. Scoped Maps across catch regions: completed the measured snapshot and
   callable-boundary epoch optimizations above. Depth 80 fell from 6.067 s to
   1.403 s in the first A/B, then from a fresh 1.376 s to 0.250 s in the second.
   Depth 128 is now 0.564 s versus 4.631 s on the preceding commit. Keep these
   regressions; proceed to schema trials before further compiler optimization.
2. Schema restoration adapters: completed the measured typed-restoration
   optimization above. Collection workload is 1.11× faster with 43 rather than
   48 allocations per iteration; retain the rollback/lifetime regressions.
   Move to mixed-type arrays before further schema allocation optimization.
3. Scalar-union arrays: completed safe inline storage above. Small-array
   scenarios measured 1.09–1.22× faster; type_aliases allocations fell by one
   per iteration without a reliable timing gain. Heap control also measured.
   Keep recursive/nested storage and enum text witnesses; move to JSON decoding.
4. Plain-schema `DeserializeSeed` decoding accepted above: nested 3.122×,
   fused 1.161× faster, fewer allocations. Advanced schemas retain the existing
   decoder; profile remaining costs before extending the fast path.
5. Completed the measured string sizing pass above: single empty-pattern token
   replacement 1.434x faster; replaceAll pre-sizing rejected. Rust-default
   ownership retained: borrow where lifetimes permit, copy independent owned
   strings. No reference-counted sharing or arbitrary size threshold.

**Next continuation:** the ranked passes and typed JSON pipeline restoration
have measured outcomes, but do not claim universal hand-written-Rust parity.
Advanced composite pipelines, choices and collections still box restoration
state. Reprofile those boundaries before extending the accepted typed approach.
Initial required validation and the full later milestone are complete; only run
affected suites for the next coherent change. Active candidate/session is recorded
above; follow that state before starting another job.

## Most recent owner correction — highest immediate priority

The owner said:

> im basically just seeing a lot of example how to documents appearing instead of
> actual documentation of the std lib and how the language works - it would be
> better to have an interactive guide of how tos than to just have a bunch of
> articles hogging up the search results

The current site over-fragments language topics into many small walkthroughs,
especially callable/generic features. Do not continue producing those articles.
Their verified facts/examples are reusable evidence, not an approved final docs
architecture. The initial `docs/plans/public-documentation-site.md` already proposed
broad language-reference topics; implementation drifted from that intent.

The requested direction is:

- Make language semantics and standard-library API reference the primary docs.
- Consolidate how-tos into an interactive guided learning experience; do not index
  every tutorial step as another competing reference article.
- Keep examples within reference entries where they explain an operation, without
  making examples the whole documentation model.
- Plan the revised structure in Markdown, then implement it incrementally. Preserve
  useful audited contracts and migrate links/anchors safely. No empty placeholders.
- Every page still needs factual and presentation review. Reorganization does not
  establish correctness, and a successful build does not establish a completed audit.

No revised navigation, consolidated reference, interactive guide or API-reference
generator has been implemented. Codex stopped after inspecting options. The
information-architecture skill was read, but no `.design` artifacts were created.

Useful implementation facts: Astro 7.4.2, Starlight 0.42.7; site configuration is
`website/astro.config.mjs`, content is `website/src/content/docs/`. Installed
Starlight schema supports frontmatter `pagefind: false`. Reuse its navigation,
styles and accessibility instead of adding another framework. Do not hide useful
reference content before its replacement exists. Preserve old URLs through the
migration, then remove superseded public source copies.

`crates/tsrust-lsp/src/builtins.json` has 103 groups / 894 entries, including
signatures, parameter documentation, behavior, ownership and examples. It is a
useful canonical source for a synchronized stdlib reference. Some group keys are
internal names; do not expose them as public types without mapping them. The
existing library overview lists 23 public module names, but detailed API reference
coverage is still incomplete. `std:path` (7 exports) and `std:env` (3 entries) were
only inspected as possible starting points; no generation work was done.

## Full goal and remaining work

Finish the finite backend-only language/CLI/docs/platform polish from PR #2. Do
not expand into native UI or arbitrary TypeScript compatibility. The original fan
stop was explicitly lifted: the owner is using a different computer.

- Async testing registration and HTTP body/SDK work have substantial accepted
  evidence in `docs/agent-goal.md` and the progress record; preserve that evidence.
- Shared dependency caches and paired CLI/LSP installation/update have implemented
  workflows and focused macOS/Linux/Windows evidence. Remaining distribution
  checks must be reconciled, not assumed complete.
- Windows and Linux installer/locked-binary checks passed previously. The newer
  isolated editor UTF-16 fixture and fresh toolchain smoke/acquisition paths still
  need their outstanding platform executions. Exact release-build commands and
  actual published acquisition remain unproven where marked pending.
- GitHub Pages was selected and configured, but a successful production deployment
  and public URLs remain pending. Local preview/build is not deployment evidence.
- Complete the finite remaining docs inventory in the migration/audit plans, using
  the corrected reference/learning architecture. Old pending/unsupported claims
  often contradict newer compiler behavior; check source before copying them.
- Retain completed family correctness/performance records. Do not restart full
  matrices or timing protocols after routine documentation changes.

A reported LSP retention issue was repaired and a bounded 30-minute macOS stress
check passed. The old blanket warning was removed. This is not proof of every
possible long-session behavior. HTTP performance evidence and original SQLite
records already exist; no new measurements were taken during recent docs edits.

The last retained nested-function guide still contains unaudited owned/scoped
selection, single-use call scopes, generic replacement and mutable factory arrays.
Its source-flow paragraph was investigated, but is not a mandate for another page.
Other language/library/operational inventory remains pending too.

## GitHub and release constraints

Original PR #2 was bootstrapped with owner approval. Last recorded draft delivery
PR is https://github.com/Forever-Frameworks-LLC/tsrust/pull/4, base `stag`, remote
branch `codex/backend-polish-staging`. Last recorded remote head was
`61ee67a44152437848a53d0fea6499e7ffec56ec`; its tree matched local `1b375f06`.
Later local work, including the recent docs commits, has not been pushed.
These are last-recorded values, not a fresh remote-state check. Revalidate before
any push. Do not force-push or trigger duplicate CI runs.

Owner-approved policy: branch-to-stag PRs run tests and require one review, with
owner review bypass; stag-to-main promotion compiles macOS/Linux/Windows artifacts,
then auto-merges/publishes if successful. New tags may be created by repository
writers; existing tags/releases must remain immutable. Prefer caching, minimal
permissions and dependable artifact reuse. Avoid unnecessary GitHub Actions usage.
No concrete new release version was selected; do not invent one or publish a new
release as part of this handoff.

## Validation and cleanup state

Recent accepted docs evidence is indexed in
`docs/plans/public-documentation-content-audit.md` and
`docs/plans/public-documentation-migration.md`. Their content-audit acceptance does
not mean the owner approved the fragmented article architecture.

The unfinished `language/callable-control-flow.md` article was deleted; its sidebar
addition and source-guide edits were reverted. No unfinished implementation remains.
Before deletion its two intentional rejection examples returned `arity`, and two
native examples printed `4 1` / `7` and `7 1`. Focused compiler flow checks passed
15 tests and editor diagnostics passed 4. Disposable logs, if still present:
`/tmp/tsrust-flow-doc-contracts.log` and `/tmp/tsrust-flow-doc-editor.log`. These are
optional current-run evidence, not durable analytics or required deliverables.

Cleanup rebuilt the restored 66-page site successfully (`npm --prefix website run
build`), removing the deleted article from generated output. The last full internal
link/anchor pass on that same committed site also passed. The agent-started docs
preview on port 4322 was stopped, its browser tab closed and viewport override reset.
No known compiler/test job is running. Restart preview if needed with
`npm --prefix website run dev -- --host 127.0.0.1 --port 4322`.

198 disposable native program cache entries (274 MiB) were pruned from
`target/tsrust-native/v2`; root lock files, shared native runtime cache and Cargo
target were preserved. Native programs rebuild when needed. Source, owner edits,
retained UI, fixtures and historical evidence were not removed.

## Working rules

One agent in this tree at a time. Inspect status first. Use the existing Cargo
target, `CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1`, and focused section-level checks.
Do not touch the owner's VS Code instance. Format the entire project before each
source checkpoint with `TSRUST="$PWD/target/debug/tsrust" node tools/format-project.mjs`.
Keep native behavior, diagnostics, IntelliSense and examples consistent when code
changes. No new heavy suite for prose alone.

No source snapshots/archive directories. SQLite remains canonical analytics;
never publish ignored `benchmarks/records.sqlite`. Public measurements must use
`benchmarks/storage/public.py` and its provenance allowlist. Preserve original
records and distinguish unmeasured work from measured work. Keep the handoff and
finite acceptance checklist current; do not claim completion from partial evidence.

## Claude progress — 2026-10-28

Architecture recorded in `docs/plans/documentation-architecture.md` (reference
first, one interactive Learn track per journey, chapter merge table and rules).

- Standard-library and built-in reference: `website/tools/generate-api-reference.mjs`
  renders all 103 catalog groups (minus frozen `std:ui` and the internal testing
  runtime) into 28 generated pages before every `dev`/`build`. Output is ignored
  by Git. The docs workflow now also triggers on `builtins.json` changes.
- First language chapter merged: `language/optional-values/` replaces
  borrowed-optional-fallback, optional-chaining, nullish-coalescing and
  nullish-assignment. Old routes redirect to chapter anchors. All ten examples ran
  with the current debug compiler with matching output; five rejection claims
  were checked with `tsrust check`.
- Link checker now validates redirect pages against their destination anchor
  (unit test added). Site build: 91 pages + redirects; link/anchor check passed.

Not done: remaining nine chapter merges, the first Learn track, folding library
guides and backend how-tos into tracks, and the CI/installation/platform/deployment
reconciliation listed above. Nothing was pushed; no GitHub run was triggered.

## Distribution, CI and deployment reconciliation — 2026-10-28 (rows updated 2026-10-29)

Checked against workflow source and read-only `gh` queries; no run was triggered.

| Requirement | Current state | Evidence |
| --- | --- | --- |
| Release artifacts for macOS, Linux, Windows | All four targets built and smoke-tested; `v0.1.0-rc.1` published as a prerelease (updated 2026-10-29). | Release run 40861023720, Promote run 40864132322 |
| Windows paired install, locked binaries, shared caches | Passed on hosted Windows. An earlier failure (40724567590, 90382caa) was fixed by 9b658857. | Runs 40731485167 (9b658857), 40748893795 (86435b88) |
| Async testing on three platforms | Passed. | Run 40672007920 (c5b3d4a9) |
| PR CI on the staging branch | Linux compiler/LSP, VS Code extension and docs build passed for PR head 61ee67a4. | Runs 40761578381, 40761578389 |
| macOS | Local focused acceptance only (see `docs/agent-goal.md`). | Local evidence |
| GitHub Pages | Deployed at https://forever-frameworks-llc.github.io/tsrust/ (updated 2026-10-29). | Documentation run 40864208241 |
| Install/self-update from a published release | rc.1 archive installed by hand on macOS arm64 (version, doctor, native run). Managed install and self-update need the first stable release: pending `v0.1.0`. | See the release-candidate status section |
| Windows editor UTF-16 fixture | Passed on Linux and Windows (updated 2026-10-29). | Editor acceptance run 40816862762 |

Remote branch `codex/backend-polish-staging` (PR #4, draft, base `stag`) is still at
61ee67a4. Local history is not a descendant of it (the PR branch was rebuilt
from the same trees), so compare trees before any push. All local commits since
then, including this documentation restructure, are unpushed.

## Release candidate status — 2026-10-29

- Editor acceptance passed on Linux and Windows (CI run 40816862762), including
  the surrogate-pair UTF-16 diagnostic fixture.
- The five package-export tests pass locally once `tsrust setup --portable
  --offline --yes` has prepared source packs.
- Owner chose a pre-release. Version is `0.1.0-rc.1`. The toolchain installer
  now accepts `MAJOR.MINOR.PATCH-rc.N` (ordered before the stable version);
  update discovery still considers only stable releases, so self-update from a
  published release still needs a stable release to verify.
- PR #4 (`codex/backend-polish-staging` → `stag`) was merged by the owner with
  the review bypass (squash commit cf4e729c). Authors cannot approve their own
  PRs; the bypass is the owner's path for agent-opened PRs.
- PR #5 (`stag` → `main`) built all four targets and the extension (Release run
  40861023720). Promote (run 40864132322) merged it and published the
  `v0.1.0-rc.1` prerelease with four archives, `tsrust-0.2.0.vsix` and
  `SHA256SUMS`. The Documentation workflow (run 40864208241) deployed
  https://forever-frameworks-llc.github.io/tsrust/.
- Release smoke test on macOS arm64: the archive matched `SHA256SUMS`;
  `tsrust --version` printed `0.1.0-rc.1`, `tsrust-lsp --version` printed
  `0.2.0`; `doctor` and `doctor --native` passed; `tsrust run` executed a native
  program correctly. `tsrust toolchain install --root` refused as designed: "No
  published stable official toolchain is available yet".
- Pages smoke test: the home page, Learn, Language, globals guide, generated
  globals and `std:sql` reference returned 200; the old
  `reference/builtins/globals/` path redirects to `reference/globals/functions/`.
- Still open: managed install and self-update need the first stable release.
  Linux, Windows and Intel macOS archives were built and checked in CI, not
  installed by hand. This handoff commit is local only and reaches `stag` with
  the next PR.

## 0.1.0 milestone — 2026-10-29

Owner direction: no rc.2; finish the work, then publish `0.1.0`; run the
performance milestone and put the measurements in the docs.

- **Legacy UI out of the release CLI.** `tsrust ui`, `new --ui`, `dev --target
  web` and the UI menu are behind the `legacy-ui` Cargo feature (off in release
  builds). The bundled component catalog and `shadcn-MIT.txt` are not in the
  archive. Compiler `std:ui` support and the `tsrust-ui` crate remain, frozen.
  UI CLI tests run only with `--features legacy-ui`.
- **Docs inventory finished.** New chapters `language/operators` and
  `language/errors`; numbers, enums and recursive records in `language/types`;
  `for...in` and labels in `language/control-flow`; diagnostic suppression in
  `tools/cli`. 24 retained guides merged or deleted; research and developer
  guides labeled. See `docs/plans/documentation-architecture.md`.
- **Defects fixed on the way** (each with a regression test): formatter spacing
  for `c ? (): T => a : b`; misleading diagnostics for non-boolean `||=` on an
  array element and for rebinding a borrowed-view `let`; `setup --portable`
  failing on a damaged cached pack (now rebuilt and replaced); benchmark
  harness hiding prepared source packs behind its isolated Cargo home; three
  benchmark policies rejecting formatting-only source changes (now use
  registered token-identical proofs).
- **Performance milestone.** All 930 workloads measured native vs Node after
  tsc (3 × 31 samples, 175,620 samples), 880 profiled. Native faster on 613
  (median 1.52×). Median native change vs earlier measurements 1.00×; no
  regression beyond noise. Evidence: `benchmarks/storage/release-0.1.0-analytics/`.
  The public page `project/performance` is generated from
  `benchmarks/measurements.sqlite` at docs build time; publish a different run by
  editing `website/tools/performance-runs.json`.
- **Optimization targets** (native slower than Node): JSON/schema decoding,
  record/object copies, Unicode case conversion, `String.at`, recursive array
  copies, enum and interface records.
- **Known pre-existing issues, not fixed:** `store.py verify` rejects old
  absolute-path log records already in `records.sqlite`; about 55 benchmark
  harness unit tests are stale (hard-coded `target/editor/debug/tsrust`, frozen
  manifest-prefix hashes broken by fingerprint refreshes). Neither affects
  measurement.

## v0.1.0 published — 2026-10-29

- PR #6 (owner bypass merge) into `stag`; PR #7 `stag` → `main` built all four
  targets and the extension (Release run 40930352738); Promote (run
  40934384351) merged it and published `v0.1.0` as a regular release; the
  Documentation workflow redeployed Pages (performance page live).
- Archive check (macOS arm64): checksum OK; attributions hold only the Tsrust
  license and dependency notices (no UI notice); help has no `tsrust ui`;
  `tsrust 0.1.0` / `tsrust-lsp 0.2.0`; `doctor` OK.
- Managed install against the published release: `toolchain install --root`
  selected 0.1.0 and its LSP; `check` reports no newer release; `update` changes
  nothing; `rollback` refuses (no previous version); `doctor` reports a managed,
  intact pair. A console project runs and tests, and a backend project checks.
- Still unverified: updating across two real releases (needs a second release),
  and hand installation on Linux, Windows and Intel macOS (CI smoke only).


Baseline: the 0.1.0 run in `benchmarks/measurements.sqlite`. Every "after" is a
quiet-machine run (runner load guard, limit = half the logical CPUs).

| Change | Result | Kept |
| --- | --- | --- |
| Runner waits for load < CPUs/2 before each timed process; records max load | Desktop apps alone keep load near 3–4 on this M3; a limit of 2 never starts | Yes |
| `String(stringEnum)` borrows the static member text; whole numbers below 2^53 format as integers | `enum_values` 6.65M → 17.3M ops/s (2.6×; 0.14× → 0.35× of Node); other number workloads within noise | Yes (cc2dc382) |
| Left-nested string `+` appends to the fresh left temporary (exact `reserve_exact` growth) | `string_add_chained_*` 23.2M → 22.8M, `string_concatenation`/`interfaces` unchanged: allocation, not intermediate copying, dominates short strings; V8 uses ropes | No (reverted) |
| Dead `const` array receivers of `concat()`/`toReversed()` are moved (single reference, same statement list, no closures) | concat-zero workloads 1.94–3.01× (record 0.43× → 1.04× of Node); moved `toReversed` 1.27–1.57×; others within noise | Yes |
| Receiver moves extended to `slice()` (plain `Vec` only) and `toSorted()`; closures only block locals they capture | Tested (75 suites); no benchmark workload has a dead `slice()`/`toSorted()` receiver, so unmeasured | Yes |
| String search: first-byte scan for haystacks ≤ 64 bytes (`indexOf`, `lastIndexOf`, `includes`) | `indexOf` 2.45–2.72×, `lastIndexOf` 1.05–1.74×, `includes` 1.26–1.32×, JSON string projection 12.3× (0.33× → 4.1× of Node). A plain byte-window scan was tried first: 0.43–0.63× on `includes`/default `lastIndexOf` (slice compare at every position), so it was replaced | Yes |
| mimalloc as global allocator (owner approved trying it; then paused) | Schema pipeline 1.85× with mimalloc, `record_mutation` 0.91×. Profiles showed time inside macOS `free()`, but this Mac has 8 GB RAM with ~7.4 GB swap in use and ~3 GB compressed, which inflates allocator cost. Not adopted: a dependency in every program needs evidence from a clean machine. Dependency changes reverted; the harness keeps an experimental `--allocator-rlib` option | No |
| Dynamic `JSON.parse` re-scans integer tokens only when serde produced a float (only an integer too large for i64/u64 arrives that way) | JSON workloads up to 1.32× (`string_reads`, `refinement`, `typeof`, `scalars`); 54 JSON/schema workloads median 1.00×; `native_json_raw` 0.84× re-measured alone at 0.99× and does not use the parser | Yes |
| Typed JSON decode: validation pass records floats; integer re-scan only after one | 54 JSON/schema workloads median 1.04× (typed JSONC 1.18×, roundtrip 1.15×, object strip 1.15×); lowest 0.97× (noise). The validation pass itself stays: it keeps malformed input from reaching user callbacks | Yes |
| `JSON.parse` of a string skips UTF-8 re-validation (`parse_valid_utf8`); JSONC and byte inputs keep the check | vs the previous run: 26 JSON workloads median 1.01×, `typeof` 1.29×, scalars 1.18×; lowest 0.98× (noise). Inputs are tiny, so larger documents gain more | Yes |
| `JSON.parse`/`tryParse`/`std:json` `parse` without limits = plain serde_json (owner direction: TypeScript-familiar API, Rust accuracy). No default limits (HTTP `bodyLimit` and serde's 128-level nesting remain); over-range integers still reject; Rust number formatting kept (a JavaScript-style formatter was tried and reverted as a JS quirk). Also fixed a CLI panic when a diagnostic points past the user file | Speed roughly neutral: median 0.97× vs the previous run, 1.05× vs 0.1.0; drops of 0.82–0.84× are in workloads that do not call `JSON.parse`, so run-to-run noise here is about ±15%. Generated/hand-written for `int_bounds` 0.39× → 0.43×. First version regressed `typeof` to 0.33× by allocating in the float check; fixed | Yes |
| Parity engine `rust-hand` (hand-written Rust in `benchmarks/runtime/parity/`) | `native_schema_int_bounds`: generated 0.34× → 0.40× of hand-written. Per iteration: generated 2 allocations vs 1; the rest is parse overhead (UTF-8 re-check, budgeted seed, typed decode parses twice) and issue conversion | Tooling |
| `dynamic_unicode_uppercase` investigated | Node reaches ~3 ns per 20-char Unicode `toUpperCase`, i.e. it reuses the loop-invariant result; native does the real conversion (~75 ns, std ASCII fast path already). Benchmark-design artifact, not a conversion gap | n/a |

Measurement caveat: every local number, including the 0.1.0 baseline, comes from a memory-starved Mac (8 GB RAM, heavy swap). Code-level wins (fewer copies, faster search) reduce work on any machine, but magnitudes need confirming on a clean runner before publishing or adopting dependencies.
| Typed `JSON.parse<T>`/`tryParse<T>` without limits: one serde decode plus the integer-token scan (`typed::decode_plain`), no validation pass. Explicit limits, JSONC and JSONL keep the budgeted path; schema decoding is unchanged | Median duration vs the previous run: `native_json_typed` 1.38×, `native_json_literal_union` 1.35×; Node.js controls 1.01× (comparable conditions). Native ÷ Node.js: typed 0.71× → 0.96×, literal union 0.64× → 0.85×. Load ≤ 4 throughout | Yes |
| Schema rejection path (`native_schema_int_bounds`): pooled the intermediate `Issues` buffer to drop one allocation and free per rejection | No wall-clock change at 40M iterations (3.23–3.58 s vs 3.19–3.37 s): the thread-local pool costs about what the `free` did. A profile shows ~85% of the loop is the rejection path (issue buffer, `error.issues` array, record conversion); the array allocation is required by the API. Generated 79–85 ns/iteration vs hand-written 34 ns. Not adopted | No |
| `text + String(n)` (int, number or numeric enum) writes the digits into the concatenation buffer instead of formatting a temporary string; template interpolations of int and whole numbers append digits directly (`tsr_push_int`, `tsr_push_whole_number`; other numbers keep their existing spelling) | 18 affected workloads, stashed before vs after, back to back, load ≤ 3.9: median 1.07×, none slower; `enum_values` 1.62× (0.33× → 0.54× of Node.js), `array_from_map_string` 1.54× (0.51× → 0.79×), `interfaces` 1.25×, `function_expressions` 1.23× (0.73× → 0.92×). Node.js controls 0.87–1.06× | Yes |
| String-addition chains build one buffer (`6f474114`); int/number/boolean chains emit one running value (`d3f85d41`) | 27 workloads, `3790b8e0` vs `423a8dbd` back to back, load ≤ 4: median 1.04×, none slower; `interfaces` 1.84× (0.21× → 0.39× of Node.js), `map_iteration` 1.76× (0.30× → 0.52×), `string_iteration` 1.41×, `map_for_each` 1.28×, `object_values_entries` 1.23×. Arithmetic and boolean chains 1.00–1.07× (LLVM already folded the nested form; the flat form exists for depth, not speed). Node.js controls 0.95–1.13× | Yes |
| `String(x)` of a dead `const` string local moves it instead of cloning (`92600e26`; receiver-move rule) | 8 record workloads, `7d7b97d6` vs `92600e26`, load ≤ 3.9: `type_aliases` 1.13× (0.19× → 0.23× of Node.js); the other seven do not use the pattern and stay within noise (0.94–1.01×, while Node.js controls drifted 0.88–0.96×, so the second half ran on a slightly slower machine) | Yes |
| In-place `text += value` for uncaptured locals (`4855dbee`, `tsr_string_append`: borrowed → one exact allocation, owned → `push_str`) | Long accumulation (20,000 appends of 3 bytes, ×20): 0.02 s vs 0.20 s CPU for the copying form (10×, and the gap grows with length: linear vs quadratic). Short single appends on the 14 `+=` workloads, `28f66cdb` vs `4855dbee`: 0.88–1.02× (`compound_string_add_ascii` 0.94×, `declaration_lists` 0.94×), within noise or a few percent from amortized capacity retained in output arrays, the same as hand-written `push_str` | Yes (kept for the algorithmic win; parity with Rust `push_str`) |
| `text += a + b` (and longer right-side chains) appends each operand in place after one reserve (`500c09a2`, `tsr_string_reserve`) | 15 workloads, `d6fae277` vs `500c09a2`, load ≤ 4: `labeled_control` 2.37× (0.30× → 0.71× of Node.js), `for_pattern_initializers` 1.19× (0.19× → 0.25×), `in_operator` 1.09×; others 0.94–1.04× (the `compound_string_add_*` single appends are unaffected by this change). Node.js controls 0.91–1.08× | Yes |
| Appended `name.slice(...)` of a local borrows its range (`0800c571`, `tsr_slice_str`); new hand-written parity program `parity/declaration_lists.rs` | `1c652680` vs `0800c571`, load ≤ 3.9: `declaration_lists` 1.24× (0.18× → 0.22× of Node.js; generated/hand-written 0.38× → 0.46×); five other slice workloads 0.98–1.03× (no appended slices). Hand-written Rust itself is 0.53× of Node.js on this workload: V8 shares strings, so `slice()`/`repeat(1)` are free there. Remaining generated overhead: the `words` array allocated only to be destructured, `repeat(1)` and `second.slice()` copies of borrowed elements | Yes |
| Map/Set keys hashed once with a per-process random key; bucket tables use that hash directly (`27bb63d4`) | 11 workloads, `a3959610` vs `27bb63d4`, load ≤ 4: `map_values` 1.23× (0.39× → 0.48× of Node.js), `set_values` 1.14× (0.40× → 0.49×), `set_for_each` 1.13×, `map_for_each` 1.11×, `native_schema_collections` 1.06×; `map_iteration` 1.12× matched by its Node.js control (noise); unrelated controls 0.98–1.04× | Yes |
| Maps and Sets with at most 8 slots scan their slots instead of hashing; the 9th slot builds the index once (`f43235a6`) | 13 collection workloads, `647bd09f` vs `f43235a6`, load ≤ 3.9: 1.17–1.60×, none slower. `set_algebra` 1.60× (0.54× → 0.88× of Node.js), `map_values` 1.57× (0.45× → 0.73×), `set_for_each` 1.47×, `set_relations` 1.44× (0.73× → 1.03×), `set_entries` 1.40× (0.99× → 1.43×), `map_for_each` 1.35×, `array_from_collections` 1.27×, `map_iteration` 1.22×, `map_group_by` 1.21×, `map_snapshots` 1.17×, `native_schema_collections` 1.17×. Node.js controls 0.93–1.11× | Yes |
| `String(boolean)` borrows "true"/"false"; `Object.hasOwn` recognizes canonical index keys without formatting the index (`9ca1e25b`) | 10 workloads, `3aad90d0` vs `9ca1e25b`, load ≤ 4: `object_has_own` 3.16× (0.31× → 1.02× of Node.js), `array_splice_boolean` 2.13× (0.59× → 1.32×), `map_values` 1.20× (0.74× → 0.87×); others 0.99–1.03×. `array_splice_int` first read 0.86× but re-timed alone at 0.98× (noise; it does not use either path) | Yes |
| One-character ASCII strings borrow a static 128-byte table (`charAt`, `split("")`, `for…of` text, spreads, `Set(text)`); `text.at(i)` selects one character instead of `text.split("").at(i)` (`c35e14da`) | 8 workloads, `85912b00` vs `c35e14da`, load ≤ 4: `string_at_characters` 30.1× (0.13× → 3.71× of Node.js), `array_from_string_characters` 2.12× (0.31× → 0.65×), `string_iteration` 1.71× (0.25× → 0.44×), `string_split_empty_delimiter` 1.13×; controls 0.98–1.01×, Node.js controls 0.98–1.02× | Yes |
| Small-string optimization: runtime `String` as `Borrowed`/`Owned`/`Inline(≤15 bytes)` (24 bytes, like `Cow`), inline clones/slices/padding/`+=`, chains and templates through a sink (branch `codex/small-string-experiment`, `53c111b2`…) | 20 workloads vs `85912b00`: `string_at_characters` 2.03×, `array_from_string_characters` 1.57×, splits 1.03–1.20×, but `string_add_chained_*` 0.53–0.56×, `map_iteration` 0.75×, `enum_values` 0.81×, `interfaces` 0.85×, `string_iteration`/`string_concatenation` 0.87–0.88×. Generated code may not contain `unsafe`, so inline text is re-validated as UTF-8 whenever it is read as `&str`; standalone, the chain is 0.66 ms vs 0.39 ms and an unchecked variant still only reached ~0.55×. The character wins were taken by the static-table change above instead | No (branch kept) |
| Schema collections: duplicate keys of ≤8 entries compared pairwise without a HashMap; leaves whose int/number/boolean output equals the input restore through the capture-free identity closure (`283f61cb`) | 12 schema/JSON workloads, `c35e14da` vs `283f61cb`: `native_schema_collections` 1.26× (0.25× → 0.33× of Node.js; 59 → 48 allocations per iteration, `map.safeParse` of one entry 12 → 9). Others 0.90–1.03× with Node.js controls 0.92–1.05×; `native_json_containers`/`native_schema_json_array` re-timed alone at 1.01×/1.00×. Remaining schema allocations are structural: each trial layer (sequence, pair, remap, inject) boxes a `dyn FnOnce` restoration capturing the previous one, and entry/value Vecs are rebuilt between layers; generic restorations are the next fix | Yes |
| `String(int)` formats with `i64::to_string` instead of building a `TsrConsoleValue` and calling `label()` (`2f29f4e3`) | 8 workloads vs `85912b00`: `array_splice_int` 1.18× (0.56× → 0.66× of Node.js), `type_aliases` 1.15×, `labeled_control` 1.10×, `declaration_lists` 1.09× (gen/hand 0.46 → ~0.50), `interfaces` 1.04×, `enum_values` 1.00×; `string_concatenation` 1.07× and `map_iteration` 1.03× had Node.js controls at 0.81×/0.93× so are not conclusive | Yes |
| `String(number)` tries whole-number digits before the number formatter (tried as `cce967c1`, dropped) | 6 workloads vs `8afa1e58`: `array_pop_number` 0.93×, `array_shift_number`/`nullable_number_null` 0.96×, others 0.98–1.01× (Node.js controls 1.00–1.03×). Whole numbers allocate once either way and mixed inputs (0.5, NaN, ±Infinity) pay for the extra check | No |
| `String(record.field)` moves the field out of a `const` record local that is read only there (`52fef7cb`) | 4 workloads vs `2f29f4e3`: `interfaces` 1.15× (0.41× → 0.47× of Node.js; it now allocates what hand-written Rust would), `type_aliases` 1.01×, `declaration_lists` 1.00×, `labeled_control` 0.99× (Node.js controls 0.93–1.00×) | Yes |

Allocation profile (2026-10-30, `profile-native.mjs`): `native_schema_recursive`
47 allocations per iteration, `native_schema_export` 25, `native_schema_pipeline`
13, `interfaces` 8, `native_schema_string` 3, `enum_values` 2. Schema paths
build a dynamic `JSON.parse` tree and then copy it into typed output; fusing
`schema.safeParse(JSON.parse(text))` into one typed decode must keep the throw
for malformed JSON. Records now allocate close to hand-written Rust; the rest of
the gap to Node.js is short strings, which V8 keeps inline. A safe
small-string representation was tried on 2026-10-30 and rejected (see the
optimization log): without `unsafe`, reading inline text re-validates UTF-8 and
concatenation-heavy workloads lost up to 47%. Remaining short-string wins should
come from borrowing (static tables, slices of stable receivers) and avoiding
intermediate strings, as the one-character table did.

Schema text decoding (2026-10-30): `schema.safeParseJson(text)` measured about
1.25× slower than `schema.safeParse(JSON.parse(text))` (300,000 nested-object
decodes: 0.27 s vs 0.22 s CPU), so fusing the latter into the former would be a
regression. `source::document` validates the whole document, then each cursor
re-parses its subtree with serde when a field is read. Skipping the integer
re-scan without floats changed nothing measurable. The fix is a single-pass
schema `DeserializeSeed` decoder (as typed `JSON.parse<T>` already uses), which
replaces the cursor emission in `stdlib/schema`.

Next targets: add more `parity/` programs; add workloads with dead `slice()`/`toSorted()` receivers; `with()`/`toSpliced()` moves (need element assignment across array wrappers); JSON/schema decoding (group median 0.78×); record construction (records group 0.51×); `String.at`.

## Measurement store — 2026-10-30

Owner direction: canonical records are not kept. `benchmarks/measurements.sqlite`
is one table, `measurements(workload, engine, history)`, with a JSON history of
`{at, ms, ops, samples}` per run (commit `chore(benchmarks): one row per measured
workload…`). After timing, run `npm run store:ingest` in `benchmarks/runtime`;
`npm run checkpoint` prints the latest native/Node.js ratios. The ignored
`benchmarks/records.sqlite` (596 MB) was deleted, and the old analytics/export
scripts were removed. Raw files in `benchmarks/runtime/results` are disposable
once ingested; the 128 existing files (881 MB) were ingested and deleted on
2026-10-30 at the owner's request. Regenerating `BENCHMARKS.md` (`npm run
report`) now only sees result files written after that.

## Nesting limits removed — 2026-10-30

Owner direction (2026-10-29): remove limits like the 128-level nesting limit.

- `d3f85d41`: chains of `+ - * / % && ||` are flat in the checker (shared type
  hints during descent, linear time) and in emitted Rust (one buffer or one
  running value); generated programs allow `unused_parens`.
- Discovery (`benchmarks/compiler/nesting-discovery/run.py --unbounded-parser
  --breadth-first --start 256 --factor 4`, 60 s/case) over all 80 families found
  no crash. With the parser bound lifted, families complete to 4,096–1,048,576
  levels before the case budget. Evidence JSON is session scratch, not imported.
- rustc's own stack was the remaining ceiling (SIGBUS near 1,000 nested
  blocks). Tsrust now runs rustc/cargo with `RUST_MIN_STACK=1073741824` unless
  the user set it (virtual reservation); 16,384 nested blocks then compile.
  Exported packages document the variable instead of setting Cargo `[env]`,
  which would also change thread stacks under `cargo run`.
- `NestingPolicy::CONSUMERS` is unbounded: expression, block, conditional,
  match, template, type annotation, JSX and destructuring nesting have no limit.
- The CLI and the LSP analysis worker run the compiler on a
  `COMPILER_STACK_BYTES` (1 GiB, virtual) thread, so passes that still recurse
  (for example `in`, checked by synchronous `object_has_own`) reach tens of
  thousands of levels. Tests use `support::check_deep`/`accept_deep`.
- Owner direction (2026-10-30): "match the rust defaults, if rust doesnt have
  limits then we shouldnt either". rustc has no count limit on
  monomorphizations, type parameters, type arguments or record nesting; its
  only default is `recursion_limit = 128`, which bounds polymorphic
  instantiation depth and nested-type trait solving. Removed: callback origin
  specialization counts, per-target callback instance counts, generic
  specialization counts (128/declaration, 1,024/program), closure environment
  complexity, direct record layout depth, explicit type argument and type
  parameter counts. Inline record and callable storage now fail only past
  Rust's maximum object size (`isize::MAX`). Generic instantiation instead
  fails when an argument nests deeper than 128 (`INSTANTIATION_DEPTH_LIMIT`),
  which is how polymorphic recursion (`f<T>` calling `f<Box<T>>`) still gets a
  diagnostic, as in rustc. Callback specializations get the same bound:
  closures inherit the instantiation depth of the body that creates them
  (`specialization_depths`/`closure_depths`), so wrapping the callback in a
  new closure on each recursive call fails at depth 128 instead of growing
  without end (it reached ~30 GB once before the guard existed). Kept,
  matching `recursion_limit`: array rank 128, callable result discovery
  depth 128. Commit `0be17d66`.
- Measured follow-up: `scoped-map-catch` check time is cubic in depth (debug
  compiler: 32 → 0.42 s, 64 → 2.9 s, 80 → 5.5 s) while plain `catch-regions`
  and `cleanup-regions` stay at 0.02 s, so the cost is callback-holding Map
  storage across catch regions in analysis (`map_exception_storage_types`,
  `scoped_map_carriers`, `map_storage_edges` are the candidates). Profiling is
  profiled (Developer Mode enabled 2026-10-30): `finish_block` →
  `record_sequence_carriers` runs every carrier pass over the block's whole
  subtree each time any block finishes, and `scoped_map_entry_epochs` then
  filters every function binding through `contains_scoped_map_storage` for
  each statement (`native_inputs`). Work is blocks × statements × bindings.
  A scalar fast path changed nothing (the bindings are Map/record types). The
  fix is incremental: process each statement's epochs once instead of
  re-walking nested blocks from every enclosing block.
- Known failures before 0.1.0 (also in full run 40909244877, frozen UI):
  `retained_ui_catalog_entries_have_current_compiler_witnesses` and
  `retained_ui_catalog_examples_match_native_output` fail because the
  `UiComputed`/`UiSignal` catalog entries have no witness arm and no in-text
  example. Left as is under the backend-only reset; fix only if UI revives.
- Follow-up (performance, not limits): `scoped-map-catch`, `-cleanup`,
  `-catch-cleanup` time out at 256 levels and `scoped-map-mixed` at 1,024 in
  ownership analysis; `builtin-string-chains`, `captured-local-functions`,
  `field-*-cleanup`, `fresh-generic-adapters` time out at 4,096.
