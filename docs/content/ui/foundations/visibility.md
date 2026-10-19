<!-- tsrust-docs
{"id":"visibility","group":"foundations","order":50,"status":"partial","targets":["web","native"],"examples":["visibility"]}
-->
# Visibility

`View.onVisibilityChange` reports whether a view has positive rectangular area
inside the window and every clipping ancestor. The callback receives a boolean
after initial layout, then only when that boolean changes.

## Try it

Scroll the example's pane to the observed view. Its status changes to visible.
Scrolling within the visible range does not produce duplicate notifications.
Hiding the view reports false while retaining its state; showing it observes its
position again. Removing it disposes the view and releases its observers; mounting
again creates a new view with an initial notification. PageUp, PageDown, Home and End work when the pane has keyboard focus.

## State and cleanup

Callbacks can update component state. Keep those updates stable: a callback that
continually hides and shows its own view can create a feedback loop. A thrown error
stops the mounted tree with ordinary cleanup. Removing a view releases its callback;
late browser notifications cannot activate its replacement.

## What visibility means

Zero-size views, hidden subtrees and views outside a clipped pane report false.
Opacity, rounded corners, overlapping siblings and other windows do not determine
this rectangular visibility value. It is not an exposure or analytics guarantee.
Browser notifications are asynchronous; native notifications follow layout.
Static HTML serialization does not invoke visibility callbacks.
