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
code at hand-written-Rust parity. The docs restructuring in the next section is
still open but is not the current loop's focus.

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
Latest accepted optimization:readonly small parameter slices,nested rest1.066×
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
   no formatter dependency, arena or storage rewrite is preapproved by this plan.
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
in the latest checkpoint (1.066×,repeat1.046×);see measured sections above. Reprofile residual
string costs before extending storage hints or changing formatting. Destination reuse
is limited to values whose ownership permits it; escaping strings remain distinct
owned outputs. Arena allocation and new formatting dependencies remain conditional,
unapproved design options, not scheduled implementations. The old .413/.755ms
comparison is historical, not a current optimization target.


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
