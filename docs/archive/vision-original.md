Tsrust language

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

⸻

1. Language Philosophy
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

⸻

2. Core Ownership Model
Research and formally define an ownership model that preserves memory safety without requiring normal developers to annotate function parameters with borrowing syntax.
Initial proposed semantics:
Owned values
Ordinary complex values have a single logical owner.
const user = getUser();
user owns the value.
Assignment
For nontrivially-copyable owned values:
const second = user;
ownership moves to second.
Using user afterward should produce a compile-time error unless the compiler can prove the operation was only a temporary borrow.
Copying
Provide explicit built-in operations.
Potential model:
copy(value)
clone(value)
Suggested distinction:
copy(value)
* Only allowed for trivially copyable values.
* Must not perform hidden allocation.
* Intended for primitive/small value types.
clone(value)
* Creates an independent owned duplicate.
* May allocate.
* Requires a Clone capability/trait.
Do not silently clone expensive values.
Research whether these names are ideal.
Function arguments
Function calls should not inherently move values.
The compiler should inspect usage and infer whether a parameter can be:
* borrowed immutably
* borrowed mutably
* moved
* copied
Example:
function logUser(user: User) {
    console.log(user.name);
}

const user = getUser();

logUser(user);
saveUser(user);
The compiler should be capable of lowering logUser(user) to an immutable borrow internally.
The source-language function syntax should remain simple.
Mutation
The compiler should infer mutable access when a function mutates an argument.
function normalize(user: User) {
    user.name = user.name.trim();
}

normalize(user);
save(user);
Potential internal lowering:
normalize(&mut user);
save(user);
No explicit &mut should normally appear in source code.

⸻

3. Shared Ownership
Treat shared ownership as part of the type system rather than merely as a keyword.
Initial candidate types:
Shared<T>
LocalShared<T>
Weak<T>
Possible semantics:
LocalShared<T> -> single-thread reference-counted ownership
Shared<T>      -> thread-safe reference-counted ownership
Weak<T>        -> non-owning reference into shared ownership
Potential Rust implementation:
LocalShared<T> -> Rc<T>
Shared<T>      -> Arc<T>
Weak<T>        -> Weak<T>
This mapping is implementation detail and should not appear in ordinary language documentation unless discussing compiler internals.
Research:
* Whether Shared<T> should imply thread safety.
* Whether separate LocalShared<T> and Shared<T> types are worthwhile.
* Mutation rules for shared values.
* Whether shared mutation should use explicit wrappers such as:
SharedCell<T>
SharedMap<K, V>
Atomic<T>
* Whether interior mutability should be available at all in general-purpose types.
* How to prevent reference cycles.
* Whether cycle detection is necessary.
* Whether Weak<T> is sufficient.
* Whether some shared structures should use arenas rather than reference counting.

⸻

4. Type System
The type system should be stricter than TypeScript.
Do not preserve TypeScript behaviors solely for compatibility if they weaken static guarantees.
Likely remove or prohibit:
any
Research treatment of:
unknown
never
object
{}
undefined
null
Potential goals:
* No implicit any.
* No unchecked type assertions.
* No arbitrary casts.
* No prototype mutation.
* No runtime property injection on statically shaped objects.
* No implicit coercive equality.
* No widespread truthiness-based type confusion.
* No unchecked indexed access.
* Exhaustive union handling.
* Strong nullability guarantees.
* Sound generic constraints.
Preserve TypeScript-style control-flow narrowing where possible.
Example:
function getUser(): User | null {
    ...
}

const user = getUser();

if (user) {
    console.log(user.name);
}
The compiler should narrow user to User inside the branch.

⸻

5. Primitive Types
Do not automatically inherit JavaScript’s primitive model.
Research and decide:
Numbers
Should:
number
mean:
* f64, JavaScript-style
* inferred integer/float
* compiler-selected numeric representation
* an alias for a default integer or floating type
Potential explicit types:
i8
i16
i32
i64
u8
u16
u32
u64
f32
f64
int
uint
number
Determine whether this is too much complexity for the target audience.
Investigate:
* overflow semantics
* integer division
* checked arithmetic
* decimal/money support
* bigint support
* literal inference
Strings
Research:
* UTF-8 behavior
* indexing semantics
* substring slicing
* grapheme versus byte versus Unicode scalar indexing
* immutable vs mutable string types
* whether string methods should mimic JavaScript even when JavaScript behavior is inefficient

⸻

6. Objects, Structs, and Data Shapes
Object syntax should feel like TypeScript:
type User = {
    id: number;
    name: string;
};

const user: User = {
    id: 1,
    name: "Pat",
};
But object shapes should normally be statically fixed.
This should likely fail:
user.randomField = 123;
Use maps/dictionaries for dynamic key-value structures.
Research distinction between:
* record/struct types
* classes
* maps
* anonymous object types
* tuples
* tagged unions
* enums
Determine whether JavaScript-style classes are desirable or whether classes should be syntactic sugar over data + traits + methods.

⸻

7. Traits / Interfaces / Capabilities
Create a capability system inspired by Rust traits but expressed through familiar TypeScript syntax.
Potential syntax:
trait Display {
    toString(): string;
}

class User implements Display {
    name: string;

    toString(): string {
        return this.name;
    }
}
Generic constraints might resemble:
function print<T extends Display>(value: T) {
    console.log(value.toString());
}
Static dispatch should be preferred where possible.
Research a separate dynamic-dispatch type such as:
Trait<Display>
Potential semantics:
function print(value: Trait<Display>) {
    console.log(value.toString());
}
which may internally use a trait object.
Research whether this syntax is intuitive enough.
Potential compiler-known traits/capabilities:
Copy
Clone
Equal
Hash
Display
Debug
Serialize
Deserialize
Send
Sync
Iterator
IntoIterator
Error
Default
Determine which should be language primitives versus ordinary standard-library traits.

⸻

8. Collection Model
Preserve familiar JavaScript methods and chaining where practical:
users
    .filter(user => user.active)
    .map(user => user.email)
    .take(100);
Do not necessarily preserve JavaScript execution mechanics.
The compiler should be free to implement this as:
* lazy iterators
* fused loops
* stack allocation
* optimized native loops
* SIMD where possible
Research whether:
array.map()
returns a realized collection, as in JavaScript, or whether chains are lazy by default.
Potential compromise:
array.map(...)
returns an array.
array.iter()
    .filter(...)
    .map(...)
is lazy.
However, additional syntax may reduce JavaScript familiarity.
Explore whether the compiler can automatically fuse eager-looking pipelines without observable semantic differences.
Core APIs to evaluate:
map
filter
reduce
find
findIndex
some
every
includes
slice
splice
sort
reverse
flat
flatMap
forEach
join
push
pop
shift
unshift
For every familiar JavaScript API, explicitly determine whether preserving exact semantics would harm performance, safety, or predictability.

⸻

9. Async and Concurrency
Preserve:
async
await
Promise<T>
at the language surface if practical.
Example:
async function getUser(id: string): Promise<User> {
    return await db.users.find(id);
}
Research:
* whether Promise<T> should really be named Promise<T> or something more runtime-neutral
* structured concurrency
* task cancellation
* async drop/destructors
* executor/runtime selection
* work stealing
* blocking vs nonblocking APIs
* async streams
* backpressure
* timers
* channels
* task-local storage
Do not blindly reproduce JavaScript’s event-loop semantics.
Determine which Node behaviors developers actually rely on versus which should be intentionally replaced.

⸻

10. Error Handling
This needs substantial research.
Possible mechanisms:
Result types
Result<T, E>
Throw/catch syntax backed by Result
Allow:
try {
    const user = await getUser();
} catch (error) {
    ...
}
while compiling ordinary recoverable errors into explicit result types.
Potential ergonomic propagation operator:
const user = try getUser();
or:
const user = getUser()?;
But ? is Rust-like and may violate the design goal of hiding Rust vocabulary.
Research:
* checked versus unchecked errors
* fatal/panic conditions
* stack traces
* error causes
* error wrapping
* async errors
* automatic propagation
* interoperability with Rust libraries
Goal: retain Node-like ease without making error behavior implicit or unpredictable.

⸻

11. Memory Allocation Strategy
A major goal is avoiding garbage collection without making application developers perform manual memory management.
Research:
* stack allocation
* escape analysis
* region/arena allocation
* heap allocation
* deterministic destruction
* reference counting
* temporary allocation
* bump allocators for request scopes
* allocator customization
* memory pools
* zero-copy APIs
Investigate whether HTTP request processing can benefit from request-scoped arenas.
Example:
incoming request
    ↓
request arena
    ↓
temporary parsing / validation / routing values
    ↓
response
    ↓
entire arena released
This could provide excellent backend performance while reducing ownership complexity.
Do not assume Rust’s default allocation patterns are automatically ideal for the language.

⸻

12. Compiler Architecture
Initial architecture:
Source
  ↓
Lexer / Parser
  ↓
AST
  ↓
Name Resolution
  ↓
Typed AST
  ↓
Ownership / Escape Analysis
  ↓
Language IR
  ↓
