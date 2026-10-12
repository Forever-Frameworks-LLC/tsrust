# Schema diagnostic messages and paths

Supply a fallback validation message as the constructor's final option:

```ts
import { s } from 'std:schema';

const count = s.int({ message: 'Count must be at least two' }).min(2);
```

The option changes failure text, not validation rules. A custom child message takes
precedence over an enclosing object's message. Messages remain owned by reusable
schemas and are copied only into retained issues. Unknown options and non-string
messages are checker errors. Invalid constraints still throw
`SchemaConstructionError`; they do not become validation failures.

Refinements may attach an issue to a relative field or array element:

```ts
const account = s.object({ password: s.string(), confirm: s.string() })
  .refine(value => value.password === value.confirm, {
    path: ['confirm'],
    message: 'Passwords must match',
  });
```

The checker validates literal path segments against the callback's input type.
Use field names for objects and nonnegative integer indices for arrays. Every
non-null union alternative must support the path. An index names a diagnostic
location; it does not read the array or prove that element exists at runtime.
An empty or omitted path names the current value. Enclosing schemas prepend their
own path, so an account nested under `profile` reports `profile.confirm`.

IntelliSense offers declared field names, including fields reached through array
indices. For unions it offers only shared fields. Arbitrary dictionary keys have
no finite completion list. The predicate borrows its value read-only; options do
not permit mutation or change the inferred output type. Predicate exceptions
propagate normally.
