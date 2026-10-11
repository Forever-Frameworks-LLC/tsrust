# Gate 173 prerequisite: owned conditional callable carriers

Concrete owned monomorphic functions with compatible parameter/result contracts
can now be selected without an explicit common annotation. Fresh arrows, saved
aliases, copied scalar captures and consuming factory results retain their native
origins and ownership. Only the selected branch executes. If either alternative
consumes its captured owner, the common carrier permits one invocation.

Two modules separate inferred contract construction from conversion of already
checked environments. Conversion does not evaluate source expressions again or
repeat capture transfers. Borrowed, polymorphic and template-only environments
remain outside this completed prerequisite. Their complete support remains planned.

Native conditional tests pass 11 cases. Four selected original owners dispose
exactly once after success, callback failure and abandonment, at optimization
levels 0 and 3. Unselected branches create no owners. The compiler batch passes
115 tests, including the eight depth canaries and callable field controls; two
existing stored-record native unit controls pass. All 331 editor tests pass,
including inferred signature help, hover and chained results. Grouped initializer
variants also pass focused editor verification.

The generic value adapter workload now retains 129 observations. Five native and
Node-after-tsc cases pass the independent oracle. Updated token guards, source
hashes and registration mutation controls pass. Existing ABI and run/profile/report
hooks remain applicable. No timing was performed and no old measurement is
reclassified. Whole-project formatting changes zero files or fingerprints.

Gate173 remains open. Read ../plans/conditional-callable-carriers-plan.md and the current
handoff for borrowed lifetime carriers, polymorphic replacement, capture-mode
integration, unique/merged instances and recursive convergence requirements.