Rust-oriented IR
  ↓
Generated Rust
  ↓
rustc
  ↓
Native binary
Initially, generating Rust source is acceptable and likely desirable.
Benefits:
* leverage rustc
* leverage LLVM
* leverage platform support
* leverage mature native code generation
* simplify compiler bootstrap
Do not make direct LLVM generation an early requirement.
Research the point at which a custom backend would become beneficial.
The compiler should retain source maps or equivalent metadata so compiler/runtime errors refer to original source files rather than generated Rust.

⸻

13. Rust Interoperability
This may become critical to ecosystem development.
Investigate a controlled FFI system that allows Rust libraries to be wrapped.
Potential syntax:
extern rust package "serde";
or build-time bindings.
Research:
* automatic trait mapping
* ownership mapping
* async interoperability
* generic interoperability
* macros
* unsafe Rust boundaries
* lifetime-heavy APIs
* procedural macros
* C ABI fallback
* build scripts
A major design goal should be:
Rust can act as the native ecosystem beneath the language without forcing ordinary users to write Rust.

⸻

14. Standard Library
Initial standard library should provide familiar JavaScript-style APIs where sensible.
Major categories:
Core
Array
Map
Set
String
Math
Date / DateTime
Regex
JSON
URL
UUID
Result
Option
Iterator
Range
Buffer / Bytes
Filesystem
Node-like ergonomics:
await fs.readFile(path);
await fs.writeFile(path, data);
Networking
fetch()
Request
Response
Headers
WebSocket
TCP
UDP
TLS
Processes
process.env
process.args
process.exit()
spawn()
Consider preserving familiar naming where it is useful without pretending a Node runtime exists.
Crypto
Native safe cryptographic APIs.
Testing
Testing should be built into the toolchain.
Serialization
First-class JSON support.
Potential later formats:
MessagePack
CBOR
Protobuf
YAML
TOML

⸻

15. Backend-First Framework Strategy
A later project stage should port or recreate popular JavaScript/TypeScript libraries around language-native primitives.
Priority candidates:
* Hono
* Zod
* Express-style routing concepts
* Fastify concepts
* tRPC concepts
* Prisma-like database ergonomics
* Drizzle-style typed SQL
* date utilities
* JWT libraries
* validation libraries
* logging libraries
* testing frameworks
Do not necessarily line-by-line port these libraries.
Instead:
1. Identify why developers like the library.
2. Identify its public API and ergonomic conventions.
3. Reimplement those concepts natively.
4. Remove runtime behaviors that are unnecessary in a statically compiled language.
5. Take advantage of compile-time generation where possible.
Example:
Zod exists partly because TypeScript’s compile-time types disappear at runtime.
A native version could potentially derive validation metadata directly from types:
const UserSchema = schema<User>();
or:
@validate
type User = {
    email: Email;
    age: int >= 18;
};
Research whether the language can eliminate entire classes of runtime JavaScript tooling through compiler support.

⸻

16. NPM Compatibility / Conversion Layer
Investigate a tool capable of consuming npm packages and determining whether they can be used, translated, wrapped, or rejected.
Potential workflow:
lang add npm:zod
The tool should analyze the package and classify it.
Possible classifications:
GREEN
Can be translated automatically.

YELLOW
Mostly compatible but requires changes.

RED
Depends on unsupported dynamic JavaScript behavior.
The analyzer should identify problem areas such as:
* eval
* prototype mutation
* dynamic require
* runtime code generation
* unrestricted any
* monkey patching
* dynamic object shapes
* reflection-heavy behavior
* unsupported Node native modules
* JS Proxy usage
* weak-reference assumptions
* runtime polymorphism incompatible with static lowering
* cyclic mutable object graphs
* hidden shared ownership
* dependence on V8 behavior
* package postinstall scripts
* WASM/native extensions
Potential command:
lang inspect npm:zod
Example output:
Compatibility: 87%

Compatible:
✓ Pure TypeScript type definitions
✓ Static classes
✓ Standard collection usage
✓ Promise-based async

Needs adaptation:
! Dynamic property construction in parser.ts
! Runtime schema registry
! instanceof behavior

Unsupported:
✗ eval() usage in legacy.ts
Potential assisted conversion:
lang convert npm:zod
The converter should:
1. Download source.
2. Parse JS/TS.
3. Run compatibility analysis.
4. Translate compatible code.
5. Mark incompatible regions.
6. Suggest semantic replacements.
7. Generate tests.
8. Preserve upstream attribution/license information.
9. Produce a local package.
10. Record upstream version information.
Research how much automatic conversion is realistic.
Do not assume arbitrary npm packages can be automatically translated.

⸻

17. Package Management
Do not reproduce node_modules.
The package manager should use a global or content-addressed package store.
Potential structure:
~/.language/packages/
    sha256/...
Projects contain only:
project.lang.json
project.lock
src/
Packages should be globally cached and referenced by version/hash.
Example manifest:
{
  "name": "my-api",
  "version": "1.0.0",
  "dependencies": {
    "hono": "4.2.1",
    "zod": "3.24.0"
  }
}
No package code should need to be copied into every project.
Research models from:
* pnpm
* Cargo
* Go modules
* Nix
* Bun
* Deno
* Zig
* modern package content-addressable stores
Goals:
* reproducible installs
* deterministic builds
* minimal disk duplication
* lockfiles
* dependency graph inspection
* straightforward version pinning
* offline builds
* package integrity verification
* package signing/provenance where possible

⸻

18. Dependency Versioning
Avoid npm’s dependency-chaos failure modes.
Research whether the package system should allow multiple versions of the same dependency in one binary.
Potential behavior:
App
├── framework A -> parser 2.0
└── library B   -> parser 1.7
Possible approaches:
* allow both versions
* deduplicate compatible versions
* force single version
* namespaced dependency identities
Investigate consequences for:
* binary size
* shared types
* trait identity
* dynamic loading
* security patching

⸻

19. Package Registry
Eventually define a first-party registry.
Potential requirements:
* immutable published versions
* signed packages
* checksums
* dependency metadata
* source availability
* license declaration
* native-code declaration
* unsafe-code declaration
* package health indicators
* compiler-version compatibility
* security advisories
* yank/deprecate functionality
Research whether packages should publish source, intermediate representation, or compiled artifacts.

⸻

20. Build System
One unified CLI.
Potential commands:
lang new
lang run
lang build
lang test
lang check
lang fmt
lang lint
lang add
lang remove
lang update
lang inspect
lang convert
lang doctor
Potential production build:
lang build --release
Output:
dist/my-api
A native executable.
Research:
* cross-compilation
* static linking
* musl
* Windows
* macOS
* Linux
* ARM
* Docker
* WASI/WASM
* debug symbols
* PGO
* LTO

⸻

21. Development Experience
Developer experience is a first-class product goal.
Provide:
* formatter
* linter
* language server
* autocomplete
* rename/refactor
* go-to-definition
* inline types
* dependency diagnostics
* ownership visualization
* compiler explanations
* native debugger integration
* profiler integration
Optional ownership-debug command:
lang inspect ownership
Potential output:
UserService.getUser

config      borrowed
request     borrowed mutable
user        owned
cache       shared
response    moved into return
This should be educational, not required during normal development.

⸻

22. Compiler Diagnostics
Compiler errors should be a major differentiator.
Errors should:
* use source-language concepts
* explain why
* point to both cause and consequence
* suggest likely fixes
* avoid leaking generated Rust details
* provide deeper explanation via optional flags
Example:
error: value already transferred

12 | const backup = user;
                    ---- value transferred here

15 | console.log(user.name);
                   ^^^^ `user` is no longer available

Try one of:

const backup = clone(user);   // independent value
const backup = Shared(user);  // shared ownership
Potential advanced command:
lang explain E1042

⸻

23. Unsafe Code
Decide whether the source language permits unsafe code at all.
Possible model:
* ordinary language has no unsafe memory operations
* unsafe/native extensions require Rust
* unsafe packages must declare that fact
* package tooling surfaces unsafe dependencies
Example:
Dependency safety report:

32 packages
29 safe-language only
2 Rust-backed safe wrappers
1 contains unsafe Rust
This could provide a significant security advantage.

⸻

24. Reflection and Runtime Type Information
Research carefully.
Questions:
* Are types available at runtime?
* Can developers inspect field names?
* Can frameworks automatically serialize objects?
* Can decorators/annotations attach metadata?
* Is reflection compile-time generated?
* Should runtime reflection be opt-in?
Avoid carrying heavyweight type metadata into every binary unless needed.
Potential syntax:
@reflect
type User = {
    id: int;
    email: string;
};

⸻

25. Decorators / Attributes / Metadata
TypeScript developers increasingly use decorators.
Research a safer compile-time equivalent.
Example:
@route("/users/:id")
async function getUser(id: string): Promise<User> {}
This could generate routing tables at compile time.
Potential uses:
* serialization
* validation
* database mappings
* dependency injection
* HTTP routes
* CLI parsing
* documentation generation
Avoid runtime decorator machinery where compile-time expansion works.

⸻

