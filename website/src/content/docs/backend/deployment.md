---
title: Deploy a native backend
description: Build a release executable and verify its requirements on the destination host.
---

Build your service on the OS and architecture where it will run. Prepare the
[native build tools](../../tools/rust-setup/) first, then run these commands from
the configured [backend project](../applications/) root.

## Check and build

Check the source and run its discovered tests before producing a release build:

```sh
tsrust check
tsrust test
tsrust format --check
```

On macOS or Linux:

```sh
tsrust build --release -o target/server
./target/server
```

In Windows PowerShell, choose an executable filename explicitly:

```powershell
tsrust build --release -o target/server.exe
.\target\server.exe
```

`--release` selects optimized native compilation. `-o` selects the output path;
it does not select a target platform. These build commands have no cross-compilation
switch. The server stays running until stopped; use your deployment environment's
process supervisor to manage its lifetime.

## Choose the listening address and access policy

The starter listens on `127.0.0.1:4318`, which accepts connections from the same
host. Set the address passed to `listen` in `app/main.tsr` for your deployment's
network arrangement. A reverse proxy on the same host can forward to loopback;
remote clients need an appropriately reachable listener or proxy.

The starter greeting is deliberately public. Review
[authorization](../authorization/) before adding protected operations. Its
`/api/description` and `/api/openapi.json` handlers use application policy too;
decide whether those routes should be reachable by your clients.

Configure TLS termination, request limits and process supervision for the actual
service. See [HTTP transport](../http-transport/) for the native TLS and server
configuration APIs. Keep deployment secrets out of source and generated client
packages.

## Copy the executable and application files

The compiled program runs without the Tsrust compiler, Cargo, Node or a JavaScript
runtime. Copy the executable plus any configuration, assets and other files your
application reads at runtime. For example, a static-file directory must remain
available at the path your application uses; a relative path depends on the
process working directory.

A native executable is not necessarily statically linked. The destination may
still need OS libraries, external native dependencies and TLS trust material.
Building for the same OS and architecture alone does not establish compatibility
with every OS release or container image.

## Verify the destination

Run the actual artifact in the intended destination image or machine. Verify that
it starts, can read its required files, accepts the intended requests, rejects
unauthorized requests and can reach its external services. Check the process's
working directory and environment as part of that deployment.

`tsrust doctor --native` validates the build host. It does not prove that a copied
executable will run on another machine. A successful source test run also does not
exercise your production network, certificates or access configuration.

## Roll out API changes

Generate independent clients from the checked contract. Keep compatible prior
contracts available while older clients are deployed, and follow the
[API compatibility guide](../api-compatibility/) when changing operation shapes or
versions. Updating the server executable does not update client applications.
