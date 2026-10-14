# Match expressions and scalar/record patterns

Implementation checkpoint, 2026-10-14. Family regression and measurement remain
pending until the thread ergonomics section finishes.

- Value arms yield their expression. Block arms exit explicitly; return targets
  the enclosing callable. Ordinary concise arrow functions retain expression returns.
- Strings, integers, numbers, booleans, null and undefined support literal patterns.
  Open scalar types require `_`; finite alternatives require complete coverage.
- Named record unions support nominal variant patterns binding the whole record.
  Result variants continue binding their payload. No new variant constructors.
- Match bodies and yields use the shared checker work queue. A 64-level witness
  passes on the normal test stack. No broader scaling claim.
- Native witnesses cover once-only evaluation, argument order, short-circuiting,
  nested matches, enclosing returns, try/finally, typed throws, owned closures,
  owned array payloads and readonly single-record/union patterns. Ownership and
  exhaustiveness rejection cases stay active. Three readable examples execute
  after formatting.
- Focused compiler acceptance: `match_values` 7/7 and `result_match` 18/18.
  Exact logs, including repaired failures, are retained in SQLite under
  `tsrust-match-*`; final focused record: `tsrust-match-values-accepted5.log`.
- Editor payload hover/navigation and member completion pass focused checks,
  including unrelated syntax errors. Complete editor matrix is still running.
- Matching benchmark has native/TypeScript counterparts, an independent arithmetic
  oracle, fingerprint and mutation guards. Initial untimed verification returns
  checksum 4203521 for both engines. No timing claim yet.
- Whole-project formatting changed four files; second pass changed none. Matching
  benchmark tokens remained identical and fingerprints were refreshed.
