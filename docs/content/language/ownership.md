<!-- tsrust-docs
{"id":"reference-language-ownership","group":"language","order":20,"status":"partial","targets":["web","native"]}
-->
# Ownership and resources

Owned values move when consumed. Borrowed observation does not extend an owner lifetime. Cloning must be explicit where the type supports it; a resource is not made copyable by putting it in a container.

UI callbacks retain checked bindings while their component owns state. Removing a conditional subtree releases that subtree; [retained routes](/ui/foundations/retained-routes) preserve visited content until its parent is released.

The full reference still needs a systematic move/borrow/resource table and diagnostic examples. This page does not relax existing ownership or cleanup rules.
