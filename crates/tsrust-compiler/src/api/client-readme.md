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

## Optional React/TanStack entry

Applications using React and TanStack Query can import createReactClient from
`<package>/react-query`. The framework peers are optional: plain and HTTP clients
do not import them. This entry currently requires TanStack Query 5.105.0 or later
within major 5, and the upstream-supported React 18/19 peers. The development
fixture tests TanStack 5.105.0 and React 19.3.1.

```ts
const api = createReactClient(transport, { scope: JSON.stringify([backendURL, userId]) });
const query = api.tasks.list.useQuery(input);
const mutation = api.tasks.create.useMutation();
await mutation.mutateAsync({ title: 'Ship prototype' });
// Standard TanStack APIs also accept the generated options:
await queryClient.fetchQuery(api.tasks.list.queryOptions(input));
```

Keep the client stable for a given transport and scope. Scope must identify the
backend and authenticated principal; never use credentials as a visible cache
key. Change scope when either identity changes. Clear the application's QueryClient
on logout if cached private data must be removed immediately. Contract identity
is included automatically. Inputs are validated/copied before cache key creation,
and optional undefined fields normalize to absence. Serialized schema-ordered
input keys preserve special data fields such as `__proto__`.
For result schemas containing that field, default structural sharing is disabled
because upstream's ordinary-object accumulator drops it during cache replacement.
Explicit true rejects for those schemas; false or a caller-supplied sharing
function remains available. Other schemas keep normal upstream structural sharing.

Queries expose `key(input)`, `queryOptions(input, options?)` and
`useQuery(input, options?, queryClient?)`. Use upstream skipToken in place of an
unavailable input to disable a query safely. TanStack select, enabled, staleTime,
initialData and other ordinary options remain available. Generated hooks use
conservative possibly-undefined query data types even with initialData; the
options can also be passed to upstream useQuery for its overload behavior.
Generated query functions/keys/hash functions cannot be replaced.

Mutations expose `mutationOptions(options?)` and
`useMutation(options?, queryClient?)`, with ordinary typed variables, context and
callbacks. A successful response invalidates queries in this client scope;
unrelated scopes remain untouched. Refresh errors stay on query observers and
do not turn an acknowledged write into a mutation failure. Tagged domain failures
remain ordinary typed result data; inspect them just as with plain calls.
Queries and mutations default to no retries, overriding upstream global retry
defaults; callers may opt into retries explicitly. Direct calls on the React
client remain ordinary promises and do not invalidate/cache automatically.

The package is ordinary ESM with no install scripts or backend source. It is
private by default; review its contents and metadata before explicitly enabling
publication. Installing/linking the package in an application must make optional
React/TanStack peers resolvable there, as with any other peer-based library.
