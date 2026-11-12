# Native AI admission

Experimental, compiler-owned native engine adapters called from Tsrust programs.
No model server, Node host or Python training environment is used. This is not
registered `std:ai`, an adapter-training pipeline or a qualified distribution.

Prepare the exact training and selected inference dependencies explicitly:

```sh
tsrust bindings resolve experiments/studio-ai/training.bindings.json
tsrust bindings resolve experiments/studio-ai/llama.bindings.json
```

Run the separate suites (the distinct checked dependency closures cannot be
combined into a single Tsrust program):

```sh
tsrust test experiments/studio-ai/training.test.tsr --profile
CMAKE_BUILD_PARALLEL_LEVEL=1 LLAMA_BUILD_SHARED_LIBS=0 LLAMA_LIB_PROFILE=Release \
STUDIO_PROOF_MODEL=/absolute/verified/model.gguf \
STUDIO_PROOF_TOKENIZER=/absolute/verified/tokenizer.json \
  tsrust test experiments/studio-ai/inference-tests --profile
```

The training test checks one scalar Candle autodiff/SGD update against the
independent derivative of squared error, repeated Tsrust-controlled updates and
invalid rates. Scalar state carried across calls is not optimizer checkpoint
persistence. No model fine-tuning or adapter support is implied.

The ordinary project path alias `@engine` selects `llama.bindings.json`; worker
and tests use the same alias and assertions. Candle's `inference.bindings.json`
remains a reproducible alternative, with its three resource stops in the audit.

The inference suite requires local verified artifacts and fails when paths are
missing. It checks request-limit rejection and actual greedy Metal generation
with an exact arithmetic oracle and strict embedded/external tokenizer agreement.
`inference.tsr` is a separate Tsrust worker entry
that prints its buffered result; it does not stream tokens or execute tools.

The inference boundary enforces exploratory limits of 2,048 input and 32 output
tokens. These are admission limits, not Studio's final context or quality gates.
Cancellation is checked between model operations, and owners drop on return.
Hard deadlines and whole-machine memory supervision belong to the invoking
proof run; do not treat these cooperative checks as crash/process isolation.

The resident candidate API uses `qwen_load` to create an opaque `QwenModel` inside
a Tsrust worker, then `qwen_generate_loaded` to borrow it exclusively for each
request. The model, tokenizer and backend remain loaded until the owner leaves
scope; each request creates a fresh context. Prompts and scheduling stay in
Tsrust. The pinned SDK permits only one backend owner at a time, and warns that
concurrent context creation is unsafe; the exclusive borrow prevents that path.
Model loading itself is not interruptible. Output is still buffered, with the
same exploratory request limits. The resident test checks two distinct answers
and an invalid request between them; its current acceptance is in the handoff.

Streaming moves the model into `qwen_stream_start`; `qwen_stream_next` returns a
complete UTF-8 prefix or waits up to 10 ms on a background worker. One prefix is
queued, so a slow consumer cannot accumulate output. `qwen_stream_cancel` also
interrupts queue backpressure. `qwen_stream_finish` joins and returns the same
model plus success/error; dropping a stream cancels/joins and unloads it. Native
decode and model loading remain cooperative cancellation boundaries.

`model-worker.tsr` supplies the actual Tsrust request loop over bounded blocking
channels. It sleeps while idle, serializes generation, tags updates by request
identity, rejects stale controls, supports targeted cancellation and shuts down
through its caller-owned worker. It performs no tools or workspace writes.
Native thread code only evaluates tokens. Ordinary native cases verify prefix/
final equality, cancellation and invalid-request recovery, abandonment/backend
reload, real worker routing/shutdown and cancellation while blocked idle.
Prefix detokenization is quadratic within the existing 32-token admission limit;
an incremental decoder is required before increasing that limit. This remains
an experimental engine/worker boundary, not the editor's agent or a coding-quality
qualification.

Model identities, initial memory observations, stop conditions, source references
and honest current acceptance status live in
[the audit](../../docs/plans/tsrust-studio-audit.md). In particular, arithmetic
smoke success will not establish useful Tsrust completion/agent quality.

The selected candidate pins llama-cpp-2 and llama-cpp-sys-2 to 0.1.161 through the
checked lock. It compiles a static native library, not a model server. CMake and
Clang are build tools; the successful 27.55 MB native test executable links only
macOS system libraries/frameworks. The upstream sys crate transitively enables
common even with SDK defaults disabled; actual CMake common=ON, server/tools/app
and shared libraries=OFF, OpenMP=OFF, Metal=ON. CMake found Homebrew OpenSSL during
build, but otool finds no OpenSSL or other non-system runtime dependency. Minimal
clean-machine build and portable export still require separate acceptance.

Set CMAKE_BUILD_PARALLEL_LEVEL=1 explicitly: this upstream build script otherwise
chooses all available cores independently of CARGO_BUILD_JOBS. Retain the shared
Cargo target and incremental-off policy. No model/server tool is installed.
The bounded inference gate passed 2/2, including actual output `5` and EOS, with zero
observed swap-usage increase; this is not coding-quality or full-machine acceptance.
