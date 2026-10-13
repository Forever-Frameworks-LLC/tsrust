# Parser nesting discovery

`parser.py` grows parser inputs geometrically. Each case runs the opt-in unit
test `syntax::parser_probe::observe_parser_stages` in a fresh process on the
ordinary test worker stack, without `RUST_MIN_STACK`.

The probe uses the unbounded admission policy for lexing, parsing, tree clone,
disposal and statement recovery. It then observes two consumers: the formatter
and the public bounded parse. `--stages` selects the stages after lexing, so a
slow consumer does not end parser discovery.

Valid families cover each grammar scope alone, then mixed shapes: expressions
inside defaults inside closures inside control-flow bodies; annotations with
generic arguments and defaults; templates, records, arrays, calls and indexes;
every statement form together. Malformed families place one error at the
deepest point, an error in every nested block, and unclosed or extra closers.

Each case records phase events, durations, peak process RSS, source size and
hash, and one classification: `complete`, `operational-timeout`,
`stack-overflow` or `crash`. A timeout is an unfinished observation. Budgets
bound the run; they are not language limits. The durations are single
diagnostic observations of an unoptimized build. They are not formal timing.
