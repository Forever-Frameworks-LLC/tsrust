# Desktop prototypes and shared code

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Desktop research; later implementation.

**Scope:** Preserve open desktop questions and the counter, data application, and shared server/client experiments.

**Planning deliverable:** A gated counter prototype, then a user manager, then a shared-model server/desktop demonstration.

**Dependencies to coordinate:** [desktop-controls](desktop-controls.md), [desktop-async-and-platform](desktop-async-and-platform.md), [desktop-tooling-and-benchmarks](desktop-tooling-and-benchmarks.md), [http-and-hono](../guides/language/http-and-hono.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="desktop-31"></a>

## Backend + Desktop Shared Code

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

<a id="desktop-32"></a>

## Example Full-Stack Native Application

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

<a id="desktop-38"></a>

## Research Questions

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

<a id="desktop-43"></a>

## First Native UI Prototype

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

<a id="desktop-44"></a>

## Second UI Prototype

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

<a id="desktop-45"></a>

## Third UI Prototype

Create a desktop client for the language’s own Hono-compatible/native server.
Structure:
shared models
shared validation
server API
desktop client
Use this to validate the full-stack thesis.