26. Macros and Code Generation
Research whether macros are necessary.
Potential goals:
* derive implementations
* schema generation
* SQL compile-time checking
* route generation
* serialization
* FFI bindings
Avoid exposing Rust-style procedural macro complexity unless absolutely necessary.
Potential simpler model:
derive(Serialize, Clone)
type User = {
    ...
};
or annotations.

⸻

27. Database Layer
Backend adoption will require excellent database support.
Research native integrations for:
* PostgreSQL
* MySQL
* SQLite
* Redis
* MongoDB
Potential typed SQL:
const user = await db.query<User>`
    SELECT id, name, email
    FROM users
    WHERE id = ${id}
`;
Compiler could validate:
* parameter types
* returned columns
* nullable fields
Research offline schema snapshots versus live database introspection.

⸻

28. HTTP Framework
Eventually create a language-native framework inspired by Hono’s simplicity.
Target experience:
const app = new App();

app.get("/users/:id", async ctx => {
    const user = await users.find(ctx.params.id);

    if (!user) {
        return ctx.json({ error: "Not found" }, 404);
    }

    return ctx.json(user);
});

app.listen(3000);
Desired characteristics:
* tiny core
* low allocation
* async
* middleware
* typed context
* typed routes
* request validation
* WebSocket support
* streaming
* HTTP/1.1
* HTTP/2
* eventually HTTP/3
Research compile-time route generation.

⸻

29. Validation Framework
Build a Zod-like API, but investigate compile-time integration.
Potential syntax:
const CreateUser = schema({
    email: string().email(),
    age: int().min(18),
});
Potential stronger language-native approach:
type CreateUser = {
    email: Email;
    age: int where age >= 18;
};
Research:
* refinement types
* branded/opaque types
* runtime validation generation
* JSON parsing
* error reporting
* schema export
* OpenAPI generation

⸻

30. Serialization
Investigate whether types can automatically generate serializers.
Example:
const json = JSON.stringify(user);
could compile to specialized serialization code without reflection.
Likewise:
const user = JSON.parse<User>(body);
should validate the incoming data.
Never make unchecked JSON parsing silently produce typed values.

⸻

31. Module System
Prefer a simple static module system.
Potential syntax:
import { User } from "./user";
import { App } from "http";
Research:
* filesystem conventions
* package imports
* barrel files
* circular dependencies
* tree shaking
* conditional compilation
* platform-specific modules
Avoid CommonJS-style dynamic loading as a core mechanism.

⸻

32. Global Runtime Model
Explicitly research what runtime the language actually needs.
Questions:
* Is there a global async executor?
* Who initializes it?
* Is there thread-local state?
* Is there a runtime allocator?
* How large is a minimal binary?
* Can synchronous programs avoid linking async components?
* Can applications opt into different executors?
* How much runtime machinery is acceptable?
The ideal is a very small runtime rather than “Node rewritten in Rust.”

⸻

33. Startup and Deployment
Target:
lang build --release
producing one deployable native executable where practical.
Investigate:
* static binaries
* dynamic linking
* CA certificates
* timezone database
* native dependencies
* database drivers
* TLS implementations
Goal:
COPY app /app
ENTRYPOINT ["/app"]
with no language runtime installation required.

⸻

34. Debugging and Stack Traces
Ensure native compilation does not destroy developer ergonomics.
Research:
* source-level stack traces
* async stack traces
* local variable inspection
* panic/crash reporting
* mapping optimized native frames back to source
* VS Code debugging
* LLDB/GDB integration

⸻

35. Performance Philosophy
Avoid promising universal “Rust performance.”
Instead define performance goals.
Benchmark:
* HTTP requests/sec
* p50/p95/p99 latency
* idle RAM
* peak RAM
* startup time
* JSON serialization
* JSON parsing
* database throughput
* string processing
* CPU-bound workloads
* async task overhead
* shared-state contention
Compare against:
* Node.js
* Bun
* Deno
* Go
* Rust
* Java/Kotlin where relevant
Measure generated code rather than relying on assumptions.

⸻

36. Security Model
Research security as a first-class design dimension.
Potential advantages:
* no arbitrary eval
* no dynamic prototype mutation
* immutable package versions
* deterministic builds
* limited install scripts
* safe native APIs
* auditable unsafe dependencies
Investigate sandboxed build scripts if build scripts exist at all.

⸻

37. NPM Supply-Chain Improvements
Do not copy npm’s postinstall model without evaluating its security implications.
Potential rules:
* packages cannot execute arbitrary code during installation by default
* build scripts require explicit permission
* native extensions are declared
* network access during builds can be disabled
* dependency capabilities are visible
Potential manifest:
{
  "permissions": {
    "build": false,
    "network": false,
    "filesystem": ["./generated"]
  }
}

⸻

38. Toolchain Version Management
Avoid Python/Node-style environment confusion.
Potential model:
lang toolchain install 1.4
lang toolchain use 1.4
Project pins:
language = "1.4.2"
Research compatibility policies:
* semantic versioning
* editions
* compiler channels
* stable/beta/nightly
Rust editions are worth studying.

⸻

39. Foreign Function Interface
Eventually support:
* Rust
* C
* possibly C++
Potential:
extern "C" function sqlite3_open(...);
But ordinary users should consume safe wrapper packages instead.
Research automatic header/binding generation.

⸻

40. WebAssembly
Investigate WASM as a secondary target.
Potential uses:
* edge runtimes
* serverless
* plugins
* browser compute modules
* sandboxed extensions
Determine whether language semantics translate cleanly to WASM without creating a second runtime model.

⸻

41. Generics Strategy
Research:
* monomorphization
* code bloat
* dynamic dispatch
* generic constraints
* variance
* higher-kinded types
* associated types
* generic specialization
Do not automatically add advanced type-system features unless they solve concrete application problems.

⸻

42. Closures
Closures are one of the hardest ownership areas.
Research:
* captured values
* mutable captures
* escaping closures
* shared captures
* async closures
* recursive closures
The compiler should infer capture strategy wherever possible.
Only require explicit developer intervention when semantics are genuinely ambiguous.

⸻

43. Cyclic Data Structures
Explicitly investigate graphs, trees with parent references, ORM entity graphs, caches, and observer patterns.
Examples:
parent.children.push(child);
child.parent = parent;
Determine supported patterns without tracing GC.
Possible techniques:
* weak references
* arenas
* handles/IDs
* specialized graph containers
* explicit Shared/Weak
* region lifetimes
Do not hand-wave this problem.

⸻

44. ORM and Object Graphs
Many backend frameworks rely heavily on mutable cyclic object graphs.
Research whether the ecosystem should encourage:
* immutable records
* IDs rather than object references
* data-oriented design
* explicit graph containers
Determine what ergonomic cost this creates for developers coming from Node.

⸻

45. Global State
Define safe global initialization.
Potential syntax:
const CONFIG = lazy(() => loadConfig());
Research:
* thread-safe initialization
* mutable globals
* test isolation
* dependency injection
* application state

⸻

46. Hot Reloading
Node developers expect fast development loops.
Investigate:
lang dev
with:
* incremental compilation
* automatic restart
* state preservation where possible
* fast diagnostics
Hot module replacement for backend applications may not be necessary initially.
Compile speed matters greatly to adoption.

⸻

47. Incremental Compilation
This should be researched early.
A native compiled language that takes 30 seconds after every edit will fail the Node-like developer-experience goal.
Investigate:
* module-level incremental compilation
* cached intermediate representation
* rustc incremental mode
* parallel compilation
* dependency artifact caching
Set compile-time benchmarks early.

⸻

48. Standard Library Stability
Establish a compatibility policy early.
Research:
* stable API guarantees
* experimental modules
* deprecation periods
* language editions
* standard-library versioning

⸻

49. Source Compatibility
Decide how closely valid TypeScript should resemble valid source in this language.
Potential goal:
A developer should be able to paste ordinary strict TypeScript business logic and have most syntax parse, even when semantic changes are required.
Do not promise arbitrary TypeScript compatibility.
Potential compatibility levels:
Syntax-compatible
API-compatible
Semantically compatible
Runtime-compatible
Clearly distinguish them.

⸻

50. Initial Non-Goals
Suggested early non-goals:
* browser DOM compatibility
* full npm compatibility
* full Node API compatibility
* dynamic eval
* JavaScript prototype semantics
* JIT compilation
* runtime monkey patching
* arbitrary native addons
* reflection-heavy frameworks
* frontend UI framework support
* exact JavaScript numerical semantics
* exact JavaScript event-loop semantics
These can be revisited later.

⸻

51. Research Phase Deliverables
Before implementing the full language, produce a research/design package covering:
1. Language philosophy.
2. Formal ownership semantics.
3. Function argument inference.
4. Copy/clone/share semantics.
5. Shared ownership model.
6. Type system.
7. Trait/capability system.
8. Object model.
9. Closure model.
10. Async runtime.
11. Error model.
12. Collection semantics.
13. String semantics.
14. Numeric semantics.
15. Memory allocation strategy.
16. Cyclic data structures.
17. Reflection strategy.
18. Serialization strategy.
19. Rust interoperability.
20. Package management.
21. npm conversion feasibility.
22. compiler architecture.
23. debugging architecture.
24. security model.
25. incremental compilation.
26. performance targets.
27. native deployment model.
28. standard library boundaries.
For each topic provide:
Problem
Why it matters
Possible approaches
Advantages
Disadvantages
Performance impact
Developer-experience impact
Implementation complexity
Interaction with other language features
Recommended initial direction
Open questions
Prototype required? yes/no
Do not prematurely resolve questions where experimentation is necessary.

