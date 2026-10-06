# Core and backend prototypes

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Core proof; Backend proof.

**Scope:** Preserve the first compiler, ownership, and native HTTP demonstration requirements.

**Planning deliverable:** A native core example, compile-time move rejection, explicit cloning, inferred borrowing, then a measured HTTP endpoint.

**Dependencies to coordinate:** [compiler-and-modules](./compiler-and-modules.md), [toolchain-and-development](./toolchain-and-development.md), [benchmarks](./benchmarks.md)

## Current experiment

The original section 52 `User`/`normalize` example now compiles and prints `pat` under the [integer amendment](./milestones/01-integer-subset.md). The [ownership milestone](./milestones/01-ownership-proof.md) records the bounded source and native-execution evidence. Async and HTTP remain unimplemented.

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-52"></a>

## 52. Prototype Phase

After the research phase, build the smallest compiler capable of demonstrating the core thesis.
The first compiler should support approximately:
type User = {
    id: int;
    name: string;
};

function normalize(user: User): string {
    return user.name
        .trim()
        .toLowerCase();
}

function main() {
    const user: User = {
        id: 1,
        name: " Pat ",
    };

    console.log(normalize(user));
}
Must demonstrate:
* parser
* static typing
* object types
* functions
* strings
* method calls
* function argument borrow inference
* Rust generation
* native executable generation
* source-level diagnostics

<a id="core-53"></a>

## 53. Ownership Prototype

Next demonstrate:
const user = getUser();

const backup = user;

console.log(user.name);
Compiler should reject it.
Then:
const backup = clone(user);
should compile.
Then:
log(user);
save(user);
should compile through automatic borrowing.
This prototype is critical.

<a id="core-54"></a>

## 54. Async Backend Prototype

Next build:
const app = new App();

app.get("/hello", async ctx => {
    return ctx.json({
        message: "Hello world",
    });
});

app.listen(3000);
Compile to a native binary.
Measure:
* binary size
* idle memory
* cold startup
* throughput
* latency
* compile time
