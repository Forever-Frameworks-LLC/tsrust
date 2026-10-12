# JSON scalar extraction: focused acceptance

S10 remains open. This batch accepts std:json.asBoolean, asInt, asNumber and
takeString in compiler source, native execution and editor metadata. Installed
consumers were refreshed from bdedda8a and passed scalar/inspection smoke checks. No timing measurements ran.

The readers return checked Results without coercion or input mutation. asInt
checks signed 64-bit range and fractional values; asNumber rejects precision loss
for integer-backed values. Floating negative zero survives number extraction.
takeString consumes the value on both success and failure, moving string storage
without copying. Private borrowed-string backend support does not publish a
borrowed string API or weaken lifetime requirements.

## Evidence

Canonical records reside in benchmarks/records.sqlite:

- tsrust-json-scalars-complete.log: ten backend checks and three public/native
  checks, including all four executable catalog examples and ownership rejection.
- tsrust-json-scalars-shadow-red.log and tsrust-json-scalars-shadow-green.log:
  failing nested-shadow witness, then four scalar checks and all 18 Result-match
  regressions pass. Block cleanup now removes a name only if it still maps to that
  local slot, preserving outer bindings restored by nested blocks.
- tsrust-json-scalars-editor-2.log: project hover, signature help, ownership docs
  and import completion pass. An earlier single-source editor test used the wrong
  harness for imports; its failure remains retained.
- tsrust-json-scalars-audit.log and tsrust-json-scalars-inventory.json: all 408
  public identities have exactly one placement review.
- tsrust-json-scalars-workload-contract.log: three registration, source guard,
  ABI and independent-oracle checks pass.
- tsrust-json-scalars-workload-smoke-2.log: five native/Node-after-tsc scalar cases
  agree, with zero timing samples. Original failed smoke is retained.
- tsrust-json-scalars-format.log and tsrust-json-scalars-format-final.log:
  whole-project token-preserving formatting; final pass changes zero files.

The scalar workload includes three parses and four checked extractions per
iteration over a fixed exact integer and ASCII string domain. Source hashes,
operation guards, adapters, automatic dependency preparation, result oracle and
report registration are ready. It does not claim parity for large integers or
isolate extraction cost from parsing.

Remaining S10 scope includes dynamic children/construction, direct typed runtime
paths, borrowed/raw adapters, public writers, allocation baselines, full family
regressions and measurements. Completed checks must not be repeated merely to
resume that work.
