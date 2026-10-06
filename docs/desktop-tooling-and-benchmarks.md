# Desktop tooling and benchmarks

[Documentation index](./README.md) · [Proposed roadmap](./roadmap.md) · [Source section map](./section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Desktop research; later implementation.

**Scope:** Scope compile iteration, reactive inspection, ownership inspection, and UI/resource benchmarks.

**Planning deliverable:** Repeatable update and teardown measurements with source-level inspection output.

**Dependencies to coordinate:** [toolchain-and-development](./toolchain-and-development.md), [benchmarks](./benchmarks.md), [desktop-reactivity](./desktop-reactivity.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="desktop-35"></a>

## Performance Benchmarks

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

<a id="desktop-36"></a>

## Key Fine-Grained Update Benchmark

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

<a id="desktop-37"></a>

## Memory Benchmark

Create/destroy complex window trees repeatedly and verify deterministic resource cleanup.
Measure:
* retained memory
* allocation count
* shared reference count behavior
* handles
* native resources
No GC should be required to eventually reclaim abandoned windows.

<a id="desktop-39"></a>

## Development Experience

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

<a id="desktop-40"></a>

## Reactive Debugging

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

<a id="desktop-41"></a>

## Ownership Debugging for UI

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
