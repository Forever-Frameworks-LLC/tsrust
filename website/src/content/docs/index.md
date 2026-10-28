---
title: Build native backends with Tsrust
description: Learn the TypeScript-style language, native toolchain and checked backend APIs.
---

Tsrust is an experimental language for native tools and HTTP services. It combines
familiar TypeScript-style syntax with inferred ownership and a Rust backend.
Deployed programs run without a JavaScript runtime.

## Start with a working program

[Build your first program](./start/quickstart/) and learn what the compiler checks,
what native execution requires, and how to create a backend service.

```typescript
function main() {
  const languages = ['TypeScript', 'Rust'];
  console.log(languages.join(' + '));
}
```

The output is `TypeScript + Rust`.

## Bring your own frontend

Keep service source and access policy in the backend. Generate independent named
TypeScript or asynchronous Rust clients from its checked API description. Tsrust
does not require a frontend framework or a generated client to run the server.

## Know the boundaries

Tsrust is not a compiler for arbitrary existing TypeScript, React or Node packages.
Ownership, checked arithmetic and documented string indexing rules matter when
porting code. Read [project status](./project/status/) before adopting it.

These docs are being reviewed and migrated section by section. Published pages
explain supported behavior; the sidebar grows as complete guidance becomes ready.
