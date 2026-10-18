# JSX integration

JSX is syntax for ordinary checked component calls. `<Label title="Hello" />`
passes one props record to `Label`. Components are uppercase function names in
scope, including imported names. Attributes become record fields in source order;
bare attributes mean `true`. Braced attributes use the ordinary expression parser,
including checked closures. Children become an ordered `children` array when
present. The component's declared props type checks attributes and children, and
normal ownership rules govern captures, transfers and borrowing.

`.tsrx` files participate in project discovery, checking, formatting, dev watching
and editor imports. Extensionless resolution tries `.tsr`, `.tsrx`, `index.tsr`,
then `index.tsrx`. Explicit extensions remain exact.

The first syntax section supports named components, quoted/braced attributes,
self-closing elements, nested components, text and braced children. Indentation in
multiline text is folded; inline spaces are retained. Use a braced string for an
exact text value. Fragments, lowercase intrinsic elements, namespaced/member tags,
attribute spreads and generic tag arguments are not part of this section. There
is no implicit DOM or React runtime. Formatting preserves JSX islands verbatim
while formatting surrounding language syntax and verifying token preservation.

This syntax section does not itself establish renderer, reactive-state or paired
counter acceptance. Those use the same checked calls and remain separate work.

## Focused evidence

Five compiler witnesses pass, including generated native execution and a configured
`.tsrx` project. Editor attribute completion, hover and signature help pass; an
unsaved imported `.tsrx` component invalidates and rechecks its caller. The matched
TypeScript/native JSX witness passes its independent text oracle. Initial template
lexer failure and TypeScript reference typing failure are retained with repaired
results in SQLite under `tsrust-jsx-*`. No full regression, SQL or timing run.

The child contract always supplies an array, including a single child. The
TypeScript reference admits a scalar or array for TypeScript JSX checking, while
its explicit factory always supplies an array. This is a documented source typing
difference, not an implicit conversion in Tsrust.
