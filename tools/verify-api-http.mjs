import assert from 'node:assert/strict';
import { createServer } from 'node:http';
import { once } from 'node:events';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

export async function verifyHttp(directory) {
  const { createClient, ApiError, contractId } = await import(pathToFileURL(path.join(directory, 'index.js')));
  const { createHttpTransport } = await import(pathToFileURL(path.join(directory, 'http.js')));
  const requests = [];
  let finishSlow;
  let slowStarted;
  const slowReady = new Promise(resolve => { slowStarted = resolve; });
  const server = createServer(async (request, response) => {
    const chunks = [];
    for await (const chunk of request) chunks.push(chunk);
    const body = JSON.parse(Buffer.concat(chunks).toString());
    requests.push({ url: request.url, headers: request.headers, body });
    response.setHeader('content-type', 'application/json; charset=utf-8');
    if (request.url === '/redirect') {
      response.writeHead(307, { location: '/api' });
      response.end();
    } else if (request.url === '/denied') {
      response.writeHead(403);
      response.end(JSON.stringify({ ok: false, error: { code: 'denied', message: 'Access denied' } }));
    } else if (request.url === '/html') {
      response.writeHead(502, { 'content-type': 'text/html' });
      response.end('<html>private upstream detail</html>');
    } else if (request.url === '/invalid') {
      response.end('{');
    } else if (request.url === '/false-success') {
      response.writeHead(500);
      response.end(JSON.stringify({ ok: true, value: 2 }));
    } else if (request.url === '/wrong-type') {
      response.end(JSON.stringify({ ok: true, value: 'two' }));
    } else if (request.url === '/slow') {
      finishSlow = () => {
        if (!response.writableEnded && !response.destroyed) response.end(JSON.stringify({ ok: true, value: 2 }));
      };
      slowStarted();
    } else {
      const value = body.operation === 'tasks.create'
        ? { kind: 'task', id: 1, title: body.input.title, completed: false }
        : body.input + 1;
      response.end(JSON.stringify({ ok: true, value }));
    }
  });
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const base = `http://127.0.0.1:${server.address().port}`;
  const client = (route, options = {}) => createClient(createHttpTransport({ url: base + route, ...options }));
  const rejects = async (promise, code) => assert.rejects(promise, error => error instanceof ApiError && error.code === code);
  try {
    let token = 'first';
    const api = client('/api', { headers: async () => ({ authorization: `Bearer ${token}`, accept: 'text/plain', 'x-tsrust-contract': 'wrong' }) });
    assert.equal(await api.math.count(1), 2);
    token = 'second';
    assert.equal(await api.math.count(2), 3);
    assert.equal(requests[0].headers.authorization, 'Bearer first');
    assert.equal(requests[1].headers.authorization, 'Bearer second');
    assert.equal(requests[0].headers['x-tsrust-contract'], contractId);
    assert.equal(requests[0].headers.accept, 'application/json');
    assert.deepEqual(requests[0].body, { operation: 'math.count', input: 1 });
    assert.equal((await api.tasks.create({ title: 'Ship \u{1f680}' })).title, 'Ship \u{1f680}');
    await rejects(client('/denied').math.count(1), 'denied');
    await rejects(client('/html').math.count(1), 'api-http');
    await rejects(client('/invalid').math.count(1), 'api-protocol');
    await rejects(client('/false-success').math.count(1), 'api-http');
    await rejects(client('/wrong-type').math.count(1), 'api-validation');
    const beforeRedirect = requests.length;
    await rejects(client('/redirect').math.count(1), 'api-network');
    assert.equal(requests.length, beforeRedirect + 1, 'redirect does not forward authorization or replay a mutation');
    const beforeInvalid = requests.length;
    await rejects(api.tasks.create({ title: '\ud800' }), 'api-validation');
    await rejects(api.tasks.create({ title: '\udfff' }), 'api-validation');
    const aborted = new AbortController();
    aborted.abort();
    await rejects(api.math.count(1, { signal: aborted.signal }), 'api-cancelled');
    await rejects(createHttpTransport({ url: base + '/api' }).request('hidden', null), 'api-operation');
    assert.equal(requests.length, beforeInvalid, 'invalid, cancelled and unlisted calls never reach HTTP');
    const slow = new AbortController();
    const pending = client('/slow').math.count(1, { signal: slow.signal });
    await slowReady;
    slow.abort();
    await rejects(pending, 'api-cancelled');
    finishSlow();
    let attempts = 0;
    await rejects(client('/api', { fetch: async () => { attempts++; throw new Error('secret credential'); } }).math.count(1), 'api-network');
    assert.equal(attempts, 1, 'transport never retries');
    let captured;
    const injected = client('/api', { fetch: async (url, init) => {
      captured = { url, init };
      return new Response(JSON.stringify({ ok: true, value: 2 }), { headers: { 'content-type': 'application/json' } });
    } });
    assert.equal(await injected.math.count(1), 2);
    assert.equal(captured.init.credentials, 'omit');
    assert.equal(captured.init.redirect, 'error');
    for (const url of ['file:///private', 'https://user:secret@example.test/api', base + '/api#fragment']) {
      assert.throws(() => createHttpTransport({ url }), TypeError);
    }
    console.log('API HTTP: real loopback, rotating auth, structured failures, strict responses, no retries/redirects and abort passed.');
  } finally {
    finishSlow?.();
    server.closeAllConnections();
    await new Promise(resolve => server.close(resolve));
  }
}
