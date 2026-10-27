# Generated Tsrust API client

Import createClient and supply a transport. Named calls validate inputs and results.
Wire integers must be safe JavaScript integers; numbers must be finite. Strings
must be well-formed Unicode. Optional fields omit undefined; null remains distinct.
Unknown fields, resources and recursive data reject. Void results use a null wire
value and return undefined. The transport returns `{ok:true,value}` or
`{ok:false,error:{code,message}}`.

The separate `/http` export provides `createHttpTransport({url,headers?,fetch?})`
using the host Fetch API and its types. It posts JSON, omits cookies, rejects
redirects and forwards AbortSignal. Cancellation does not prove rollback or
backend cleanup completion. Ordinary named calls do not add caching or retries.

## Versioned APIs and rolling deployments

The npm package version identifies an SDK release. `apiVersion` identifies its
explicit API compatibility major; `contractId` identifies the exact manifest, and
`wireContractId` also distinguishes different API majors with identical schemas.
Generation retains checked baselines in compatibility.json and rejects accidental
breaking changes within one API major. Keep that generated metadata in version
control with the package. New required result fields require a new API major;
optional additions can preserve existing clients and older replicas.

The HTTP adapter selects a known safe request profile before sending each call.
Common operations can keep working when unrelated operations change or the SDK
selects a smaller operation set. Arguments are never silently removed and writes
are never retried to discover server support. ApiCompatibilityError reports an
unsupported operation or incompatible contract, preserves its public error code,
and includes the requested API major. A new operation or supplied new argument
can still require a newer server. Baseline checks cannot establish business
semantics, deployment-wide capabilities or support on unknown historical servers.

Older API majors remain usable only while the application retains compatible
handlers or migrations. An SDK cannot implement behavior absent from an older
replica. Arrange deployment/routing accordingly when introducing new features.

The package is ordinary dependency-free ESM with no install scripts or backend source.
It is private by default; review contents before explicitly enabling publication.
Use it from any UI or JavaScript host with a compatible HTTP transport.

The public `http-api.json` describes the HTTP wire contract and retained version
history without backend sources. Regenerate from a saved description with
`tsrust api generate --description http-api.json --out sdk`. The generated
`openapi.json` describes the single POST endpoint and includes the checked contract
as `x-tsrust-http`. Set its server URL and authentication policy for your deployment.
It does not invent REST endpoints or infer application authorization rules.
