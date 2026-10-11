# Language primitives and namespace headings audit

Status: the remaining 22 discovered rows are reviewed here. These are language
syntax, types or namespace headings, not 22 runtime functions. Keep language-level
availability; imports would obscure rather than clarify their semantics.

## Per-symbol contracts

| Symbol | Purpose, signature and behavior | Ownership, errors and dependency |
|---|---|---|
| clone | clone(value:T):T; explicit independent clone of supported owned data | Borrows source, recursively copies owned descendants; preserves active union alternatives. Scalar/Copy-only input and non-cloneable views/resources/callables rejected. Static text can remain static. Native clone capability gates copying; no allocation Result promised |
| Result | Result<T,E>; exactly one success/error payload, exhaustively matched | Owned data carries its owners; readonly matching lends selected payload; mutable matching obeys exclusive loans. Missing/unreachable variants diagnosed; emitted tagged representation |
| Ok | Ok(value:T):Result<T,E>; explicit/contextual types required; void payload takes no argument | Transfers payload, no implicit clone or exception; exact type/arity checking; Result construction emission |
| Err | Err(error:E):Result<T,E>; explicit/contextual types required; void payload takes no argument | Transfers error payload; no throw; exact type/arity checking; Result construction emission |
| null | null literal/type, distinct from undefined | Copy absence value; supported unions/fields/arrays; strict guards refine stable places, member/arithmetic misuse rejected |
| undefined | Global missing-value constant/type; may be shadowed by local binding | Copy absence; distinct from unusable void type; lexical use-before-declaration rejected; optional/default contracts use it |
| typeof | Unary expression returning type-name string | Evaluates/observes once; data ownership retained. Bare unknown name yields undefined text; invalid member/call or lexical early use remains an error. Active unions inspected; no dynamic unchecked value produced |
| void | Unary void expression yields undefined; separate void return annotation means no usable value | Evaluates/discards effects once; named owners observed rather than moved; does not suppress type errors. Return-type void cannot be stored as a usable data value |
| Function | Function<R,[Args]> describes callable result and ordered argument contracts; omitted argument list means no arguments | Owned callable by default; signatures carry access, optional/default/rest rules; malformed type lists and incompatible invocation diagnosed; native closure/callback representations |
| Mutable | Mutable<T> access wrapper; data parameter gets exclusive access, callable gets reusable mutable invocation | Borrows owner/excludes conflicts; does not clone or authorize escaping a borrowed environment; nested/misplaced wrappers rejected |
| ReadOnly | ReadOnly<T> access wrapper; readonly data borrow or readonly callable invocation | Owner remains usable outside loan; mutation/transfer/escape conflicts rejected; no hidden retained shared owner |
| Once | Once<Function<...>>; owned single-invocation callable | Invocation may transfer captures; second call rejected; noncallable T invalid; no separate library dependency |
| Owned | Owned<T> argument access; Owned<Function<...>> spells owned callable | Transfers noncopy argument; scalars still copy. Later owner use rejected; one wrapper only; not a shared-reference type |
| Array | Namespace for isArray/of/from; array literals are construction syntax | Reviewed static operations in array slice; general Array constructor unavailable; namespace heading adds no separate runtime |
| Object | Namespace for reviewed is/keys/values/entries/hasOwn operations | No general Object constructor/type; supported data/capability checks still apply; heading itself adds no separate runtime |
| Math | Namespace for reviewed numeric methods/constants | Numeric scalar operations; no first-class namespace object; selected helper families only, heading adds no service dependency |
| console | Namespace for reviewed output/inspection/state methods | Borrowed output values; stdout/stderr and thread-local state as reviewed; inspector methods are explicit no-ops |
| int | Signed 64-bit primitive, -9223372036854775808..9223372036854775807 | Copy; checked native integer contracts, no implicit widening of int variables into number; invalid arithmetic terminates |
| number | IEEE binary64 primitive with NaN, infinities and signed zero | Copy; integer literals can receive number context, distinct from int variables; no owned resource |
| string | Valid UTF-8 text | Owned/borrowed/move rules; static literals may use static storage; byte length/search versus scalar access documented separately; lone surrogate strings unavailable |
| boolean | true/false primitive; conditions require boolean | Copy; no implicit JavaScript truthiness in conditions; explicit Boolean conversion available |
| Tuple.length | Readonly int equal to statically declared tuple arity | Borrows tuple; no resizing; tuple array classification and indexed field ownership remain checked |

All rows retain current spelling and placement. These types/operators do not imply
a JavaScript global object, boxed primitives, garbage collection or shared ownership.
Namespace headings link to the individual member reviews rather than introducing
an additional API contract. Callable call/apply/bind methods are not advertised or
recognized as public builtins; ordinary invocation follows the declared contract.

## Source anchors and reconciliation

Compiler anchors under crates/tsrust-compiler/src:

- type_syntax.rs parses Function arguments and callable/data ownership wrappers,
  enforces one wrapper and limits Once to callable types. analysis.rs resolves
  annotations into concrete access and callback contracts.
- result_types.rs, result_construction.rs and result_matching.rs check payload
  types, inferred/explicit constructors and exhaustive matching. Native emission
  preserves transfer/read/mutate contracts rather than copying selected owners.
- analysis.rs clone dispatch and require_cloneable, plus cloneability.rs and
  native_types.rs metadata, reject unsupported independent copies. Private testing
  callback retention is separately reviewed and does not grant public cloneability.
- analysis.rs observation/typeof/void dispatch and emit.rs implement evaluated
  observations, absence values and primitive representations. Strict narrowing is
  tied to stable storage, not a blanket permission to project arbitrary unions.
- Tuple metadata in checked records and tuple field/length dispatch provide fixed
  arity and readonly length; literals/array conversions retain element ownership.
- crates/tsrust-lsp/src/builtins.json supplies 15 remaining global rows and tuple
  length. intelligence.rs fallback supplies Math, console and four primitive types.
  The void fallback describes its type role in addition to the catalog operator.

No imported alias, external dependency or user linker flag is needed for these
language constructs. Emitted support depends on selected types/operations; helper
family inclusion is not a per-symbol binary-elimination guarantee.

## Acceptance and closure boundary

SQLite `tsrust-api-primitives-*.log` records focused Function wrapper, Result match,
tuple and editor checks. Prior slices retain native examples and ownership evidence
for primitive observations, namespace operations and cloning of native storage.
Whole-project formatting runs before checkpoint. No timing run is claimed.

The explicit coverage manifest reconciles every discovered identity with a review
file. This completes per-row accounting of the current 399-row inventory. It does
not by itself prove discovery exhaustive: final cross-surface source reconciliation
and review of any omissions remain required before declaring the audit closed.
