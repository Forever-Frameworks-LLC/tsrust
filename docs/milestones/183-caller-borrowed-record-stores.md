# Gate 183: caller-borrowed record stores

Status: accepted. B7 in the compiler completion ledger.

## Agreed scope

A callback installed through a mutable record parameter may keep a borrow of an input
that the caller owns, provided the caller's owner outlives the stored callback. The
caller cannot change or move that owner while the callback is stored. A callback
cannot borrow a local of the installing function. No implicit clone, shared capture
cell, lifetime cast or new callable kind is used.

The complete status of every form is in
[the acceptance map](b7-caller-borrowed-acceptance-map.md): supported forms,
conservative forms that keep an input borrowed longer than necessary, and forms that
receive a source diagnostic. The plan's acceptance sequence is covered. Abstract
installer invocation, stores into captured records and a held installer with a mutable
captured source are diagnosed, not supported; they are listed as possible later work.

## Final checkpoint

- An installer closure called from a closure that captured it is native: projected
  data lifetimes, one owner input per retained source, owner markers through every
  layer, and owner provenance resolved where each closure was constructed.
- The shared statement and expression visitors now enter `try`, `catch` and `finally`
  bodies and `throw` values. Before, a store inside an exception region had no
  retained-borrow summary: the source check accepted mutation of a borrowed input and
  generated Rust was rejected.
- `installer_value_uses.rs` reports abstract uses of installers as source errors.
- Other defects repaired at this gate are listed in
  `docs/handoffs/handoff-2026-10-10-b7-captured-receiver-native.md` and
  `docs/handoffs/handoff-2026-10-10-b7-accepted.md`.

## Correctness evidence

- Whole compiler crate: 2,702 passed, 0 failed, 3 ignored.
- Whole editor crate: 452 passed, 0 failed.
- Four probe batches of 67 programs, each compared with the pinned Node reference: no
  program reaches "Rust rejected generated code" or a compiler panic.
- Workload `caller_borrowed_record_stores`: 74 observations per iteration with an
  independent arithmetic oracle, source contract and registration controls. Native and
  Node-after-tsc agree on all correctness cases.

## Measurement and review

The workload keeps the original protocol: three processes, eight warmups, 31 samples
per process, execution only. SQLite stores `runtime/results/2026-10-10T00-39-35-029Z.json`
with 186 execution samples. Startup, compilation and allocation were not measured.

| Complete workload batch (2,048 scenarios) | Native median (ms) | Node-after-tsc median (ms) | Node/native |
| --- | ---: | ---: | ---: |
| Caller-borrowed record stores | 0.275958 | 5.002625 | 18.13 |

Native process medians are 0.273666 to 0.280750 ms; Node process medians are 4.243042
to 6.375291 ms. Sample ranges: native 0.268208 to 0.349708 ms, Node 3.549750 to
16.692042 ms. All samples are kept.

The workload has no earlier timing, so there is no comparable baseline and no
regression signal. Native is faster by a wide margin on this pipeline. No optimization
is justified by a demonstrated regression.

## Known debt outside this gate

`node --test benchmarks/runtime/*.test.mjs` reports 49 failures in 441 tests. The same
49 fail at the commit before this gate. They are older pinned-identity and harness
failures and were not changed here.

## Correction (2026-10-10)

The whole-compiler run recorded for this gate was not clean. One or two suites aborted
with a stack overflow and were not counted, because an aborted suite prints no result
line. The cause was stack growth in the expression checker, not a wrong result. It is
repaired at gate 185; see [the gate 185 record](185-project-editor.md).
