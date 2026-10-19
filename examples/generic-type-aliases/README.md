# Generic type aliases

Run `tsrust run examples/generic-type-aliases/main.tsr`.

Aliases can describe arrays, unions, tuples, functions with ownership wrappers and
applications of generic records/native types. State data borrowing at the function
parameter, for example `rows: ReadOnly<Rows>`, rather than aliasing a data access
wrapper. Their parameters substitute types;
they do not allocate a wrapper, clone resources or change ownership. Optional type
arguments use their defaults; defaults can refer only to earlier parameters.

Record-body aliases still use the existing generic-record syntax. Defaults on
record declarations, interfaces and function type parameters remain unsupported;
name an ordinary record template and give its forwarding alias a default instead.
Recursive aliases and expansions larger than 65536 annotation nodes are rejected.
