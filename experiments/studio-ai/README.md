# Native AI admission

Experimental, compiler-owned native engine adapters called from Tsrust programs.
No model server, Node host or Python training environment is used. This is not
registered `std:ai`, an adapter-training pipeline or a qualified distribution.

Prepare exact CPU dependencies explicitly:

```sh
tsrust bindings resolve experiments/studio-ai/training.bindings.json
tsrust bindings resolve experiments/studio-ai/inference.bindings.json
```

Run the separate suites (the distinct checked dependency closures cannot be
combined into a single Tsrust program):

```sh
tsrust test experiments/studio-ai/training.test.tsr --profile
STUDIO_PROOF_MODEL=/absolute/verified/model.gguf \
STUDIO_PROOF_TOKENIZER=/absolute/verified/tokenizer.json \
  tsrust test experiments/studio-ai/inference-tests --profile
```

The training test checks one scalar Candle autodiff/SGD update against the
independent derivative of squared error, repeated Tsrust-controlled updates and
invalid rates. Scalar state carried across calls is not optimizer checkpoint
persistence. No model fine-tuning or adapter support is implied.

The inference suite requires local verified artifacts and fails when paths are
missing. It checks request-limit rejection and actual greedy CPU generation with
an exact arithmetic oracle. `inference.tsr` is a separate Tsrust worker entry
that prints its buffered result; it does not stream tokens or execute tools.

The inference boundary enforces exploratory limits of 2,048 input and 32 output
tokens. These are admission limits, not Studio's final context or quality gates.
Cancellation is checked between model operations, and owners drop on return.
Hard deadlines and whole-machine memory supervision belong to the invoking
proof run; do not treat these cooperative checks as crash/process isolation.

Model identities, initial memory observations, stop conditions, source references
and honest current acceptance status live in
[the audit](../../docs/plans/tsrust-studio-audit.md). In particular, arithmetic
smoke success will not establish useful Tsrust completion/agent quality.
