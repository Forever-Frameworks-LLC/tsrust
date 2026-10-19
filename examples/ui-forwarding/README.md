# Owned JSX forwarding and metadata conditions

Run `tsrust run examples/ui-forwarding/inspect.tsrx` for native ownership assertions.
Build/run the configured project for the paired renderer witness. `Panel` is a small
application component, not a published Nova Card.

JSX spread moves its source. Only fields declared by the receiving component are
forwarded; excess spread fields are destroyed once, and omitted optional fields
become undefined. Explicit unknown attributes and missing required props fail.
Ordinary object spread/rest still makes a checked copy and rejects resources.

`data-size` carries metadata, not intrinsic dimensions. Checked data predicates,
descendant slot predicates, named ancestor groups and own-class predicates select
ordinary shared utilities. Later classes win per property/state. Group/descendant
conditions exclude self. These examples do not add arbitrary CSS selectors,
responsive styles, dynamic tree reconciliation or custom HTML attributes.
