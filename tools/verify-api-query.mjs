import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { spawnSync } from 'node:child_process';
import { fileURLToPath, pathToFileURL } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const fixture = path.join(root, 'tools/api-client-tests');
const compiler = process.env.TSRUST || path.join(root, 'target/debug/tsrust' + (process.platform === 'win32' ? '.exe' : ''));
assert.ok(process.argv.length === 2 || process.argv.length === 3, 'Optional argument: an already staged test directory');
const staged = process.argv.length === 3;
const directory = staged ? path.resolve(process.argv[2]) : fs.mkdtempSync(path.join(fixture, '.tmp-query-'));
const client = path.join(directory, 'client');
function command(file, args) {
  const result = spawnSync(file, args, { cwd: root, encoding: 'utf8', timeout: 60000, maxBuffer: 4 * 1024 * 1024, windowsHide: true });
  assert.ifError(result.error);
  assert.equal(result.status, 0, result.stdout + result.stderr);
}
const module = name => import(pathToFileURL(path.join(fixture, 'node_modules', name)));
const { QueryClient, QueryObserver, isCancelledError, skipToken } = await module('@tanstack/react-query/build/modern/index.js');
const { createElement } = await module('react/index.js');
const { renderToString } = await module('react-dom/server.node.js');
let cache;
try {
  if (!staged) {
    const config = path.join(root, 'examples/api-contract/tsrust.query.api.json');
    command(compiler, ['api', 'generate', '--config', config, '--out', client]);
    command(compiler, ['api', 'generate', '--config', config, '--out', client, '--check']);
  }
  const { createReactClient } = await import(pathToFileURL(path.join(client, 'react-query.js')));
  const { ApiError, contractId } = await import(pathToFileURL(path.join(client, 'index.js')));
  let calls = [], rows = [], pendingSignal, prototypeResult;
  const transport = { async request(operation, input, options) {
    calls.push({ operation, input });
    if (operation === 'tasks.list') {
      if (input.search === 'wait') return new Promise((_, reject) => {
        pendingSignal = options.signal;
        options.signal.addEventListener('abort', () => reject(new ApiError('api-aborted', 'Cancelled', operation)), { once: true });
      });
      if (input.search === 'fail') return { ok: false, error: { code: 'api-denied', message: 'Denied' } };
      return { ok: true, value: rows };
    }
    if (operation === 'tasks.create') {
      if (input.title === 'deny') return { ok: false, error: { code: 'api-denied', message: 'Denied' } };
      const item = { kind: 'task', id: rows.length + 1, title: input.title, completed: false };
      rows = [...rows, item];
      return { ok: true, value: item };
    }
    if (operation === 'data.inspectKeys' && prototypeResult !== undefined) return { ok: true, value: JSON.parse(`{"__proto__":${JSON.stringify(prototypeResult)}}`) };
    if (operation === 'data.inspectKeys' || operation === 'data.echo') return { ok: true, value: input };
    if (operation === 'meta.marker') return { ok: true, value: '__TSRUST_API_ID__' };
    return { ok: true, value: input + 1 };
  } };
  assert.throws(() => createReactClient(transport, {}), /scope/);
  const api = createReactClient(transport, { scope: 'backend-one/user-one' });
  const other = createReactClient(transport, { scope: 'backend-one/user-two' });
  cache = new QueryClient({ defaultOptions: { queries: { gcTime: Infinity, retry: 2 }, mutations: { gcTime: Infinity, retry: 2 } } });
  const input = { search: null };
  const options = api.tasks.list.queryOptions(input, { staleTime: Infinity });
  input.search = 'changed';
  assert.deepEqual(await cache.fetchQuery(options), []);
  assert.equal(calls.at(-1).input.search, null, 'query captures checked input before caller mutation');
  const count = calls.length;
  assert.deepEqual(await cache.fetchQuery(options), []);
  assert.equal(calls.length, count, 'TanStack owns cache/deduplication');
  assert.notDeepEqual(api.tasks.list.key({ search: null }), other.tasks.list.key({ search: null }));
  assert.deepEqual(api.tasks.list.key({ search: null, completed: undefined }), api.tasks.list.key({ search: null }));
  assert.notDeepEqual(api.tasks.list.key({ search: null, completed: false }), api.tasks.list.key({ search: null }));
  assert.equal(options.queryKey[2], contractId);
  assert.ok(Object.isFrozen(options.queryKey));
  for (const value of ['first', 'second']) {
    const data = JSON.parse(`{"__proto__":${JSON.stringify(value)}}`);
    const result = await cache.fetchQuery(api.data.inspectKeys.queryOptions(data, { staleTime: Infinity }));
    assert.equal(Object.hasOwn(result, '__proto__'), true);
    assert.equal(result.__proto__, value, 'special data key participates in cache identity');
  }
  prototypeResult = 'refetched';
  const refetched = await cache.fetchQuery(api.data.inspectKeys.queryOptions(JSON.parse('{"__proto__":"first"}'), { staleTime: 0 }));
  assert.equal(Object.hasOwn(refetched, '__proto__'), true, 'upstream cache replacement must preserve wire data fields');
  assert.equal(refetched.__proto__, 'refetched');
  const refetchedCache = cache.getQueryData(api.data.inspectKeys.queryOptions(JSON.parse('{"__proto__":"first"}')).queryKey);
  assert.equal(Object.hasOwn(refetchedCache, '__proto__'), true, 'stored data preserves special wire fields after refetch');
  assert.equal(refetchedCache.__proto__, 'refetched');
  assert.throws(() => api.data.inspectKeys.queryOptions(JSON.parse('{"__proto__":"first"}'), { structuralSharing: true }), /structuralSharing/);
  prototypeResult = undefined;
  const skipped = api.tasks.list.queryOptions(skipToken);
  assert.equal(skipped.queryFn, skipToken);
  assert.throws(() => api.tasks.list.queryOptions({ search: null }, { queryKey: ['spoof'] }), /owns queryKey/);
  assert.throws(() => api.tasks.create.mutationOptions({ mutationFn() {} }), /owns mutationFn/);
  assert.throws(() => api.tasks.list.queryOptions({ search: 7 }), error => error.code === 'api-validation');
  const beforeFailure = calls.length;
  await assert.rejects(cache.fetchQuery(api.tasks.list.queryOptions({ search: 'fail' })), error => error.code === 'api-denied');
  assert.equal(calls.length, beforeFailure + 1, 'no implicit query retries');
  const otherOptions = other.tasks.list.queryOptions({ search: null }, { staleTime: Infinity });
  await cache.fetchQuery(otherOptions);
  const beforeMutationFailure = calls.length;
  const deniedMutation = cache.getMutationCache().build(cache, api.tasks.create.mutationOptions());
  await assert.rejects(deniedMutation.execute({ title: 'deny' }), error => error.code === 'api-denied');
  assert.equal(calls.length, beforeMutationFailure + 1, 'no implicit mutation retries');

  const observer = new QueryObserver(cache, options);
  const unsubscribe = observer.subscribe(() => {});
  let contextSeen;
  const mutation = cache.getMutationCache().build(cache, api.tasks.create.mutationOptions({
    onMutate: variables => ({ previous: variables.title }),
    onSuccess: (_, __, context) => { contextSeen = context.previous; },
  }));
  const item = await mutation.execute({ title: 'Created through TanStack' });
  assert.equal(item.title, 'Created through TanStack');
  assert.equal(contextSeen, item.title);
  assert.deepEqual(cache.getQueryData(options.queryKey), [item], 'acknowledged mutation refreshes active queries in this scope');
  assert.deepEqual(cache.getQueryData(otherOptions.queryKey), [], 'another user cache remains isolated');
  assert.equal(cache.getQueryState(otherOptions.queryKey).isInvalidated, false);
  unsubscribe();
  let invalidated = false;
  const refreshFailure = api.tasks.create.mutationOptions();
  assert.ok(Object.isFrozen(refreshFailure.mutationKey));
  const acknowledged = await refreshFailure.mutationFn({ title: 'Write acknowledged' }, { client: { invalidateQueries() { invalidated = true; return Promise.reject(new Error('Refresh failed')); } } });
  assert.ok(invalidated);
  assert.equal(acknowledged.title, 'Write acknowledged', 'refresh failure never converts an acknowledged write into failure');

  const waiting = cache.fetchQuery(api.tasks.list.queryOptions({ search: 'wait' }));
  const cancelled = assert.rejects(waiting, isCancelledError);
  await Promise.resolve();
  assert.ok(pendingSignal);
  await cache.cancelQueries({ queryKey: api.tasks.list.key({ search: 'wait' }) });
  await cancelled;
  assert.equal(pendingSignal.aborted, true);

  function Probe() {
    const query = api.tasks.list.useQuery({ search: null }, { select: items => items.length }, cache);
    const mutation = api.tasks.create.useMutation({}, cache);
    const disabled = api.tasks.list.useQuery(skipToken, undefined, cache);
    return createElement('output', null, `${query.data}/${mutation.status}/${disabled.fetchStatus}`);
  }
  assert.equal(renderToString(createElement(Probe)), '<output>1/idle/idle</output>');

  fs.writeFileSync(path.join(directory, 'consumer.mts'), `import {QueryClient,skipToken} from '@tanstack/react-query';
import {createReactClient} from './client/react-query.js';
import type {Transport,ApiError} from './client/index.js';
declare const transport:Transport;
const api=createReactClient(transport,{scope:'backend/user'});
const cache=new QueryClient();
const options=api.tasks.list.queryOptions({search:null},{select:rows=>rows.length});
const original:{kind:'task';id:number;title:string;completed:boolean}[]|undefined=cache.getQueryData(options.queryKey);
const selected:number|undefined=api.tasks.list.useQuery({search:null},{select:rows=>rows.length}).data;
const failure:ApiError|null=api.tasks.list.useQuery(skipToken).error;
const direct:Promise<number>=api.math.count(3);
const marker:'__TSRUST_API_ID__'|undefined=api.meta.marker.useQuery().data;
const mutation=api.tasks.create.useMutation({onMutate:input=>({previous:input.title}),onSuccess:(data,input,context)=>{const previous:string|undefined=context?.previous;void previous;void data.id;void input.title;}});
mutation.mutate({title:'Typed mutation'});
// @ts-expect-error Required query input is checked.
api.tasks.list.queryOptions({});
// @ts-expect-error No mutation hook on a query.
api.tasks.list.useMutation();
// @ts-expect-error No query options on a mutation.
api.tasks.create.queryOptions({title:'wrong kind'});
// @ts-expect-error Generated keys cannot be replaced.
api.tasks.list.queryOptions({search:null},{queryKey:['spoof']});
// @ts-expect-error Options are independent of a QueryClient; the hook accepts that argument.
api.tasks.list.queryOptions({search:null},{},cache);
// @ts-expect-error Mutation inputs retain their field types.
mutation.mutate({title:4});
// @ts-expect-error Selection results retain their type.
const wrong:string|undefined=api.tasks.list.useQuery({search:null},{select:rows=>rows.length}).data;
void [original,selected,failure,direct,marker,wrong];
`);
  command(process.execPath, [path.join(fixture, 'node_modules/typescript/bin/tsc'), path.join(directory, 'consumer.mts'), '--strict', '--noEmit', '--module', 'NodeNext', '--target', 'ES2022', '--lib', 'ES2022,DOM']);
  if (!staged) {
    const originalConfig = path.join(root, 'examples/api-contract/tsrust.query.api.json');
    const majorConfig = path.join(path.dirname(originalConfig), `.tsrust-query-major-${process.pid}.json`);
    const configuration = JSON.parse(fs.readFileSync(originalConfig));
    configuration.apiVersion = 2; configuration.package.version = '1.0.0';
    fs.writeFileSync(majorConfig, JSON.stringify(configuration), { flag: 'wx' });
    try {
      const majorClient = path.join(directory, 'major-client');
      command(compiler, ['api', 'generate', '--config', majorConfig, '--out', majorClient]);
      const metadata = await import(pathToFileURL(path.join(majorClient, 'index.js')));
      const { createReactClient: createMajorClient } = await import(pathToFileURL(path.join(majorClient, 'react-query.js')));
      assert.equal(metadata.contractId, contractId, 'the public DTO schema is identical');
      assert.notEqual(metadata.wireContractId, contractId, 'the API major has distinct wire identity');
      const next = createMajorClient(transport, { scope: 'backend-one/user-one' });
      assert.notDeepEqual(next.tasks.list.key({ search: null }), api.tasks.list.key({ search: null }), 'different API majors cannot share cached results');
    } finally { fs.unlinkSync(majorConfig); }
  }
  console.log(`${staged ? 'Staged' : 'Generated'} API Query: scoped/special keys, copied inputs, cache/invalidation, mutation context, no retries, abort, real React hooks and upstream TypeScript inference passed.`);
} finally {
  cache?.clear();
  if (!staged) {
    assert.equal(path.dirname(fs.realpathSync(directory)), fs.realpathSync(fixture));
    assert.ok(path.basename(directory).startsWith('.tmp-query-'));
    fs.rmSync(directory, { recursive: true, force: true });
  }
}