⸻

52. Prototype Phase
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

⸻

53. Ownership Prototype
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

⸻

54. Async Backend Prototype
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

⸻

55. Package Manager Prototype
Implement:
lang add package
lang remove package
lang install
lang update
Use a global content-addressed store.
Demonstrate two projects sharing the same physical package files.

⸻

56. JavaScript Package Analyzer Prototype
Implement:
lang inspect npm:<package>
Initially analyze TypeScript/JavaScript ASTs and flag:
* eval
* Proxy
* prototype writes
* dynamic require
* dynamic imports
* native addons
* postinstall scripts
* runtime property injection
* unsupported Node APIs
Do not attempt conversion yet.
First prove that useful compatibility analysis is possible.

⸻

57. First Native Ecosystem Packages
After the core language stabilizes, build first-party/native equivalents inspired by:
1. Hono
2. Zod
3. dotenv/config
4. logging
5. PostgreSQL client
6. Redis client
7. JWT
8. test runner
9. mocking
10. date/time utilities
Focus on API ergonomics rather than literal source compatibility.

⸻

58. NPM-Assisted Porting System
Later build:
lang port npm:hono
Pipeline:
npm package
   ↓
dependency analysis
   ↓
AST compatibility analysis
   ↓
automatic translation
   ↓
unsupported-pattern report
   ↓
suggested rewrites
   ↓
generated native package
   ↓
test comparison
Eventually support maintaining a relationship to upstream:
lang port update hono
The tool should identify upstream changes and re-run conversion rather than requiring complete manual re-porting.
Investigate semantic diffing.

⸻

59. Package Port Metadata
Store something like:
{
  "source": "npm:hono",
  "sourceVersion": "4.10.0",
  "portVersion": "1.2.0",
  "compatibility": {
    "automatic": 81,
    "manual": 14,
    "unsupported": 5
  }
}
Track manually adapted sections so upgrades do not overwrite them blindly.

⸻

60. Longer-Term Vision
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

⸻

61. Questions That Must Remain Open During Groundwork
Do not force answers to these prematurely:
* How much ownership can realistically be inferred?
* When should inference stop and require explicit source syntax?
* What is the correct shared-ownership model?
* How should closures capture shared/mutable state?
* How should cyclic graphs work?
* Should arrays have value or shared semantics?
* Should strings be immutable?
* What should number mean?
* Should null and undefined both exist?
* Should Promise<T> exist as a source-level concept?
* Should async functions be Send by default?
* How should runtime reflection work?
* Should decorators exist?
* How should dynamic trait dispatch look?
* Should classes exist as a fundamental concept?
* Should inheritance exist?
* Should interfaces have runtime representations?
* Should exception syntax exist?
* How should cancellation work?
* How should database drivers expose borrowed data?
* Should request-scoped arenas be first-class?
* How much npm conversion is realistically possible?
* Should generated Rust remain inspectable?
* Is Rust-source generation viable long-term?
* How should native Rust dependencies be sandboxed?
* How should package versions coexist?
* How should compile times be kept competitive with Node development?
* How much JavaScript method compatibility is worth preserving?
* Which JavaScript behaviors should be intentionally rejected?
* What compiler optimizations are required to meet the performance thesis?
* What language features would make Rust lowering significantly harder?
* Which design choices could accidentally reintroduce GC-like overhead through pervasive reference counting?
These questions are part of the project, not obstacles to be hidden.

⸻

62. Decision Rule
Whenever choosing between JavaScript compatibility and the language’s foundational goals, evaluate in this order:
Does it preserve memory safety?
Does it avoid tracing GC?
Can ownership remain understandable?
Can the compiler optimize it effectively?
Does it preserve predictable performance?
Can developers understand the behavior?
Does it retain familiar TypeScript ergonomics?
Can it interoperate cleanly with the ecosystem?
Exact JavaScript compatibility comes after these concerns.

⸻

63. Project Principle
Do not build “JavaScript implemented in Rust.”
Build:
A native, statically compiled backend language whose human-facing vocabulary comes from TypeScript and JavaScript, but whose execution model is designed around ownership, deterministic memory management, strong static guarantees, native concurrency, and predictable performance.
The central technical thesis to validate is:
Rust’s ownership discipline may be substantially automatable when the source language is designed around ownership inference from the beginning rather than exposing Rust’s borrowing syntax directly.
The central product thesis to validate is:
Backend developers may be willing to give up JavaScript’s runtime dynamism if they can retain TypeScript’s familiar syntax, APIs, chaining, tooling ergonomics, and rapid development experience while gaining native deployment, stronger correctness guarantees, lower runtime overhead, and more predictable production behavior.
Research both theses critically rather than assuming they are true.

Project Workstream: Identity, Competitive Positioning, Benchmarking, and Native Web Experience
This workstream follows the initial language-design proposal for a new backend-first, statically compiled language with TypeScript/JavaScript ergonomics, Rust-backed native compilation, strong static guarantees, inferred ownership, no tracing garbage collector, and explicit cost semantics.
The immediate goals are:
1. Select a distinctive and defensible language name.
2. Select a source-file extension.
3. Select a CLI/toolchain command.
4. Select names for the compiler, package manager, registry, and related ecosystem components if separate names are needed.
5. Research neighboring languages and transpilers to understand what already exists.
6. Clearly identify which ideas are established versus which aspects of this project should intentionally differ.
7. Establish a benchmark methodology against neighboring languages without incorporating, vendoring, or installing their source code into this project.
8. Investigate making a Hono-compatible HTTP framework/API a core built-in part of the language’s backend experience.
9. Investigate JSX/TSX-style native server-side HTML generation as a first-class feature.

⸻

1. Current Language Thesis
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

⸻

2. Identity Research
Perform a systematic naming search before committing to branding.
Do not evaluate names solely based on whether the exact .com domain is available.
Evaluate the entire developer namespace.
For every candidate name, investigate:
* existing programming languages
* transpilers
* compilers
* interpreters
* developer platforms
* package managers
* build systems
* CLIs
* GitHub organizations/projects
* npm packages
* crates.io packages
* PyPI packages
* Homebrew formulas
* VS Code extensions
* JetBrains plugins
* Docker images
* domain usage
* software companies
* relevant U.S. trademarks
* relevant international trademark concerns if obvious
* common Unix executable collisions
* search-engine ambiguity
Pay particular attention to exact executable collisions.
A language name may be acceptable while its ideal CLI command is not.

⸻

3. Identity Components Should Be Evaluated Separately
Do not assume the following must be identical:
Language name
Source extension
CLI executable
Compiler command
Package manager command
Registry name
Website/domain
GitHub organization
Existing languages regularly separate these.
Examples conceptually include:
TypeScript
.ts
tsc
and:
Rust
.rs
rustc
cargo
Therefore evaluate combinations independently.
A hypothetical result could be:
Language:        ExampleScript
Files:           .tsr
Toolchain CLI:   ex
Compiler:        exc
Registry:        Example Packages
if that produces a cleaner namespace.

⸻

4. Source Extension Research
The .tsr extension remains interesting because it visually communicates the project’s lineage:
.ts   TypeScript
.rs   Rust
.tsr  TypeScript ergonomics + Rust/native execution
It can also be remembered informally as:
TS + R
Do not assume .tsr must be retained, but treat it as a strong candidate.
Research:
* existing significant .tsr development usage
* editor language associations
* MIME-type issues
* OS/file-association conflicts
* searchability
* syntax-highlighting support feasibility
* GitHub Linguist implications
* package ecosystem implications
Minor legacy file-extension collisions are acceptable if there is no meaningful modern developer conflict.

⸻

5. CLI Research
The CLI should be short, memorable, and unlikely to collide with existing developer tools.
Potential desired experience:
<cli> new api
<cli> dev
<cli> run
<cli> build
<cli> check
<cli> test
<cli> fmt
<cli> lint
<cli> add postgres
<cli> remove postgres
<cli> update
<cli> inspect
<cli> doctor
Potential npm migration commands:
<cli> npm inspect hono
<cli> npm inspect zod
<cli> npm port package-name
Do not choose a three-letter CLI merely because it matches the extension if that executable is already meaningfully occupied.
CLI uniqueness matters more than perfect naming symmetry.

⸻

