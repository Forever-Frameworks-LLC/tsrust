# Checked API client foundation

This bounded example exposes an explicit allowlist of checked functions. It is a
client-generation witness, not the completed HTTP/SQLite service or UI prototype.
The extra exported `serverOnly` function is deliberately absent from the API.

```sh
tsrust api generate --config tsrust.api.json
tsrust api generate --config tsrust.api.json --check
tsrust run
```

The generated directory is an ordinary private ESM package with JavaScript,
declarations and a portable manifest. It imports no server code or frontend
framework and installs no dependencies. Output can be selected with `--out` for
an existing project's workspace or standalone export. Existing non-generated
directories and edited generated files are preserved.

Named calls use `createClient(transport)` and validate plain inputs/results. The
transport returns `{ok:true,value}` or `{ok:false,error:{code,message}}`. A generated
HTTP adapter and optional TanStack wrapper are generated alongside the plain
client. Desktop integration remains part of the prototype work.

Operations currently accept no argument or one required plain-data argument.
Integers use safe JavaScript numbers; finite numbers, strings, booleans, null,
literal types, arrays, tuples, data records and unions have explicit codecs.
Optional record fields omit undefined and remain distinct from null. Native
handles, functions, implicit serialization of collections and recursive wire
types reject. Use explicit plain-data projections for those values. The API
hash identifies exact contracts; it is not itself a rolling-compatibility policy.
Versioned generation now retains checked SDK baselines and explicit API majors;
its retained-baseline mixed-replica acceptance passes. The server witness uses API v2
because its required revision result cannot be provided by an older v1 replica.
See [the client guide](../../website/src/content/docs/learn/first-service.md) for the policy
and [current evidence](../../docs/agent-progress.md) before claiming rollout support.
