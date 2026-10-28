# Product vision

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](section-map.md)

## Workstream scope

The current implementation direction is TypeScript-familiar authoring with the efficiency and predictable costs of idiomatic Rust. Prefer standard Rust storage and operations. Document deliberate semantic differences; exact JavaScript compatibility is secondary. Evaluate performance against equivalent idiomatic Rust as well as JavaScript engines, without requiring every operation to beat every engine.

**Status:** Native Rust implementation direction adopted. Broader product proposals and research questions below remain unresolved; the grouping and phase are planning aids, not approved language semantics.

**Proposed phase:** Groundwork.

**Scope:** Define the intended developer experience and differentiated product thesis.

**Planning deliverable:** A concise product thesis with explicit assumptions to validate.

**Dependencies to coordinate:** None.

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-00"></a>

## Tsrust language

Project: TypeScript-Ergonomic Native Language Compiling Through Rust
Core Goal
Design and build a new statically compiled programming language intended primarily for backend and systems-adjacent application development.
The language should preserve the parts of TypeScript/JavaScript that make Node.js productive and familiar while replacing JavaScript’s runtime model with a memory-safe, non-garbage-collected, native execution model.
The language should:
* Look and feel strongly familiar to TypeScript developers.
* Preserve JavaScript-style function declarations, arrow functions, chaining, collection methods, async/await, classes/interfaces where useful, object literals, destructuring, optional chaining, generics, modules, and other ergonomic syntax.
* Be substantially stricter than TypeScript.
* Remove TypeScript and JavaScript escape hatches that undermine static guarantees.
* Avoid a tracing garbage collector.
* Hide Rust implementation details from normal application developers.
* Infer borrowing, ownership behavior, lifetimes, and mutation wherever practical.
* Compile ultimately to efficient native machine code.
* Initially use Rust as the compilation backend and ecosystem substrate.
* Target performance reasonably close to idiomatic Rust for ordinary backend workloads.
* Favor predictable memory use, predictable latency, strong correctness guarantees, and operational simplicity over exact JavaScript compatibility.
* Make backend development feel approximately as approachable as Node/TypeScript while providing a much stronger runtime and type-safety model.
This is not intended to be JavaScript-compatible at all costs.
The priority is:
1. TypeScript syntax and ergonomics
2. No tracing garbage collector
3. Memory safety
4. Strong static typing
5. Automatic ownership/borrowing inference
6. Native binaries
7. Near-native Rust performance
8. Simple package/dependency management
9. Familiar JavaScript-style standard APIs
10. JavaScript semantic compatibility only where it does not conflict with the goals above

<a id="core-01"></a>

## 1. Language Philosophy

The language should expose application-level concepts rather than Rust implementation concepts.
Developers should think in terms of:
* values
* ownership
* sharing
* mutation
* copying
* cloning
* asynchronous work
* concurrency
* interfaces/capabilities
* errors
* optional values
They generally should not need to write or understand:
* &T
* &mut T
* explicit Rust lifetimes
* Rc
* Arc
* RefCell
* Mutex
* borrow checker terminology
* low-level trait-bound syntax
* unsafe memory concepts
The compiler may use those concepts internally.
A central design principle is:
Developers reason about values and capabilities. The compiler reasons about ownership and memory mechanics.
Compiler errors should use this language’s terminology rather than exposing Rust diagnostics directly.
Example:
const user = getUser();
const other = user;

console.log(user.name);
Potential diagnostic:
`user` is no longer available because its value was transferred to `other`.

Use clone(user) to create an independent copy,
or Shared(user) if both locations should share ownership.
Do not surface a raw Rust borrow-checker error to the developer.

<a id="core-60"></a>

## 60. Longer-Term Vision

Potential end-state developer workflow:
lang new api
cd api

lang add hono
lang add zod
lang add postgres

lang dev
Application:
import { App } from "hono";
import { schema } from "zod";
import { db } from "postgres";

const app = new App();

const CreateUser = schema({
    email: string().email(),
    name: string().min(1),
});

app.post("/users", async ctx => {
    const input = CreateUser.parse(await ctx.json());

    const user = await db.users.insert(input);

    return ctx.json(user, 201);
});

app.listen(3000);
Production:
lang build --release
Result:
./dist/api
No:
* Node runtime
* V8
* garbage collector
* per-project node_modules
* JIT warmup
* JavaScript runtime type ambiguity
While retaining a development experience recognizable to TypeScript backend developers.

