# Discriminated unions: dedicated compiler family

Status: required, not implemented or accepted. Owner scope decision: 2026-10-09.
This is part of D2 in the compiler completion ledger. Mixed-record unions containing
capturing callbacks are deferred from B5 into this family; they are not excluded.

## Intended behavior

A union of records can use a literal field to identify its active member. Narrowing
that field makes only the selected member's fields available. The checked tag and
payload stay together through storage and transport. Callback captures retain the
existing borrowing or owned-transfer rules, without implicit cloning or shared cells.

Typed validation is a motivating use case: a success member carries a validated
value; a failure member carries issues. A branch on the discriminant exposes the
correct payload. This family supplies language foundations, not a schema library.

## Dependency order and acceptance gates

1. Audit existing general-union representation and literal-type support. Add red
   witnesses for literal string/boolean tags and assignability. Confirm tag spelling,
   widening and mutable-tag rules before implementing dependent narrowing.
2. Implement tag-based narrowing for guards and branches, including invalid member
   access, reassignments and alias invalidation. Preserve source diagnostics and
   check both branches. Add formatter and editor facts alongside compiler changes.
3. Implement tagged record storage: locals, fields, arrays, Maps, parameters and
   results. Test replacement, extraction, readonly views and independently constructed
   values. Keep record nominal identity separate from active union membership.
4. Integrate capturing records. Cover borrowed and owned captures, independent
   environments, conflicts, moves, callback failures, early exits and exact disposal.
   Existing B5 carrier support is reusable evidence, not proof of general-union support.
5. Add validation-shaped success/error examples and exhaustive handling. Coordinate
   typed Result/propagation in B10 rather than introducing competing representations.
6. Complete matched workload support and bounded checks as each gate lands. At the
   family milestone, run complete compiler/editor regressions and matched native
   versus Node-after-tsc timing under the original protocol. Preserve exact SQLite
   evidence and review performance before acceptance.

## Required evidence

Each supported form needs source/native assertions, rejection controls for invalid
access or escape, formatted-source execution and idempotence, and editor diagnostics
and hover facts. Runtime workloads need an independent oracle, TypeScript counterpart,
source contract, hashes and runner/profile/report registration. No diagnostic alone
counts as implementation. No unmeasured workload counts as measured.

## Open design details

The dedicated-family decision does not settle literal widening, writable discriminant
semantics, exhaustiveness syntax or source syntax for schema results. Audit existing
patterns and ask only where observable behavior needs an owner decision. Do not block
B5 acceptance on these later decisions.