6. Naming Directions to Explore
Generate candidates across several conceptual families.
Trust / certainty
Concepts:
* trust
* certainty
* assurance
* verification
* truth
* correctness
* reliability
Structural integrity
Concepts:
* truss
* brace
* beam
* frame
* rivet
* anchor
* foundation
* lattice
* keel
* structure
Avoid names already strongly occupied in developer tooling.
Compiler concepts
Concepts:
* bound
* invariant
* type
* static
* proof
* verify
* compile
* native
* exact
* sound
Material / strength metaphors
Concepts:
* alloy
* iron
* steel
* forged
* tempered
* solid
Script-derived names
Investigate distinctive:
_____Script
names because this immediately communicates familiarity to TypeScript/JavaScript developers.
However, avoid names that imply this is merely a scripting runtime if the language actually produces native executables.
Investigate whether a Script suffix helps onboarding enough to outweigh that ambiguity.

⸻

7. Previously Investigated Names
Do not restart research from zero.
Names already identified as problematic include:
Trust
Problems:
* multiple developer/compiler uses
* TrustLang already occupies related programming-language territory
* generic software/security terminology
* trademark noise
The conceptual theme may remain useful even if the literal name does not.
TrustLang
Avoid because an existing language already uses it.
trustc
Avoid due to existing compiler/tool usage.
Truss
Conceptually strong, but already used by multiple active developer products and CLIs.
Avoid as the primary CLI.
RustScript
Avoid.
A current project already occupies essentially this exact name and significant parts of the same technical positioning.
tsr
CLI
Potential collision with existing developer tooling.
Do not assume .tsr files imply tsr should be the executable.
Lattice
Avoid because an existing programming language uses the name and overlaps conceptually.
Keystone
Highly crowded software namespace.
Forge
Extremely crowded development-tool/compiler namespace.
Verity
Already used in programming/formal-verification contexts.
NativeScript
Already a major development framework.

⸻

8. Competitive Landscape
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

⸻

9. Competitive Research Rule
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

⸻

10. Benchmarking Philosophy
Do not benchmark by downloading competing projects’ benchmark repositories and incorporating their code into this repository.
Instead define language-neutral workload specifications.
Then independently implement those workloads in each language.
Example specification:
Accept an HTTP POST containing a JSON array of 100 users, validate required fields, normalize names, calculate a score, serialize a JSON response, and return HTTP 200.
Implement equivalent versions in:
* this language
* Rust
* Node.js
* Bun
* Deno
* Go
* relevant TypeScript-to-native competitors where practical
If a competitor can be executed through its normal public toolchain, it may eventually be benchmarked externally, but its source implementation should remain separate from this project’s source tree.
Do not vendor competitor code.

⸻

11. Benchmark Categories
Measure at minimum:
HTTP
* plaintext
* JSON
* routing
* path parameters
* query parameters
* middleware
* JSON validation
* server-rendered HTML
* streaming response
* WebSocket where supported
Runtime
* startup time
* idle RSS
* peak RSS
* binary size
* executable startup latency
CPU
* string transformation
* JSON parsing
* JSON serialization
* hashing
* sorting
* collection pipelines
* regex
Memory
* allocation count
* bytes allocated
* peak heap
* reference-count operations if measurable
* clone count
* temporary allocation
Async
* task creation
* task switching
* timer workload
* concurrent HTTP requests
* database-style simulated I/O
* channels
Tail latency
Always measure:
p50
p90
p95
p99
p99.9 where practical
Average throughput alone is insufficient.
A major project thesis is predictable latency without GC.

⸻

12. Benchmark Against Rust Fairly
Do not expect to consistently outperform carefully written Rust.
Rust should function as an approximate performance ceiling/reference.
Measure:
this language vs idiomatic Rust
and report the gap honestly.
Potential performance target:
Ordinary backend code should remain within a practically acceptable range of idiomatic Rust while requiring significantly less ownership-specific syntax.
Do not establish an arbitrary percentage until representative prototypes exist.

⸻

13. Benchmark Against Node Fairly
Node is highly optimized and should not be treated as slow by definition.
Compare:
* CPU-heavy work
* high-concurrency I/O
* startup
* memory footprint
* p99 latency
* multithreaded workloads
* long-running server behavior
The proposition is not simply:
native is faster.
It is:
native compilation + deterministic memory + stronger static guarantees may offer better operational predictability while preserving familiar development ergonomics.

⸻

14. Hono as a Core Design Target
Investigate making a Hono-compatible HTTP programming model part of the language’s standard backend experience.
This should be considered much earlier than an ordinary library port.
Hono is attractive because its developer API is:
* small
* composable
* TypeScript-friendly
* middleware-oriented
* standards-oriented
* familiar
* portable
* JSX-capable
The target experience might intentionally resemble:
import { Hono } from "http";

const app = new Hono();

app.get("/", ctx => {
    return ctx.text("Hello");
});

app.get("/users/:id", async ctx => {
    const id = ctx.req.param("id");
    const user = await users.find(id);

    if (!user) {
        return ctx.json({ error: "Not found" }, 404);
    }

    return ctx.json(user);
});

app.listen(3000);
Research whether the public API can be made substantially source-compatible with common Hono applications.

⸻

15. Hono Compatibility Goal
Investigate three possible compatibility levels.
Level 1 — Familiarity
The API feels like Hono but is not source-compatible.
Level 2 — API compatibility
Common Hono source code can be moved with minimal modifications.
Level 3 — Near-source compatibility
A useful subset of ordinary Hono server applications can compile after changing imports/package metadata.
Determine which level is technically and legally appropriate.
Do not require JavaScript runtime behavior merely to preserve compatibility.

⸻

16. Independent Hono-Compatible Implementation
Do not make the language runtime dependent on executing Hono’s JavaScript implementation.
Instead investigate implementing equivalent concepts directly using native language/runtime primitives.
Potential internal architecture:
Hono-like API
    ↓
compile-time route graph
    ↓
native middleware pipeline
    ↓
typed request/response primitives
    ↓
native HTTP runtime
Potential benefits:
* static route construction
* lower allocation
* no dynamic middleware lookup where unnecessary
* compile-time parameter extraction
* optimized middleware composition
* typed context state
* route-specific validation
* dead-code elimination
The surface can remain familiar even when the implementation differs completely.

⸻

17. Licensing and Naming
Before claiming literal Hono compatibility or using Hono branding:
* review Hono’s current license
* review trademark usage
* determine requirements for attribution
* distinguish API compatibility from official affiliation
* avoid implying endorsement
API compatibility and independent implementation should be treated separately from copying implementation or documentation.
If Hono’s license permits reuse, still decide whether clean independent implementation provides a healthier compiler/runtime architecture.

⸻

18. HTTP as a Standard Capability
Consider whether HTTP should be part of the standard library rather than requiring an ecosystem package.
Potential built-ins:
Request
Response
Headers
URL
URLSearchParams
Cookie
FormData
WebSocket
fetch
Using Web-standard APIs may make migration easier.
Then the Hono-compatible router can be extremely thin.
Example:
const app = new App();

app.get("/", ctx => ctx.text("Hello"));
could compile directly to the native HTTP server.

⸻

19. JSX / TSX as a First-Class Backend Feature
Hono provides an important second test:
JSX-like server-rendered HTML.
Investigate supporting:
function Layout(props: { children: Html }) {
    return (
        <html>
            <body>
                {props.children}
            </body>
        </html>
    );
}
and:
app.get("/", ctx => {
    return ctx.html(
        <Layout>
            <h1>Hello</h1>
        </Layout>
    );
});
without requiring React.
This should be treated as:
native structured HTML syntax
rather than emulating React’s runtime.

⸻

20. JSX Compilation Strategy
Investigate compiling JSX directly into optimized response-generation code.
For example:
<h1>Hello {user.name}</h1>
could conceptually lower to:
write static "<h1>Hello "
escape user.name
write static "</h1>"
rather than creating an intermediate virtual DOM.
Potential advantages:
* very low allocation
* streaming
* automatic escaping
* compile-time HTML checks
* predictable memory behavior

⸻

21. HTML Type Safety
Potential built-in type:
Html
Distinguish:
string
from:
Html
so raw user strings cannot accidentally be treated as trusted markup.
Potential helpers:
html(...)
escapeHtml(...)
unsafeHtml(...)
unsafeHtml() should be explicit.
This could provide stronger XSS guarantees than ordinary string-based templating.

⸻

22. JSX Components Without React Semantics
Do not automatically reproduce:
* React state model
* client reconciliation
* hooks
* virtual DOM
* hydration
Server-side components could simply be functions:
function UserCard(user: User): Html {
    return (
        <article>
            <h2>{user.name}</h2>
            <p>{user.email}</p>
        </article>
    );
}
This retains React/JSX familiarity without introducing a React runtime.

⸻

23. Streaming HTML
Investigate:
return ctx.streamHtml(
    <Page>
        <AsyncComponent />
    </Page>
);
Potentially compile async HTML components into native streaming output.
This could become a powerful backend feature without recreating React Server Components wholesale.

⸻

24. Hono as a Compiler Stress Test
Use a Hono-style framework as one of the earliest serious compiler workloads.
It exercises:
* generics
* closures
* async
* middleware
* function composition
* route parameters
* string handling
* maps
* context propagation
* shared application state
* error handling
* JSON serialization
* HTML generation
* traits
* lifetime inference
* concurrency
If these patterns become awkward, that is useful evidence that the language design needs revision.

