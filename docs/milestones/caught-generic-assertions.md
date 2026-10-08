# Caught values in generic assertions

The checker creates an internal runtime-error record while discovering exception paths. Previously it added fields and storage but omitted that record from its named type lookup. Generic specialization then reconstructed the inferred type annotation and failed with Unknown or unsupported type "#RuntimeError" instead of checking the assertion.

Register the internally generated record alongside its fields. This exposes no source-language type declaration: the internal name cannot be written as a source type. Existing runtime storage, union alternatives, generic ownership and assertion equality remain unchanged.

A focused project/native witness fails before the repair and passes afterward at optimization levels 0 and 3. Expanded formatted witnesses cover caught integer and string payloads, borrowed generic observations, identity comparison and a runtime bounds error with its documented bounds code. The editor witness checks project analysis, hover and signature help for the original failing call. A Node assertion reference covers scalar payloads; runtime bounds errors retain Tsrust semantics.

Affected exception handling, generic exception contracts and runtime error suites pass. Eight depth canaries are checked because the shared analyzer changed. This is a bounded defect repair, with no new public API, benchmark workload or routine full regression/timing run. Existing family measurements remain immutable. Final source checkpoint, executable refresh and handoff are pending.
