# Gate 167: scalar string concatenation

String addition and standalone string compound addition accept scalar operands,
including unions containing only string, int, number, boolean, null and undefined.
Conversion follows String(), with once-only left-to-right evaluation and borrowed
sources. Arrays, records and functions remain type errors. Numeric addition and
string result annotations do not independently request conversion. This is defined
concatenation, not coercive comparison. See [String concatenation](../guides/language/string-concatenation.md).

The analyzer wraps the scalar operand in an existing String conversion call before
ordinary argument checking. A separate non-inlined helper keeps state outside the
recursive binary dispatcher. Checked addition result metadata supplies editor hover
and chained methods. String is reserved, so conversion cannot call a shadow binding.

## Focused evidence

Committed red witnesses precede implementation. The std:testing scalar fixture
passes native execution and the independent Node reference. Static rejections cover
arrays, records, callable values, numeric assignments and unsupported string operators.
The affected compiler batch and all eight depth canaries pass (65 checks) in
`target/editor/string-concatenation-final-compiler.log`. The whole editor crate passes (265 checks)
in `target/editor/string-concatenation-editor-final.log`, including mixed addition
hover and string methods.

Matched benchmark support includes seven retained cells, both operand orders,
nullable scalar selection and compound addition. The independent oracle uses
explicit String conversion and concat instead of the operators under test. Fingerprints,
source guards, registration checks and execution/profile/report hooks are ready.
All five native and Node cases pass untimed. The verification record is imported
into benchmarks/records.sqlite, checked by reading it back, and its loose copy removed. No full regression, timing or allocation-parity claim is made.

The named detailed logs are retained as gate-log records in SQLite; the concise
current summary is target/editor/gates-167-168.log. Rebuildable example outputs
and benchmark drivers were pruned after verification.
