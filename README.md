# Tsrust

**What if TypeScript compiled to Rust?**

Tsrust is a small language that looks and feels like TypeScript but compiles to
native code through Rust. You write familiar syntax (types, arrow functions,
template strings, `map` and `filter`) and get a standalone executable with no
JavaScript runtime inside it. Instead of a garbage collector, the compiler works
out who owns each value and checks that you never use something after giving it
away, much like Rust does, but mostly without making you write the annotations.

```ts
type Pet = {
  name: string;
  age: int;
};

function describe(pet: ReadOnly<Pet>): string {
  return `${pet.name} is ${pet.age}`;
}

function main() {
  const pets: Pet[] = [];
  pets.push({
    name: "Biscuit", age: 3
  });
  pets.push({
    name: "Pickle", age: 7
  });
  const grownUps = pets.filter((pet) => pet.age > 5);
  console.log(pets.map(describe).join(", "));
  console.log(`${grownUps.length} grown-up`);
}
```

Save that as `pets.tsr` and run `tsrust run pets.tsr`:

```text
Biscuit is 3, Pickle is 7
1 grown-up
```

📖 **Documentation:** https://forever-frameworks-llc.github.io/tsrust/

## A friendly warning

This is a hobby project, built for fun and curiosity. **Please don't use it in
production.** The language, the standard library and the tools can all change
without notice, and there are certainly bugs nobody has found yet.

It's also an experiment in a different way of building software: nearly all of
the code, tests and documentation were written by AI coding agents. A human sets
the direction and reviews the results; the agents do the work in small,
milestone-sized steps. Every step has to pass its own checks before it counts:

- a large compiler and editor test suite, run locally and in CI,
- runnable examples in the docs that are executed and compared with their output,
- matched benchmarks against the same programs on Node.js, kept in a SQLite
  database so regressions show up, and
- a written handoff after each milestone so the next agent picks up where the
  last one stopped.

Those guardrails catch a lot, but not everything. Treat Tsrust as something to
play with and learn from, not something to bet a business on.

## What it feels like

- **TypeScript syntax, native results.** `int` and `number` are separate types,
  strings are UTF-8, integer overflow is checked, and the output is an ordinary
  executable.
- **Ownership without the ceremony.** Values are moved or borrowed automatically.
  When you really need a copy, `clone(value)` says so out loud. Mistakes are
  compile errors with explanations, not crashes.
- **Errors you can see.** `try`/`catch` works the way you'd expect, and
  `Result<T, E>` with `match` is there when you want failures as values.
- **Batteries for small backends.** HTTP routing and middleware, JSON and schema
  validation, SQLite and SQL pools, files, async tasks and testing come built in.
  You can generate TypeScript or Rust clients for your API from its description.
- **A real editor experience.** The VS Code extension uses the same compiler for
  diagnostics, completion, hover and formatting.

Tsrust is not a TypeScript replacement: it doesn't run existing TypeScript, Node
packages or frontend frameworks. It's its own small language that borrows
TypeScript's look.

Curious how fast it is? The [performance page](https://forever-frameworks-llc.github.io/tsrust/project/performance/)
compares 930 workloads with the same programs on Node.js. Native wins most of
them, but not all, and the page shows both.

## Try it

Download the archive for your platform from
[Releases](https://github.com/Forever-Frameworks-LLC/tsrust/releases), check it
against `SHA256SUMS`, and put the two programs in `bin/` on your `PATH`. Compiling
programs also needs [Rust](https://rustup.rs) 1.96.1 and a system linker:

```sh
tsrust setup --toolchain --dependencies --yes
tsrust doctor --native
```

Then make something:

```sh
tsrust new hello        # a small console program
cd hello
tsrust run
```

Or try a tiny web service with `tsrust new service --app` and `tsrust dev`. The
[first-service guide](https://forever-frameworks-llc.github.io/tsrust/learn/first-service/)
walks through it step by step. Running `tsrust` on its own opens an interactive
menu.

Prefer building from source?

```sh
git clone https://github.com/Forever-Frameworks-LLC/tsrust.git
cd tsrust
cargo build --locked --release -p tsrust-compiler -p tsrust-lsp --bin tsrust --bin tsrust-lsp
```

The programs land in `target/release/`.

## Learn more

- [Installation](https://forever-frameworks-llc.github.io/tsrust/start/installation/) and [editor setup](https://forever-frameworks-llc.github.io/tsrust/start/editor/)
- [The language](https://forever-frameworks-llc.github.io/tsrust/language/declarations/), chapter by chapter
- [Standard library reference](https://forever-frameworks-llc.github.io/tsrust/library/overview/)
- [Examples](examples/README.md) in this repository
- [Contributing](CONTRIBUTING.md), [developer docs](docs/README.md) and [release process](docs/guides/releases.md)

## License

[MIT](LICENSE), Copyright © 2026 Forever Frameworks LLC. Third-party
dependencies keep their own licenses; their notices ship in the release archives.
