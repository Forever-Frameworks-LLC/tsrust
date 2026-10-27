# Identity and naming

[Documentation index](../../README.md) · [Proposed roadmap](../../roadmap.md) · [Source section map](../../plans/section-map.md)

## Current implementation

The current source extension is `.tsr`, aligned with the `tsrust` project and compiler name. Fixtures, compiler usage, and documentation use this spelling. Final language branding and namespace research remain open.

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Groundwork.

**Scope:** Research the language name, extension, CLI, and ecosystem namespaces.

**Planning deliverable:** A collision-backed shortlist of at most ten candidates and three candidates for further clearance.

**Dependencies to coordinate:** [competitive-landscape](../../plans/competitive-landscape.md), [research-program](../../plans/research-program.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="web-02"></a>

## 2. Identity Research

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

<a id="web-03"></a>

## 3. Identity Components Should Be Evaluated Separately

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

<a id="web-04"></a>

## 4. Source Extension Research

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

<a id="web-05"></a>

## 5. CLI Research

The CLI should be short, memorable, and unlikely to collide with existing developer tools.
Potential desired experience:
<cli> new api
<cli> dev
<cli> run
<cli> build
<cli> check
<cli> test
<cli> fmt
<cli> check
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

<a id="web-06"></a>

## 6. Naming Directions to Explore

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

<a id="web-07"></a>

## 7. Previously Investigated Names

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

<a id="web-35"></a>

## 35. Naming Should Follow Technical Positioning

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

<a id="web-36"></a>

## 36. Final Naming Output

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
