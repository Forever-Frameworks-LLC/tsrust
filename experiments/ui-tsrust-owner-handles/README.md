# Tsrust owner/handle feasibility

Identity-enforcement investigation is complete. **B1 is enforced through the
supported source API; full lifetime feasibility remains blocked by B2.** B3 immediate
retirement is adopted; B4 is repaired. This is an experiment, not a production UI runtime.

See [identity mechanism, scope and handoff](IDENTITY.md), [I01–I20](identity-matrix.json),
[lifetime contract](CONTRACT.md), [H matrix](matrix.json), [R matrix and reconciliation](resource-matrix.json)
and [tool provenance](provenance.json).

```sh
TSRUST=/private/tmp/tsrust-gate100-editor-target/debug/tsrust node --test experiments/ui-tsrust-owner-handles/tests/*.test.mjs
TSRUST=/private/tmp/tsrust-gate100-editor-target/debug/tsrust node experiments/ui-tsrust-owner-handles/cli.mjs demo
```

The suite requires the exact compiler hash in provenance.json. Demo runs formatted,
checked, emitted native witnesses and exits **2** for the B2 observation gap. Usage
or unexpected validation failures exit 1. Identity success is reported separately.
A changed compiler requires provenance review and the full focused suite. No binary
is downloaded, rebuilt or replaced by these commands. Only Node built-ins and
installed rustc are used; temporary native programs are removed.

`runtime()` returns an owned allocator; calling it creates an Owner. `root`/`mount`
create handles containing only runtime/owner/instance integers. Foreign handles
reject before access, including root and stale-removal shortcuts. Allocators cannot
be cloned; handles can. Application keys remain labels. Private counters never reset
and checked exhaustion rejects before wrap/reuse. Internal-field tampering, duplicate
library source copies, serialization and cross-thread/process transport are excluded.

The owner retains real callable registrations. Delivery extracts a registration,
invokes it with exclusive owner access, restores it and cleans retired membership.
Direct invocation while still in the registry correctly rejects overlapping access.
Active self/ancestor/root removal immediately ends handle-mediated state access;
sibling removal preserves it. FIFO, fresh replacement IDs, list edits and 1,000 logical
cycles remain tested. Actual release counts/order remain unobserved: empty arrays and
integer handles are not destruction evidence. A real File capture runs but supplies
no calibrated release observer.

Original unsafe allocator witnesses and original B4 named/factory programs are read
from Git b255009e and executed against their original module. Current equivalents
also execute. No historical counterexample is rewritten as a success. The coordinated
[compiler dependency](IDENTITY-DEPENDENCY.md) was separately merged; UI candidate and
evidence await review. The compiler worker approved the necessary B4 fixture caller
migration. Installed consumers remain unchanged.

Closed records live in ignored `evidence.sqlite` and are byte-verified before loose
copies are removed. `UI_EVIDENCE_LOG` records bounded subprocess results for import.
Main SQLite is untouched. No resource observer, renderer, HTTP/UI integration,
production API or next milestone is included. No performance claim is made.
