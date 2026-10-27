# Desktop controls, layout, and accessibility

[Documentation index](../README.md) · [Proposed roadmap](../roadmap.md) · [Source section map](section-map.md)

## Workstream scope

**Status:** Unresolved design/research. The grouping and phase below are proposed planning aids, not approved language semantics.

**Proposed phase:** Desktop research; later implementation.

**Scope:** Scope windows, controls, styling, layout, events, bindings, menus, accessibility, and the UI module.

**Planning deliverable:** A minimal UI API plus keyboard, focus, input, and accessibility acceptance criteria.

**Dependencies to coordinate:** [desktop-architecture](desktop-architecture.md), [desktop-reactivity](desktop-reactivity.md)

## Original vision sections

The sections below retain the original wording and examples. Their proposals and open questions remain proposals and open questions. Section IDs identify the original document family; they do not define implementation order.

<a id="desktop-14"></a>

## Styling

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

<a id="desktop-15"></a>

## Layout

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

<a id="desktop-16"></a>

## Built-In Windows

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

<a id="desktop-17"></a>

## Lower-Level Window API

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

<a id="desktop-23"></a>

## Event Handling

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

<a id="desktop-24"></a>

## Two-Way Binding

Research whether shorthand should exist:
<TextInput value={name} />
where a mutable signal automatically creates a two-way binding.
Or require:
<TextInput
    value={name}
    onChange={value => name = value}
/>
Evaluate ergonomics against explicitness.

<a id="desktop-25"></a>

## Native Menus

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

<a id="desktop-26"></a>

## Accessibility

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

<a id="desktop-42"></a>

## Native UI Standard Library Candidate

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
