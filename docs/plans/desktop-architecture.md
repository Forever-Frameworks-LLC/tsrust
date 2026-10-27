# Desktop architecture and rendering

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Desktop research; later implementation.

**Scope:** Compare rendering strategies and define distinct UI/HTML targets without committing to a renderer.

**Planning deliverable:** A native-control/custom-renderer feasibility decision and typed rendering boundaries.

**Dependencies to coordinate:** [ownership-and-memory](../guides/language/ownership-and-memory.md), [server-html](../guides/language/server-html.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="desktop-00"></a>

## Project Addendum: Native Desktop UI and Fine-Grained Reactive Components

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

<a id="desktop-01"></a>

## Native UI Product Goal

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

<a id="desktop-07"></a>

## Native Platform Strategy

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

<a id="desktop-08"></a>

## Option B — Custom Cross-Platform Renderer

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

<a id="desktop-09"></a>

## Shared JSX Syntax

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

<a id="desktop-10"></a>

## Strongly Typed Rendering Targets

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

<a id="desktop-33"></a>

## Server HTML and Desktop JSX Should Reinforce Each Other

Research whether a common component grammar can serve:
HTML renderer
Native UI renderer
Potential future terminal UI renderer
Potential future static-document renderer
without forcing all renderers into one overly generic abstraction.
Prefer strongly typed target-specific component sets.

<a id="desktop-34"></a>

## Potential Terminal UI Target

Do not prioritize initially, but note that the same component/reactivity system could potentially support:
<Terminal>
    <Text>Build status</Text>
    <Progress value={progress} />
</Terminal>
This could provide a third proof that the component model is not browser-specific.

<a id="desktop-46"></a>

## Core Native UI Thesis

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