⸻

25. Zod as the Second Compiler Stress Test
After Hono, use Zod’s developer experience as another compatibility target.
Do not necessarily reproduce its runtime implementation.
Investigate:
const User = z.object({
    email: z.string().email(),
    age: z.number().int().min(18),
});
versus deeper compiler integration.
The important benchmark is:
Can the language provide at least the same validation ergonomics while exploiting static type information that JavaScript cannot?

⸻

26. Hono + Validation Integration
Eventually target something like:
app.post(
    "/users",
    validate.json(CreateUser),
    async ctx => {
        const input = ctx.valid("json");

        const user = await db.users.insert(input);

        return ctx.json(user, 201);
    }
);
The compiler should know the type of input.
No runtime casts.
No separate hand-written interface if the schema already determines the type.

⸻

27. Potential Compile-Time Route Type System
Investigate whether:
app.get("/users/:id", ctx => {
    const id = ctx.req.param("id");
});
can infer:
id: string
from the route literal.
Potentially:
app.get("/users/:id<int>", ...)
could infer:
id: int
Research whether such syntax helps enough to justify diverging from Hono compatibility.
Compatibility should not prevent optional native enhancements.

⸻

28. Native Web Standard Library Vision
Potential standard modules:
http
html
json
url
websocket
crypto
fs
process
stream
test
Hono-style routing may sit directly in:
import { App } from "http";
rather than being a third-party dependency.
Alternatively:
import { Hono } from "hono";
could remain available as a compatibility package layered over the same primitives.
Research both approaches.

⸻

29. Migration Experience
An ideal experiment:
Take a small real Hono TypeScript application.
Do not directly compile its dependencies.
Port only its application source.
Measure:
Lines unchanged
Lines changed
Imports changed
Unsupported constructs
Semantic changes required
Performance
Memory usage
Binary size
A useful project KPI might eventually be:
Percentage of ordinary Hono application code that transfers unchanged.
Do not establish a target until experimentation.

⸻

30. Compatibility Test Corpus
Build original compatibility tests around documented Hono behavior.
Categories:
* routing
* route precedence
* path parameters
* query parameters
* headers
* cookies
* middleware ordering
* context variables
* error handling
* JSON
* text
* HTML
* redirects
* streaming
* JSX
* nested routers
Do not blindly copy Hono’s internal source tests.
Write independent tests describing observable public behavior.

⸻

31. Benchmark Web Workload
Create a canonical web benchmark implemented independently across languages.
Example endpoints:
GET /plaintext
GET /json
GET /user/:id
POST /validate
GET /html
GET /stream
POST /validate should:
1. Parse JSON.
2. Validate a structured object.
3. Normalize strings.
4. Perform collection transformations.
5. Serialize a typed response.
GET /html should:
1. Render nested JSX-style components.
2. Escape dynamic strings.
3. Loop through typed data.
4. Produce HTML.
This benchmark can later compare:
* this language
* Rust/Axum
* Node/Hono
* Bun/Hono
* Deno/Hono
* Go
* RustScript if practical
* other relevant native languages

⸻

32. Never Optimize Solely for Benchmarks
Benchmark workloads must guide engineering rather than define semantics.
Reject changes that make microbenchmarks faster while making ordinary application code harder to reason about.
Primary order:
correctness
safety
predictability
developer ergonomics
performance
benchmark score

⸻

33. Competitive Differentiation to Preserve
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

⸻

34. Research Deliverables
Produce four documents before finalizing the language name.
Document A — Identity Matrix
For every viable name:
Name
Meaning
Pronunciation
Language collisions
CLI collisions
Package collisions
Trademark concerns
Domains
GitHub namespace
Suggested extension
Suggested CLI
Strengths
Weaknesses
Recommendation
Document B — Competitive Matrix
Compare this project against:
* RustScript
* AssemblyScript
* Perry
* Crisp
* Jux
* Rust
* Go
* Node
* Bun
* Deno
* other relevant languages discovered during research
Do not rank them globally.
Compare concrete architecture/features.
Document C — Benchmark Specification
Define workloads independently of any implementation.
Document D — Hono Compatibility Feasibility
Determine:
* API surface worth preserving
* behavior worth preserving
* behavior that should intentionally differ
* JSX feasibility
* middleware model
* native implementation architecture
* legal/licensing considerations
* migration experiments required

⸻

35. Naming Should Follow Technical Positioning
Do not finalize the name before answering:
What exactly makes this language different from RustScript and the other neighboring projects?
The final name should reflect the actual differentiated product rather than the initial generic idea.
Possible positioning themes may ultimately emphasize:
* predictable native backend development
* explicit-cost ownership
* compiler trust
* structural correctness
* TypeScript familiarity
* native web development

⸻

36. Final Naming Output
After completing research, return a shortlist of no more than ten serious candidates.
For each candidate provide:
Language name
Pronunciation
Meaning/story
File extension
CLI executable
Compiler executable if separate
Manifest filename
Lockfile filename
Potential registry name
Collision findings
Trademark/search concerns
Why it fits this language specifically
Then identify the strongest 3 candidates for additional legal/domain clearance.
Do not choose names merely because they sound good.
They must work as developer infrastructure.

⸻

37. Core Web Vision
One potential long-term differentiator deserves explicit investigation:
The language should feel like Hono, TypeScript, and server-side JSX were designed from the beginning for native compilation rather than layered on top of JavaScript.
A possible experience:
import { App } from "http";
import { z } from "validation";

const app = new App();

const UserInput = z.object({
    name: z.string().min(1),
    email: z.email(),
});

function UserPage(user: User): Html {
    return (
        <main>
            <h1>{user.name}</h1>
            <p>{user.email}</p>
        </main>
    );
}

app.post("/users", async ctx => {
    const input = await ctx.req.json(UserInput);

    const user = await db.users.create(input);

    return ctx.html(<UserPage {...user} />);
});

app.listen(3000);
The developer experience looks familiar.
The compiler/runtime underneath should be fundamentally native.
Potential internal behavior:
typed route table
+
compile-time schema
+
native JSON parser
+
ownership inference
+
native async runtime
+
zero/low-copy HTTP
+
specialized serialization
+
streaming HTML writer
+
no V8
+
no tracing GC
This is a more meaningful target than simply translating TypeScript syntax into Rust.

⸻

38. Guiding Question
For every major design decision, ask:
Does this preserve the part of TypeScript/Node development people actually enjoy while removing runtime behavior they tolerate only because JavaScript requires it?
Do not preserve JavaScript implementation accidents merely because they are familiar.
Preserve ergonomics.
Redesign mechanics.

Project Addendum: Native Desktop UI and Fine-Grained Reactive Components
Add native desktop application development as a first-class research target for the language.
Because the language already compiles to native binaries, investigate whether it can provide a built-in cross-platform desktop UI system that feels familiar to TypeScript/React developers without relying on:
* Electron
* Chromium
* Node.js
* browser DOM
* virtual DOM diffing
* JavaScript runtime
* garbage collection
The goal is not to reproduce React internally.
The goal is:
React/JSX-like authoring ergonomics with native rendering, deterministic lifetimes, fine-grained reactive updates, and no full-screen/component-tree rerender when a single value changes.

⸻

Native UI Product Goal
Target an experience approximately like:
import {
    Window,
    Stack,
    Text,
    Button,
    signal,
} from "ui";

function Counter(): View {
    const count = signal(0);

    return (
        <Window
            title="Counter"
            width={400}
            height={240}
        >
            <Stack>
                <Text>{count}</Text>

                <Button onClick={() => count++}>
                    Increment
                </Button>
            </Stack>
        </Window>
    );
}

run(<Counter />);
This should compile to a native executable.
On Windows, for example:
counter.exe
There should be no embedded Node runtime or browser engine merely to render application UI.

⸻

Core Reactive Principle
Do not default to React-style runtime reconciliation.
Avoid:
state changes
    ↓
component function reruns
    ↓
new virtual tree generated
    ↓
old/new trees diffed
    ↓
native UI patched
Prefer:
state changes
    ↓
dependency graph identifies affected bindings
    ↓
specific native property updates
For:
<Text>{count}</Text>
the compiler/runtime should understand that the text control depends on count.
When:
count++;
occurs, only the relevant text value should need updating.
This should be investigated as a fine-grained reactive system rather than a virtual-DOM framework.

⸻

Reactive Primitives
Research a minimal built-in reactive model.
Potential primitives:
signal<T>()
computed<T>()
effect()
Example:
function Profile(user: User): View {
    const fullName = computed(
        () => `${user.firstName} ${user.lastName}`
    );

    return (
        <Stack>
            <Text>{fullName}</Text>
        </Stack>
    );
}
The compiler should ideally understand the dependency:
user.firstName ─┐
                ├─> fullName ─> Text.text
user.lastName ──┘
Research whether dependency tracking should be:
* compile-time where possible
* runtime where necessary
* hybrid
Favor compile-time knowledge where it improves performance and diagnostics without making normal code cumbersome.

⸻

