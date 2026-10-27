# Competitive landscape

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Groundwork.

**Scope:** Compare neighboring projects through public evidence without incorporating their implementation code.

**Planning deliverable:** A sourced architecture/feature matrix distinguishing evidence from hypotheses.

**Dependencies to coordinate:** [product-vision](product-vision.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="web-08"></a>

## 8. Competitive Landscape

Research these projects as neighboring approaches.
Do not import or install their source code into this project for benchmarking.
Do not derive implementation code from them.
Use public documentation, published feature descriptions, public benchmark methodology where available, and independently written equivalent benchmark programs.
Important neighboring projects include at least:
RustScript
Very close to the broad TypeScript-to-native-Rust thesis.
Investigate:
* TypeScript syntax coverage
* generated Rust architecture
* ownership handling
* clone insertion
* Rust crate interoperability
* standard library behavior
* async behavior
* compiler architecture
* benchmark claims
* current limitations
Key differentiation to investigate:
RustScript appears willing to insert clones automatically in some ownership situations.
This project should investigate a stricter model where expensive ownership duplication is explicit.
AssemblyScript
Important precedent for:
TypeScript-like syntax does not require JavaScript semantics.
Investigate:
* restricted TypeScript subset
* WebAssembly compilation
* runtime model
* GC/runtime choices
* standard library differences
* developer migration experience
Use it as evidence that familiar syntax can coexist with intentionally different runtime semantics.
Perry
Investigate its approach to:
* native TypeScript/JavaScript compilation
* JavaScript semantic compatibility
* runtime requirements
* garbage collection
* native code generation
Contrast preserving JS behavior with deliberately designing a different native language.
Crisp
Particularly relevant to:
* inference of ownership
* inference of borrowing
* inference of lifetimes
* Rust lowering
Investigate compiler-theory lessons.
Jux
Investigate:
* Java/C#-style surface syntax
* Rust backend
* hidden ownership/lifetime mechanics
* reference-counting strategies
This may provide a useful contrast with this project’s desire to avoid pervasive hidden shared ownership.
Other relevant projects
Continue researching neighboring languages such as:
* Mojo
* Vale
* Roc
* Swift
* Zig
* Go
* Kotlin/Native
* V
* Nim
* Grain
* Inko
* Odin
* Hare
* D
* Crystal
* MoonBit
Not all are direct competitors.
They may nevertheless provide useful comparisons around:
* memory management
* ARC
* borrow checking
* ownership
* compilation speed
* native deployment
* package management
* async runtimes
* backend ergonomics

<a id="web-09"></a>

## 9. Competitive Research Rule

The purpose of competitive research is not to copy implementation.
For each neighboring project document:
What problem does it solve?
What developer does it target?
What does its syntax resemble?
What is its memory model?
Does it use tracing GC?
Does it use ARC/reference counting?
Does it expose ownership?
Does it infer ownership?
Can expensive clones happen implicitly?
What is its async model?
What is its compilation target?
Does it produce native binaries?
How does it handle packages?
How does it interoperate with existing ecosystems?
How mature is it?
Where does its approach conflict with this project's philosophy?
What can be learned without adopting its architecture?
Maintain a competitive matrix.
