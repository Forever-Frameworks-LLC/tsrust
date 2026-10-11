# Globals and imports audit coverage

Status: discovery is in progress. This file does not accept the exhaustive audit
or authorize skipping its remaining semantic review before JSON globals land.

## Reproducible inventory

Run `cargo run --offline --locked -p tsrust-compiler --example public_api_inventory`
with the documented shared Cargo target and `CARGO_INCREMENTAL=0`. The inventory
joins editor catalog entries to the **compiled** native binding registry, native
type declarations and bundled testing exports. Compiled metadata resolves Rust
struct-update defaults, parameter ownership, alternative argument types, feature
selection and return-borrow contracts without guessing from source snippets.

The initial inventory has 393 distinct group/name rows, including 56 native
bindings, 18 native types and 18 bundled exports. These counts overlap: they are
not counts of independent operations. Every imported catalog entry has a native
binding, native declaration or bundled source declaration. Thirteen private
runtime helpers and the two exported expectation types lack a same-group catalog
entry; expectation members have their own catalog groups. These findings need
explicit review, not automatic classification as missing functionality.

Original inventory and build output are stored in SQLite as
`tsrust-api-audit-inventory.json` and `tsrust-api-audit-build.log`. Every generated
row remains `pending`; presence in this inventory is not semantic verification.

## Coverage still required

| Surface | Discovery evidence | Remaining review |
|---|---|---|
| Native imported functions | Compiled registry, parameter access, results, backend and features | Per-symbol behavior, effects, emission, documentation agreement and disposition |
| Native imported types | Identity lookup, representation, clone/transfer and fields | Construction, capabilities, ownership documentation and disposition |
| Bundled `std:testing` | Export declarations and original source lines | Wrapper behavior, effects, aliases and all private bridge restrictions |
| Receiver methods | Catalog rows | Independent compiler recognition/emission inventory; reconcile omissions and editor-only claims |
| Namespace members | Catalog rows | Independent recognition, shadowing, dependency selection and error semantics |
| Globals and constructors | Catalog rows | Independent compiler inventory including uncataloged language primitives; shadowing and ownership |
| JSON placement | Approved parse/tryParse and serialization policy | Complete preceding review, then implement and verify the approved contracts |

The generated inventory retains complete catalog signatures and documentation as
review input. It deliberately does not infer failure behavior or runtime effects
from names or automatically mark a family reviewed. Each public symbol still needs
the source anchors and reviewed fields specified in the policy. Compiler globals
outside the registry remain a separate discovery pass; the present count is not
proof that every public compiler symbol has been discovered.