Components
Components should remain simple functions.
Example:
function UserCard(user: User): View {
    return (
        <Card>
            <Text>{user.name}</Text>
            <Text>{user.email}</Text>
        </Card>
    );
}
Do not require classes for ordinary UI components.
Research component semantics around:
* props
* local state
* shared state
* lifecycle
* async work
* cleanup
* child ownership
* conditional rendering
* lists
* event handlers
* error boundaries
* suspense/loading equivalents

⸻

Native UI Ownership Model
Desktop UI provides an opportunity to align application structure naturally with the language’s ownership model.
A visual tree commonly resembles:
Window
├── Sidebar
│   ├── Button
│   └── Button
└── Content
    └── UserList
Investigate parent-owned child lifetimes.
Potential semantic model:
Window owns Sidebar and Content
Sidebar owns its Buttons
Content owns UserList
When the parent is destroyed, owned children are deterministically destroyed.
This may significantly reduce the need for shared ownership.
Use:
Shared<T>
only when state genuinely needs multiple owners.
Example:
const session = Shared(new Session());

run(
    <App session={session} />
);
Research whether the UI framework can make ownership particularly intuitive because component trees already have explicit structure.

⸻

No Hidden Shared Ownership by Default
Do not automatically place every component, event handler, and state object inside reference-counted shared containers.
Prefer:
1. normal ownership
2. inferred borrowing
3. scoped references
4. explicit shared ownership when necessary
This is important to preserve predictable performance and memory behavior.

⸻

Native Platform Strategy
Research two broad rendering architectures.
Option A — Native Platform Controls
Map common UI primitives onto platform controls.
Possible targets:
Windows:
WinUI / Windows App SDK / related native APIs

macOS:
AppKit and/or native platform APIs

Linux:
GTK or another mature native toolkit
Example source:
<Button>Save</Button>
could map to the appropriate native button control for the target platform.
Advantages:
* native accessibility
* correct keyboard behavior
* system text input
* IME support
* screen-reader integration
* system themes
* native menus
* high-DPI handling
* familiar platform behavior
Disadvantages:
* platform differences
* potentially inconsistent layout/rendering
* backend abstraction complexity
Research whether this should be the initial approach.

⸻

Option B — Custom Cross-Platform Renderer
Investigate a later renderer using a GPU/native graphics layer.
Possible underlying technologies could include concepts similar to:
* Skia
* wgpu
* Direct2D/DirectWrite
* Metal
* Vulkan
Do not commit to a specific implementation during groundwork.
Advantages:
* identical visual behavior across platforms
* advanced custom UI
* predictable rendering
* animation flexibility
Disadvantages:
The project would potentially become responsible for:
* text rendering
* font shaping
* accessibility
* keyboard navigation
* focus
* selection
* scrolling
* input methods
* menus
* high-DPI behavior
* screen readers
* native drag/drop
* clipboard integration
Because this is a major engineering commitment, research native-control-first development before implementing a custom renderer.

⸻

Shared JSX Syntax
Investigate JSX as a common authoring syntax for both:
* server-rendered HTML
* native desktop UI
But maintain distinct output types.
Example server component:
function UserPage(user: User): Html {
    return (
        <main>
            <h1>{user.name}</h1>
        </main>
    );
}
Example native component:
function UserPage(user: User): View {
    return (
        <Stack>
            <Text size="title">{user.name}</Text>
        </Stack>
    );
}
The shared ideas are:
* components
* props
* composition
* conditional rendering
* loops
* expressions
* reactive values
The renderer and type system remain different.

⸻

Strongly Typed Rendering Targets
Create distinct types such as:
Html
View
This should fail:
function Page(): Html {
    return <Window />;
}
because:
Window -> View
not:
Window -> Html
Similarly, browser-specific tags should not accidentally appear in native UI unless explicitly supported.
This prevents JSX from becoming an untyped generic tree syntax.

⸻

Compile JSX Directly
Research whether JSX can compile directly into imperative creation/binding operations.
For example:
function Counter(): View {
    const count = signal(0);

    return (
        <Stack>
            <Text>{count}</Text>
            <Button onClick={() => count++}>
                +
            </Button>
        </Stack>
    );
}
could conceptually lower to:
create Stack
create Text
create Button

attach Text to Stack
attach Button to Stack

bind Text.text -> count

bind Button.click -> increment count
After initial construction, updating count should not require reconstructing the entire component tree.

⸻

Conditional UI
Support familiar syntax:
{user
    ? <UserProfile user={user} />
    : <LoginView />
}
Research how conditional branches affect ownership and component lifetimes.
If user changes:
LoginView
may need to be deterministically destroyed and:
UserProfile
created.
The runtime should update the smallest meaningful subtree.

⸻

Lists
Support familiar list rendering:
<List>
    {users.map(user => (
        <UserRow user={user} />
    ))}
</List>
Investigate stable identity.
A React-style key concept may still be useful:
{users.map(user => (
    <UserRow key={user.id} user={user} />
))}
Research whether compiler-known collection identity can reduce the need for explicit keys.
Do not blindly reproduce React behavior.

⸻

Styling
Research a native styling system that feels approachable to web developers without pretending native UI is CSS.
Potential direction:
<Stack
    direction="vertical"
    gap={12}
    padding={16}
>
and:
<Text
    size={20}
    weight="bold"
>
Possible reusable styles:
const cardStyle = style({
    padding: 16,
    radius: 8,
});
Research:
* CSS-like syntax
* typed style objects
* design tokens
* platform-native theme values
* light/dark mode
* responsive layouts
* DPI scaling
Avoid implementing the entirety of browser CSS unless there is a compelling reason.

⸻

Layout
Investigate a predictable cross-platform layout model.
Potential primitives:
<Stack />
<Row />
<Column />
<Grid />
<Scroll />
<Spacer />
Potential flexible layout:
<Row gap={8}>
    <Sidebar width={240} />
    <Content grow />
</Row>
Research whether a Flexbox-inspired model is appropriate.
A familiar web-like layout model could significantly reduce desktop UI learning cost.

⸻

Built-In Windows
Native windows should be simple enough for a beginner to create without framework setup.
Potential:
run(
    <Window
        title="My Application"
        width={1200}
        height={800}
    >
        <App />
    </Window>
);
Support eventual features such as:
multiple windows
dialogs
modal windows
menus
system tray
notifications
file pickers
folder pickers
clipboard
drag and drop
fullscreen
window state
multiple monitors

⸻

Lower-Level Window API
Also investigate a lower-level imperative API.
Example:
const window = new Window({
    title: "My Application",
    width: 1200,
    height: 800,
});

window.render(<App />);
window.show();
Declarative and imperative APIs can coexist.
The declarative API should be the normal application path.

⸻

Async Integration
The UI event loop and language async runtime must integrate cleanly.
Target:
function UserView(): View {
    const user = signal<User | null>(null);

    async function loadUser() {
        user = await api.getUser();
    }

    return (
        <Stack>
            <Button onClick={loadUser}>
                Load
            </Button>

            {user && <UserCard user={user} />}
        </Stack>
    );
}
Research:
* UI-thread restrictions
* background tasks
* cancellation
* thread-safe signals
* async event handlers
* synchronization back to UI thread
* structured concurrency tied to component lifetime

⸻

Component-Scoped Async Lifetimes
Investigate whether async work started by a component can automatically belong to that component’s lifetime.
Example:
task(async () => {
    const user = await fetchUser();

    state.user = user;
});
If the component is destroyed before the operation completes, the task could potentially be cancelled automatically.
This should be evaluated as part of structured concurrency rather than implemented as hidden magic without clear semantics.

⸻

Fine-Grained State
Prefer localized reactive state.
Example:
function Form() {
    const name = signal("");
    const email = signal("");

    return (
        <Column>
            <TextInput value={name} />
            <TextInput value={email} />

            <Text>
                {name}
            </Text>
        </Column>
    );
}
Typing into name should not rerender the email field or reconstruct the form.

⸻

Computed State
Example:
const firstName = signal("");
const lastName = signal("");

const fullName = computed(
    () => `${firstName} ${lastName}`
);
Only consumers of fullName should update when one dependency changes.

⸻

Shared Application State
Research a simple pattern for genuinely shared state.
Example:
type AppState = {
    session: Session | null;
    settings: Settings;
};

const state = Shared(new AppState());
or potentially:
const state = store<AppState>({
    session: null,
    settings: defaultSettings,
});
Determine whether store() should imply shared ownership or whether that would hide an important cost.
Preserve the project’s principle that shared ownership should remain understandable.

⸻

Event Handling
Familiar API:
<Button
    onClick={() => save()}
>
    Save
</Button>
Potential strongly typed event APIs:
<TextInput
    onChange={event => {
        console.log(event.value);
    }}
/>
The compiler should know the event type.
Avoid stringly typed event systems.

⸻

Two-Way Binding
Research whether shorthand should exist:
<TextInput value={name} />
where a mutable signal automatically creates a two-way binding.
Or require:
<TextInput
    value={name}
    onChange={value => name = value}
/>
Evaluate ergonomics against explicitness.

⸻

