# Desktop reactivity and ownership

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Desktop research; later implementation.

**Scope:** Scope fine-grained updates, component lifetimes, signals, conditional trees, identity, and state.

**Planning deliverable:** A reactive semantics proposal covering cleanup, dynamic dependencies, capture ownership, and update granularity.

**Dependencies to coordinate:** [ownership-and-memory](../guides/language/ownership-and-memory.md), [desktop-architecture](desktop-architecture.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="desktop-02"></a>

## Core Reactive Principle

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

<a id="desktop-03"></a>

## Reactive Primitives

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

<a id="desktop-04"></a>

## Components

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

<a id="desktop-05"></a>

## Native UI Ownership Model

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

<a id="desktop-06"></a>

## No Hidden Shared Ownership by Default

Do not automatically place every component, event handler, and state object inside reference-counted shared containers.
Prefer:
1. normal ownership
2. inferred borrowing
3. scoped references
4. explicit shared ownership when necessary
This is important to preserve predictable performance and memory behavior.

<a id="desktop-11"></a>

## Compile JSX Directly

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

<a id="desktop-12"></a>

## Conditional UI

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

<a id="desktop-13"></a>

## Lists

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

<a id="desktop-20"></a>

## Fine-Grained State

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

<a id="desktop-21"></a>

## Computed State

Example:
const firstName = signal("");
const lastName = signal("");

const fullName = computed(
    () => `${firstName} ${lastName}`
);
Only consumers of fullName should update when one dependency changes.

<a id="desktop-22"></a>

## Shared Application State

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
