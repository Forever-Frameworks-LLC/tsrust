<!-- tsrust-docs
{"id":"retained-routes","group":"foundations","order":60,"status":"ready","targets":["web","native"],"api":[]}
-->
# Retained routes

Create one `location()` owner in the application root. `Route` uses that location
without creating or retaining another owner.

```tsrx
import { View, Text, Route, Link, location, Node } from 'std:ui';

export function App(): Node {
  const path = location();
  return (
    <View>
      <Link href="/">Home</Link>
      <Link href="/settings">Settings</Link>
      <Route path="/" render={() => <Text>Home</Text>} />
      <Route path="/settings" render={() => <Text>Settings</Text>} />
    </View>
  );
}
```

A factory runs once, on the first matching visit. Unvisited routes do not construct
content. Leaving a visited route hides its existing subtree from layout, interaction
and accessibility. Returning restores the same controls, component state and scroll
positions. Removing the owning tree disposes visited subtrees and uncalled factories.
There is no implicit route-cache limit or eviction.

`exact` defaults to `true`. Set `exact={false}` for a path and its descendants:
`/settings` also matches `/settings/profile`, but never `/settings-other`. Fragments
change focus rather than reconstructing the route. Trailing slashes normalize.
Query strings, escapes, traversal, external URLs and fragments in the route's
`path` are rejected. Routes do not parse parameters or select a single winner;
every matching route is active, so use exact paths or intentional nested layouts.

Factory errors propagate without an automatic retry. The factory releases its
captures after construction; returned content owns its ordinary checked resources.
A disposed location cannot be replaced underneath an existing route.

This is lazy construction on both targets. It does not split or defer downloading
the application executable. Ordinary JSX ternaries still dispose removed branches;
`View.hidden` retains content that has already been constructed.
