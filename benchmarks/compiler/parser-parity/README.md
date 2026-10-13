# Parser parity corpus

This tool compares every parser-facing result of two source revisions. It is
behavior evidence for parser changes. It is not a timing protocol.

`corpus.py --output NEW_DIR` builds a deterministic corpus:

- every tracked `.tsr` file;
- string literals of the Rust tests that contain Tsrust source;
- grammar snippets for each statement, declaration, pattern and type form;
- probes on both sides of each documented nesting and count limit;
- seeded token mutations (delete, duplicate, replace, insert, truncate).

The opt-in unit test `syntax::parity_dump::dump_parser_parity` writes one line
per file. Each line holds a fingerprint of nine observations: the raw program,
the test-entry program, the lowered program, recovered diagnostics for both
entry kinds, formatter output, the standalone type parser, match inspection
and the standalone expression parser with its final parser state.

Each observation parses on an ordinary 2 MiB thread. Only the derived `Debug`
rendering of a finished result uses a larger stack; that renderer is test
tooling.

`dump.py --probe TEST_EXECUTABLE --corpus DIR --output NEW_TSV` runs the dump to
completion. When the process dies, the driver records `ABORT` for the
observation that was running and resumes after it. Keep aborts: a stack
overflow of the older revision is an original failure, not noise.

Build the same dump module into both revisions, run both, then compare the two
files by observation. Explain every difference. Preserve the corpus hash, both
outputs and the abort lists as closed evidence.
