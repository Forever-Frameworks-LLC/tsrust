# Milestone 81: legacy String HTML wrappers

Status: implementation and checkpoint 81 accepted. Focused behavior and complete IntelliSense examples pass; combined regression and selected comparisons are sealed.

## Observable contract

| Method | Result structure | Arguments |
|---|---|---|
| anchor | `<a name="attribute">text</a>` | Exactly one string name |
| big | `<big>text</big>` | None |
| blink | `<blink>text</blink>` | None |
| bold | `<b>text</b>` | None |
| fixed | `<tt>text</tt>` | None |
| fontcolor | `<font color="attribute">text</font>` | Exactly one string color |
| fontsize | `<font size="attribute">text</font>` | Exactly one string size |
| italics | `<i>text</i>` | None |
| link | `<a href="attribute">text</a>` | Exactly one string URL |
| small | `<small>text</small>` | None |
| strike | `<strike>text</strike>` | None |
| sub | `<sub>text</sub>` | None |
| sup | `<sup>text</sup>` | None |

The receiver text is inserted unchanged, including empty text, Unicode, NUL and existing markup. The four attributes replace each ASCII double quote with `&quot;`; other characters remain unchanged. These methods construct text only and do not parse or validate HTML, URLs, colors or sizes. They match JavaScript's legacy wrappers for the supported valid-text, string-argument domain. Unlike JavaScript, Tsrust requires the documented argument count and string types, without implicit coercion or omitted-attribute defaults.

Every result is a new owned string. Receiver and attributes remain borrowed and reusable. Evaluate each expression once, receiver before attribute, and preserve the receiver loan during argument evaluation. Preserve UTF-8 content without normalization or UTF-16 indexing.

Reference: [ECMAScript legacy String methods and CreateHTML](https://tc39.es/ecma262/multipage/additional-ecmascript-features-for-web-browsers.html#sec-createhtml). The pinned Node executable also confirms all thirteen tags and attribute escaping on raw Unicode/NUL/markup input.

## Acceptance

Each method needs its own complete built-in entry, consistent hover, signature, parameter help and completion. Execute all new documentation examples with the current compiler at O0/O3. Focused compiler fixtures cover exact tags and escaping, strict arity/types, argument order, loans, returned ownership, chained inference, aggregate/callback contexts and disposal.

Register one equivalent complete-result workload per method. Keep previous source rows and measured records intact. At the checkpoint, run one full workspace regression and the accumulated new/changed performance workloads; retain passed checks and repair affected failures. Measurements and checkpoint acceptance remain pending until real commands finish.

## Focused implementation evidence

Four compiler groups pass at O0/O3, covering all thirteen exact tags, quote-only attributes, raw Unicode/NUL content, empty bodies, strict types, receiver/attribute evaluation order, borrowed inputs, captures/records/arrays, independent owned outputs and normal/unwind disposal. One editor group verifies all thirteen hover/completion/signature contracts and twenty examples with twenty-one exact output lines per optimization level. Existing catalog entries and parallel nullish/console/editor changes remain preserved.

Thirteen comparison sources pass 403 cases, 73 independent literal hands, 208 negative guards and fourteen executed semantic mutants. A pinned Node reference independently confirms every wrapper on raw markup/Unicode/NUL input. The [implementation evidence](../../benchmarks/runtime/diagnostics/2026-10-03T07-15-42-000Z-string-html-wrappers-implementation.json) preserves actual command records and source results. Full regression and the accumulated throughput/allocation checkpoint are pending; no new performance claim is made here.

## Checkpoint 81 acceptance

The combined checkpoint accepts 700 compiler integration groups and 735 workspace groups with retained passes and no full restart. [Runtime and resource evidence](../../benchmarks/runtime/diagnostics/2026-10-03T08-44-00-000Z-checkpoint81-runtime-proof.json) binds 29 selected workloads across eight runtimes and 87 fully disposed allocation samples. [BENCHMARKS.md](../../BENCHMARKS.md) contains actual min/avg/max ops/s. Earlier pending statements describe the implementation-stage queue; this checkpoint closes that queue.