Native Menus
Potential:
<MenuBar>
    <Menu title="File">
        <MenuItem onClick={openFile}>
            Open
        </MenuItem>

        <MenuItem onClick={quit}>
            Quit
        </MenuItem>
    </Menu>
</MenuBar>
Compile to native platform menus.

⸻

Accessibility
Accessibility must be a first-class requirement.
Research:
* screen readers
* keyboard navigation
* semantic roles
* labels
* accessibility tree
* high contrast
* reduced motion
* focus indicators
* platform accessibility APIs
A custom-rendering strategy should not proceed until accessibility implications are understood.
Native controls may provide a significant early advantage here.

⸻

Native System APIs
Eventually expose safe language-native APIs for:
clipboard
notifications
file dialogs
system tray
keyboard shortcuts
process management
filesystem
networking
camera
microphone
printing
screenshot APIs
window management
Platform-specific capabilities should be surfaced through explicit feature detection or compile-time target checks.

⸻

Platform-Specific Code
Investigate syntax such as:
if platform.windows {
    ...
}
or compile-time constructs.
Alternatively:
import { Registry } from "windows";
should produce a compile-time error on unsupported targets unless guarded appropriately.
Do not make platform behavior mysterious.

⸻

Native Packaging
Eventually investigate commands such as:
<cli> build --release
and:
<cli> package
Potential outputs:
Windows:
.exe
.msix
macOS:
.app
.dmg
Linux:
binary
AppImage
.deb
.rpm
Research:
* icons
* metadata
* code signing
* notarization
* installers
* application updates
* bundle resources

⸻

Application Assets
Provide a standard resource mechanism.
Example structure:
src/
assets/
    icon.png
    fonts/
    images/
Potential API:
const icon = asset("./assets/icon.png");
The compiler can embed resources into the binary or package them appropriately.

⸻

Backend + Desktop Shared Code
A major goal should be sharing code between native server and desktop applications.
Potential project:
myapp/
├── server/
│   └── api.tsr
├── desktop/
│   └── app.tsr
└── shared/
    ├── models.tsr
    ├── validation.tsr
    └── business.tsr
Both targets can share:
* types
* validation
* serialization
* domain logic
* API models
* authentication models
* utility functions
without sharing inappropriate UI/runtime dependencies.

⸻

Example Full-Stack Native Application
Server:
app.get("/users/:id", async ctx => {
    const user = await db.users.find(ctx.params.id);

    return ctx.json(user);
});
Desktop:
function UserScreen(): View {
    const user = signal<User | null>(null);

    async function load(id: string) {
        user = await fetch(`/users/${id}`)
            .json<User>();
    }

    return (
        <Window title="Users">
            {user
                ? <UserCard user={user} />
                : <Text>No user loaded</Text>}
        </Window>
    );
}
Same language.
Same type system.
Same async model.
Same JSON model.
Different runtime targets.

⸻

Server HTML and Desktop JSX Should Reinforce Each Other
Research whether a common component grammar can serve:
HTML renderer
Native UI renderer
Potential future terminal UI renderer
Potential future static-document renderer
without forcing all renderers into one overly generic abstraction.
Prefer strongly typed target-specific component sets.

⸻

Potential Terminal UI Target
Do not prioritize initially, but note that the same component/reactivity system could potentially support:
<Terminal>
    <Text>Build status</Text>
    <Progress value={progress} />
</Terminal>
This could provide a third proof that the component model is not browser-specific.

⸻

Performance Benchmarks
Add native UI benchmarks to the project benchmark suite.
Measure:
* application startup
* idle memory
* binary size
* window creation
* 1,000-item list rendering
* list scrolling
* text input latency
* signal update latency
* component creation/destruction
* layout recalculation
* memory allocation
* repeated state updates
* background async operations
Compare where appropriate against:
* Electron
* Tauri
* native Rust UI frameworks
* .NET desktop UI
* Swift/AppKit on macOS
* native Windows frameworks
Avoid simplistic cross-platform claims.
Use equivalent workloads.

⸻

Key Fine-Grained Update Benchmark
Create a canonical benchmark:
function CounterGrid(): View {
    const counters = createCounters(1000);

    return (
        <Grid>
            {counters.map(counter => (
                <Text key={counter.id}>
                    {counter.value}
                </Text>
            ))}
        </Grid>
    );
}
Update one counter repeatedly.
Measure whether one update causes:
one text property update
rather than:
1,000 component reevaluations
This directly tests the fine-grained reactivity thesis.

⸻

Memory Benchmark
Create/destroy complex window trees repeatedly and verify deterministic resource cleanup.
Measure:
* retained memory
* allocation count
* shared reference count behavior
* handles
* native resources
No GC should be required to eventually reclaim abandoned windows.

⸻

Research Questions
Do not prematurely resolve:
* Should UI controls use native platform widgets initially?
* Can native controls provide sufficiently consistent layout?
* Should a custom renderer eventually exist?
* What exact reactive model should be used?
* How much dependency analysis can happen at compile time?
* Are signals ordinary language objects or compiler primitives?
* Should signals require shared ownership?
* How do signals interact with threads?
* How should async tasks belong to component lifetimes?
* What happens when reactive dependencies change dynamically?
* How are conditional subtrees owned?
* How should lists preserve identity?
* Is JSX the correct syntax for native UI?
* Should .tsx-style file distinction exist, or should .tsr parse JSX everywhere?
* How should styling work?
* How closely should layout resemble CSS Flexbox/Grid?
* How are accessibility properties represented?
* How should native menus/dialogs be abstracted?
* How should platform-specific APIs work?
* How should component errors be handled?
* Should hot reload preserve state?
* Can native UI compilation remain fast enough for rapid iteration?
* How should debugging identify reactive dependencies?
* How should resource ownership interact with OS handles?
* How should application updates and packaging work?
These questions belong in the groundwork phase.

⸻

Development Experience
Target:
<cli> dev
for native UI applications.
Desired behavior:
* incremental compile
* automatic restart
* clear diagnostics
* eventually hot reload where safe
* optional state preservation
Fast development iteration is critical.
Native compilation cannot mean waiting a long time after every UI edit.

⸻

Reactive Debugging
Investigate developer tooling such as:
<cli> inspect reactive
Potential output:
Text#42.text
  depends on:
    Counter.count

UserCard.visible
  depends on:
    App.selectedUser
This can make fine-grained reactivity understandable rather than magical.

⸻

Ownership Debugging for UI
Potential tooling:
MainWindow
  owns:
    Sidebar
    Content

AppState
  Shared<AppState>
  owners:
    MainWindow
    BackgroundSync

UserDialog
  borrowed User
Again, this should be optional diagnostic tooling, not normal application syntax.

⸻

Native UI Standard Library Candidate
Potential built-in module:
import {
    Window,
    View,
    Text,
    Button,
    TextInput,
    Checkbox,
    Select,
    List,
    Grid,
    Row,
    Column,
    Stack,
    Scroll,
    Image,
    Canvas,
    Menu,
    Dialog,
    signal,
    computed,
    effect,
} from "ui";
The exact API is research work.

⸻

First Native UI Prototype
The first prototype should be intentionally small.
Support:
Window
Column
Text
Button
signal
onClick
Example:
function Counter(): View {
    const count = signal(0);

    return (
        <Window title="Counter">
            <Column>
                <Text>{count}</Text>

                <Button onClick={() => count++}>
                    Increment
                </Button>
            </Column>
        </Window>
    );
}
Requirements:
1. Compile to native binary.
2. Open a real OS window.
3. Clicking the button updates only the text binding.
4. No virtual DOM.
5. No browser engine.
6. No tracing garbage collector.
7. Closing the window deterministically destroys associated resources.
8. Compiler/runtime debugging can prove which reactive binding updated.

⸻

Second UI Prototype
Build a practical data application.
Example:
User manager
Features:
* fetch users asynchronously
* display list
* select user
* edit fields
* validate input
* save
* loading state
* error state
* multiple reactive values
This tests whether the architecture works beyond counters.

⸻

Third UI Prototype
Create a desktop client for the language’s own Hono-compatible/native server.
Structure:
shared models
shared validation
server API
desktop client
Use this to validate the full-stack thesis.

⸻

Core Native UI Thesis
The project should investigate whether this combination is feasible and compelling:
JSX-like component composition + fine-grained reactivity + native platform controls + deterministic ownership + native binary deployment.
The target developer experience should feel substantially closer to modern TypeScript UI development than traditional systems-language GUI development.
But the runtime should behave fundamentally differently:
No Chromium
No Node
No virtual DOM requirement
No full-tree rerender
No tracing GC
No implicit shared ownership everywhere
The compiler should leverage static knowledge to update only what actually changed.

⸻

Expanded Product Vision
If successful, the language would not merely be a Node alternative.
It could become:
A TypeScript-familiar native application language for servers, CLIs, desktop applications, and server-rendered UI, using one type system, one package ecosystem, one compiler, and one deterministic memory model.
The purpose is not to make every application look like a website.
The purpose is to preserve the parts of TypeScript development that developers find productive while replacing JavaScript-specific runtime machinery with native equivalents designed for stronger guarantees and predictable performance.