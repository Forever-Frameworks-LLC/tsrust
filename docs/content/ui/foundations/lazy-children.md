<!-- tsrust-docs
{"id":"ui-lazy-children","group":"foundations","order":95,"status":"partial","targets":["web","native"],"examples":["lazy-children"]}
-->
# Lazy children

A component can receive a recipe for its children instead of already constructed
nodes. Declare `children?: LazyChildren` and use ordinary nested JSX at its call
site. The component decides when to call the recipe.

## Construction order

`LazyChildren` is `Owned<Function<Child[]>>`. It takes no arguments and returns a
fresh owned array each time it is called. Attributes run before the parent call.
Between-tag children run only when the parent invokes its child factory, in source
order. Ignoring or dropping the factory does not construct the children.

Ordinary `children?: Child[]` keeps its eager behavior. Passing
`children={factory}` forwards an existing factory; it does not invoke it. There is
no implicit cache and no extra wrapper node. Explicit arrow factories use their
declared array return type, including empty and mixed child literals. Existing
array variables keep their element type; this does not add array covariance.

## Create, close and reopen

The example starts with details absent. Opening constructs the counter, including
its local state. Closing releases that subtree. Reopening constructs a fresh
counter at zero; the parent count and draft remain in their original nodes.

Use retained `hidden` content when the child state should survive closing. Choose
lazy construction when content should not exist until needed. A complete disclosure
component also needs expanded/controls semantics and event/focus policy; this
example demonstrates evaluation and ownership only.

## Ownership

The normal factory is reusable. Its captures must remain valid for another call.
A child expression cannot move the same owned Node or resource out on every call.
Use `Once<Function<Child[]>>` for a component that deliberately consumes its factory
once; a second call then fails checking. Explicit clones remain explicit.

State capture follows ordinary component rules. Factories and event handlers keep
weak state references; the returned component subtree retains its state owner.
Removing child content releases its callbacks and child state. A factory failure
releases partially constructed values without constructing later children.

## Component contracts

Automatic between-tag conversion uses declared top-level function-component props,
including inherited and concrete generic records. It requires an owned,
zero-argument callback returning an array. A stored, local or indirect component callable
can receive an explicit factory through its `children` attribute. Multiple factory
alternatives do not select an overload automatically.

This section requires compiler API 7. It provides the construction order needed
by context providers; it does not introduce context providers or finish event/ref
composition, focus scopes or compound controls.
