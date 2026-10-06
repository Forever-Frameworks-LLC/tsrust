# Gate 166: interfaces

`interface Name { ... }` declares a record type, the same kind of type as
`type Name = { ... };`. Fields end with `;` or `,`, the last separator and a `;` after
the closing brace are optional, optional fields and type parameters are allowed.

`interface Player extends Named, Positioned { ... }` gives the interface the fields of
each named record type, in written order, followed by its own fields. Bases can be
interfaces or types declared with a body, declared later in the file, or imported.
Record types stay distinct by name: a `Player` is not a `Named`. See docs/interfaces.md.

Rejected: duplicate field among inherited and own fields (`duplicate`), an interface
that extends itself (`recursive-interface`), a base that is not a record type
(`type`), a generic base and method signatures (`unsupported`), a second declaration
of the same name (`duplicate`).

## Implementation

- Parser: `Parser::interface_declaration` builds a `Record` and its base names.
  `Parser::inherit_interface_fields` resolves bases in rounds after all declarations
  are parsed, so order does not matter, and copies base fields in front.
- Projects: the linker keeps the fields of each module's plain record types with
  linked type names (`ImportedRecords`) and passes the imported ones to
  `syntax::parse_module`. A standalone parse of a module cannot see imports, so an
  unknown base is kept in `Program::unresolved_bases`; the analyzer and the linker
  report it. `export interface` is accepted.
- Editor: interfaces are record symbols with fields; inherited fields are added to the
  derived type, including bases declared later. Hover, definition, outline and member
  completion work. Grammar scopes `interface` and `extends`.

## Focused evidence

- Red: target/editor/interfaces-red.log (three failing compiler checks),
  target/editor/interfaces-editor-red.log and target/editor/interfaces-grammar-red.log.
- Compiler: `interfaces` 4 checks: a std:testing fixture (multiple bases, optional
  inherited field, generic interface, nested and array fields, alias-typed field,
  mutation through a base-typed parameter, destructuring), mixed `type`/`interface`
  bases declared later, thirteen static rejections, and a two-module project with an
  imported base and an unknown base. Affected batch: 26 suites, 310 checks, 0
  failures, including all depth canaries (target/editor/interfaces-final-compiler.log).
- The fixture and the guide example pass the pinned Node reference. Guide statements
  are checked in target/editor/interfaces-guide-verification.log.
- Editor: all suites pass, 264 checks (target/editor/interfaces-final-editor.log),
  including `interfaces_have_symbols_fields_and_inherited_members`. Grammar test
  passes (target/editor/interfaces-grammar.log).
- Harness 393/393 on the refreshed consumers (target/editor/interfaces-harness.log).
- Five matched native/node-tsc cases pass untimed; the record
  runtime/results/2026-10-06T14-59-49-220Z-verify.json is imported into SQLite and its
  loose copy removed: interfaces, optional_fields, string_indexing, type_aliases,
  exception_completion.
- Benchmark support: workload pair, independent oracle, source contract with a
  TypeScript syntax-tree check of the three interface declarations and their heritage,
  registration test, manifest row and oracle/run/profile/report hooks.
- Five consumers rebuilt and replaced by atomic rename; hashes in
  target/editor/interfaces-consumers.log.

No full regression or timing was performed. The workload is unmeasured.

## Editor presentation repair, 2026-10-06

The formatter now keeps interface declaration and constructor generic brackets tight,
including `interface Box<T>` and `new Map<string, int>()`. Direct comma-separated
interface fields each receive their own line; inherited interface lists remain on
one line. Nested tuple/function/generic commas retain their ordinary spacing.
Formatting is idempotent and the formatted witness passes compiler checking.

TextMate assigns interface names a type scope and inherited names an inherited-class
scope, including multiline headers. Interface keywords and extends were already
registered in source, but the installed grammar was stale. The repaired local VSIX
was built and installed; Developer: Reload Window activates its contributions.
Existing user documents are preserved and Format Document applies the repair.

All 17 compiler formatting tests, three LSP formatting protocol tests and real
TextMate grammar tokenization pass. Compiler/LSP consumers are atomically refreshed.
No parser/analyzer dispatch, runtime semantics or benchmark sources changed, so no
new timing or runtime workload is claimed. Original red and successful logs are
retained in SQLite under target/editor/interface-* with byte-exact read-back.
