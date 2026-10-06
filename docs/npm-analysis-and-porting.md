# npm analysis and porting

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Ecosystem proof; later conversion.

**Scope:** Separate compatibility inspection from automatic conversion and upstream maintenance.

**Planning deliverable:** An inspection-only prototype first; explicit feasibility gates before conversion or update tooling.

**Dependencies to coordinate:** [ownership-and-memory](./ownership-and-memory.md), [packages-and-registry](./packages-and-registry.md), [source-migration](./source-migration.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="core-16"></a>

## 16. NPM Compatibility / Conversion Layer

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

<a id="core-56"></a>

## 56. JavaScript Package Analyzer Prototype

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

<a id="core-58"></a>

## 58. NPM-Assisted Porting System

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

<a id="core-59"></a>

## 59. Package Port Metadata

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