<a id="core-63"></a>

## 63. Project Principle

Do not build “JavaScript implemented in Rust.”
Build:
A native, statically compiled backend language whose human-facing vocabulary comes from TypeScript and JavaScript, but whose execution model is designed around ownership, deterministic memory management, strong static guarantees, native concurrency, and predictable performance.
The central technical thesis to validate is:
Rust’s ownership discipline may be substantially automatable when the source language is designed around ownership inference from the beginning rather than exposing Rust’s borrowing syntax directly.
The central product thesis to validate is:
Backend developers may be willing to give up JavaScript’s runtime dynamism if they can retain TypeScript’s familiar syntax, APIs, chaining, tooling ergonomics, and rapid development experience while gaining native deployment, stronger correctness guarantees, lower runtime overhead, and more predictable production behavior.
Research both theses critically rather than assuming they are true.

<a id="web-01"></a>

## 1. Current Language Thesis

The project is not merely:
TypeScript that compiles to Rust.
That broad concept already exists in several forms.
The intended project thesis is closer to:
A backend-first native language with TypeScript syntax and JavaScript ergonomics, automatic borrowing and ownership inference, explicit expensive ownership operations, deterministic memory behavior without a tracing garbage collector, and a native ecosystem intentionally designed to make migration from Node/TypeScript practical.
Key goals include:
* TypeScript-like syntax.
* Familiar JavaScript collection and string APIs.
* Function syntax familiar to TypeScript developers.
* Arrow functions.
* Destructuring.
* Generics.
* Interfaces/traits.
* Discriminated unions.
* Async/await.
* JSX-style syntax where useful.
* Strong control-flow narrowing.
* Native executables.
* No V8 dependency.
* No tracing GC.
* No node_modules directory copied into every project.
* No implicit expensive clone behavior merely to satisfy ownership.
* No unrestricted any.
* No arbitrary prototype mutation.
* No eval as a normal language capability.
* No silent runtime type ambiguity.
The guiding memory-management principle is:
Infer mechanics, but do not hide costs.
The compiler may infer:
* immutable borrowing
* mutable borrowing
* moves
* stack placement
* heap placement where appropriate
* lifetimes
* escape behavior
But expensive semantic actions should generally remain visible.
For example:
const user = getUser();

const second = user;

use(user);
should normally fail if user was transferred.
The developer should explicitly request:
const second = clone(user);
for independent ownership, or:
const second = Shared(user);
for shared ownership.
Do not automatically insert arbitrary clones simply to make code compile.

<a id="web-33"></a>

## 33. Competitive Differentiation to Preserve

The project should continue investigating whether these form a distinctive combination:
Automatic borrowing
Developers should rarely annotate borrowing.
Explicit costly duplication
clone() should be visible.
Explicit shared ownership
Shared<T> should be visible.
No tracing GC
Core requirement.
Strict static semantics
No TypeScript-style unsound escape hatches by default.
Familiar TypeScript syntax
Reduce learning cost.
Native JavaScript-style APIs
Preserve chaining and familiar methods.
Backend-first design
HTTP, JSON, validation, databases, HTML, async, and observability matter before browser compatibility.
Hono-compatible native HTTP experience
Make familiar backend code a language-level showcase.
Native JSX server HTML
React-like authoring ergonomics without React’s client runtime model.
npm inspection and migration tooling
Treat ecosystem migration as a compiler/tooling problem.
Content-addressed/global package storage
Avoid duplicated node_modules.

<a id="web-38"></a>

## 38. Guiding Question

For every major design decision, ask:
Does this preserve the part of TypeScript/Node development people actually enjoy while removing runtime behavior they tolerate only because JavaScript requires it?
Do not preserve JavaScript implementation accidents merely because they are familiar.
Preserve ergonomics.
Redesign mechanics.

<a id="desktop-47"></a>

## Expanded Product Vision

If successful, the language would not merely be a Node alternative.
It could become:
A TypeScript-familiar native application language for servers, CLIs, desktop applications, and server-rendered UI, using one type system, one package ecosystem, one compiler, and one deterministic memory model.
The purpose is not to make every application look like a website.
The purpose is to preserve the parts of TypeScript development that developers find productive while replacing JavaScript-specific runtime machinery with native equivalents designed for stronger guarantees and predictable performance.
