import test from 'node:test';
import assert from 'node:assert/strict';
import {mkdtempSync, mkdirSync, writeFileSync, readFileSync, rmSync, symlinkSync, existsSync} from 'node:fs';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {spawnSync} from 'node:child_process';
import {compileManifest, discover, matchRoute, buildPath, parsePath} from '../routes.mjs';

const base = ['app/not-found.tsrx'];
const files = [...base, 'app/layout.tsrx', 'app/error.tsrx', 'app/middleware.tsr',
  'app/items/page.tsrx', 'app/items/loader.tsr', 'app/items/route.tsr', 'app/items/middleware.tsr',
  'app/items/layout.tsrx', 'app/items/template.tsrx', 'app/items/error.tsrx', 'app/items/not-found.tsrx',
  'app/items/[id]/page.tsrx', 'app/items/[id]/middleware.tsr', 'app/items/[id]/template.tsrx',
  'app/items/new/page.tsrx', 'app/items-private/page.tsrx', 'app/health/route.tsr', 'app/login/page.tsrx'];
const handlers = {'app/items/route.tsr': [{method: 'POST', path: '/'}], 'app/health/route.tsr': [{method: 'GET', path: '/'}]};
function good(input = files, declarations = handlers) {
  const result = compileManifest(input, declarations);
  assert.deepEqual(result.diagnostics, []);
  assert.ok(result.manifest);
  return result.manifest;
}
function bad(input, declarations, code) {
  const result = compileManifest(input, declarations);
  assert.equal(result.manifest, null);
  assert.ok(result.diagnostics.some(d => d.code === code), JSON.stringify(result));
  return result.diagnostics;
}
function temporary(t, entries) {
  const root = mkdtempSync(path.join(tmpdir(), 'ui-route-contracts-'));
  t.after(() => rmSync(root, {recursive: true, force: true}));
  for (const [name, content] of entries) {
    mkdirSync(path.dirname(path.join(root, name)), {recursive: true});
    writeFileSync(path.join(root, name), content);
  }
  return root;
}

test('all special files, ordered ancestry, exact local loader and optional endpoints', () => {
  const manifest = good();
  const item = manifest.segments.find(s => s.id === 'app/items');
  assert.deepEqual(item.page, {source: 'app/items/page.tsrx', loader: 'app/items/loader.tsr'});
  assert.deepEqual(item.endpoints.map(e => [e.method, e.kind]), [['GET', 'page'], ['HEAD', 'page'], ['POST', 'route']]);
  const child = manifest.segments.find(s => s.id === 'app/items/[id]');
  assert.equal(child.page.loader, null);
  assert.deepEqual(child.params, ['id']);
  assert.deepEqual(child.ancestry, {
    middleware: ['app/middleware.tsr', 'app/items/middleware.tsr', 'app/items/[id]/middleware.tsr'],
    layout: ['app/layout.tsrx', 'app/items/layout.tsrx'],
    template: ['app/items/template.tsrx', 'app/items/[id]/template.tsrx'],
    'not-found': ['app/not-found.tsrx', 'app/items/not-found.tsrx'],
    error: ['app/error.tsrx', 'app/items/error.tsrx'],
  });
  assert.equal(matchRoute(manifest, '/'), null);
  assert.equal(matchRoute(manifest, '/health').endpoint.kind, 'route');
  assert.equal(matchRoute(manifest, '/health', 'HEAD'), null);
  assert.equal(matchRoute(manifest, '/items', 'POST').endpoint.source, 'app/items/route.tsr');
  assert.equal(Object.hasOwn(matchRoute(manifest, '/items', 'POST').endpoint, 'loader'), false);
  assert.deepEqual(manifest.segments.find(s => s.id === 'app/items-private').ancestry.middleware, ['app/middleware.tsr']);
  const local = good([...base, 'app/page.tsrx', 'app/loader.tsr'], {});
  assert.equal(matchRoute(local, '/').segmentId, 'app');
  assert.equal(local.segments[0].page.loader, 'app/loader.tsr');
});

test('no security helper required; ordinary files create no endpoints', t => {
  const root = temporary(t, [...base, 'app/page.tsrx', 'app/utils.tsr', 'app/widgets/button.tsrx', 'components/page.tsrx'].map(p => [p, '// placeholder\n']));
  const result = discover(root);
  assert.deepEqual(result.diagnostics, []);
  assert.deepEqual(result.manifest.segments.filter(s => s.endpoints.length).map(s => s.id), ['app']);
  assert.deepEqual(result.manifest.segments[0].ancestry.middleware, []);
});

test('missing 404 and orphan loader fail without partial manifest', () => {
  assert.deepEqual(bad(['app/page.tsrx'], {}, 'missing-not-found')[0].sources, ['app/not-found.tsrx']);
  bad([...base, 'app/loader.tsr'], {}, 'unpaired-loader');
  bad([...base, 'app/loader.tsr', 'app/route.tsr'], {'app/route.tsr': []}, 'unpaired-loader');
});

test('page GET and HEAD collisions name both owners; repeated API method rejected', () => {
  for (const method of ['GET', 'HEAD']) {
    const diagnostics = bad([...base, 'app/page.tsrx', 'app/route.tsr'], {'app/route.tsr': [{method, path: '/'}]}, 'method-conflict');
    assert.deepEqual(diagnostics[0].sources, ['app/page.tsrx', 'app/route.tsr']);
    assert.match(diagnostics[0].message, new RegExp(`^${method} / `));
  }
  bad([...base, 'app/route.tsr'], {'app/route.tsr': [{method:'GET', path:'/'}, {method:'GET', path:'/'}]}, 'method-conflict');
});

test('mock declarations are required, exact and never inferred from source', () => {
  bad([...base, 'app/route.tsr'], {}, 'missing-handlers');
  bad(base, {'app/no/route.tsr': []}, 'unknown-handler-source');
  for (const declarations of [null, [], 'bad']) bad(base, declarations, 'invalid-declarations');
  bad([...base, 'app/route.tsr'], {'app/route.tsr': {}}, 'invalid-handlers');
  for (const handler of [null, {}, {method:'get', path:'/'}, {method:'BREW', path:'/'}, {method:'GET', path:'/child'}, {method:'GET', path:'/', extra:true}]) {
    bad([...base, 'app/route.tsr'], {'app/route.tsr': [handler]}, 'invalid-handler');
  }
});

test('literal precedence is path-first, not method fallback', () => {
  const manifest = good();
  assert.equal(matchRoute(manifest, '/items/new').segmentId, 'app/items/new');
  for (const id of ['1', '2', '4', '3', '5']) assert.deepEqual(matchRoute(manifest, `/items/${id}`).params, {id});
  assert.equal(matchRoute(manifest, '/ITEMS/1'), null);
  assert.equal(matchRoute(manifest, '/missing'), null);
  const mixed = good([...base, 'app/items/[id]/page.tsrx', 'app/items/new/route.tsr'], {'app/items/new/route.tsr': [{method:'POST', path:'/'}]});
  assert.equal(matchRoute(mixed, '/items/new'), null);
  assert.equal(matchRoute(mixed, '/items/new', 'POST').endpoint.kind, 'route');
  assert.throws(() => buildPath(manifest, 'app/items/[id]', {id:'new'}), /shadowed/);
});

test('equivalent patterns and repeated parameter names rejected', () => {
  const diagnostics = bad([...base, 'app/[id]/page.tsrx', 'app/[name]/page.tsrx'], {}, 'equivalent-pattern');
  assert.deepEqual(diagnostics[0].sources, ['app/[id]', 'app/[name]']);
  bad([...base, 'app/[id]/children/[id]/page.tsrx'], {}, 'duplicate-parameter');
});

test('unsupported conventions and invalid source identities rejected', () => {
  for (const segment of ['[...slug]', '[[...slug]]', '(group)', '@slot', '[id', '[]', '[bad-name]', '%61']) {
    bad([...base, `app/${segment}/page.tsrx`], {}, 'unsupported-segment');
  }
  for (const file of ['loading.tsrx', 'default.tsrx', 'global-error.tsrx', 'page.tsr', 'loader.tsrx']) bad([...base, `app/${file}`], {}, 'unsupported-convention');
  for (const source of ['../app/page.tsrx', 'app/../page.tsrx', 'app//page.tsrx', 'app\\page.tsrx']) bad([...base, source], {}, 'invalid-source');
  bad([...base, ...base], {}, 'duplicate-source');
});

test('path encoding, canonical slash, opaque IDs, nested params and reserved property names', () => {
  const manifest = good();
  for (const id of ['0', '001', 'a b', 'a+b', 'a?b#c', 'é', 'e\u0301', '東京', '😀', '__proto__']) {
    const url = buildPath(manifest, 'app/items/[id]', {id});
    assert.deepEqual(matchRoute(manifest, `${url}/?q=1&q=2#part`).params, {id});
  }
  assert.notEqual(buildPath(manifest, 'app/items/[id]', {id:'é'}), buildPath(manifest, 'app/items/[id]', {id:'e\u0301'}));
  assert.deepEqual(parsePath('/'), []);
  assert.deepEqual(parsePath('/items/1?bad=%GG'), ['items', '1']);
  const nested = good([...base, 'app/[tenant]/[__proto__]/page.tsrx'], {});
  const params = JSON.parse('{"tenant":"one","__proto__":"two"}');
  assert.deepEqual(matchRoute(nested, buildPath(nested, 'app/[tenant]/[__proto__]', params)).params, params);
});

test('malformed and ambiguous URLs/parameters fail rather than normalize silently', () => {
  for (const url of ['', 'https://example.com/items', '//items', '/items//', '/items\\x', '/items/.', '/items/..', '/items/%2e%2e', '/items/%2f', '/items/%5c', '/items/%25', '/items/%252f', '/items/%00', '/items/%C2%85', '/items/%GG', '/items/%C0%AF', '/items/%ED%A0%80', '/items/\ud800', '/items/\n']) {
    assert.throws(() => parsePath(url), undefined, JSON.stringify(url));
  }
  const manifest = good();
  for (const params of [{}, {id:'1', extra:'x'}, {id:1}, {id:''}, {id:'/'}, {id:'%'}, {id:'..'}, {id:'\ud800'}, null, []]) assert.throws(() => buildPath(manifest, 'app/items/[id]', params));
  assert.throws(() => buildPath(manifest, 'missing'));
  assert.throws(() => matchRoute(manifest, '/items', 'get'));
});

test('permutations preserve byte-identical manifests and diagnostics', () => {
  const expected = JSON.stringify(compileManifest(files, handlers));
  const reverseHandlers = Object.fromEntries(Object.entries(handlers).reverse());
  for (let i = 0; i < files.length; i++) {
    const rotated = [...files.slice(i), ...files.slice(0, i)].reverse();
    assert.equal(JSON.stringify(compileManifest(rotated, reverseHandlers)), expected);
  }
  const invalid = [...files, 'app/items/[other]/page.tsrx', 'app/orphan/loader.tsr'];
  assert.equal(JSON.stringify(compileManifest(invalid, handlers)), JSON.stringify(compileManifest([...invalid].reverse(), reverseHandlers)));
});

test('discovery never parses, executes or changes source bytes', t => {
  const root = temporary(t, [...base, 'app/page.tsrx', 'app/route.tsr'].map(p => [p, 'not valid source {\n']));
  const sentinel = path.join(root, 'executed');
  const content = `import {writeFileSync} from 'node:fs'; writeFileSync(${JSON.stringify(sentinel)}, 'bad'); throw new Error('executed');\n`;
  writeFileSync(path.join(root, 'app/route.tsr'), content);
  const before = [...base, 'app/page.tsrx', 'app/route.tsr'].map(p => readFileSync(path.join(root, p)));
  assert.deepEqual(discover(root, {'app/route.tsr': [{method:'POST', path:'/'}]}).diagnostics, []);
  assert.equal(existsSync(sentinel), false);
  assert.deepEqual([...base, 'app/page.tsrx', 'app/route.tsr'].map(p => readFileSync(path.join(root, p))), before);
});

test('symlink files, directories and app root are rejected without traversal', t => {
  for (const kind of ['file', 'directory', 'root']) {
    const root = temporary(t, [...base, 'outside/page.tsrx'].map(p => [p, '// placeholder\n']));
    if (kind === 'root') {
      rmSync(path.join(root, 'app'), {recursive:true});
      symlinkSync(path.join(root, 'outside'), path.join(root, 'app'));
    } else symlinkSync(path.join(root, kind === 'file' ? 'outside/page.tsrx' : 'outside'), path.join(root, 'app/link'));
    const result = discover(root);
    assert.equal(result.manifest, null);
    assert.ok(result.diagnostics.some(d => d.code === 'unsupported-entry'));
  }
  const missing = temporary(t, []);
  assert.ok(discover(missing).diagnostics.some(d => d.code === 'unreadable-entry'));
});

test('CLI demo/inspect are reproducible; errors exit nonzero', () => {
  const cli = fileURLToPath(new URL('../cli.mjs', import.meta.url));
  const demo = spawnSync(process.execPath, [cli, 'demo'], {encoding:'utf8'});
  assert.equal(demo.status, 0, demo.stderr);
  const output = JSON.parse(demo.stdout);
  assert.equal(output.manifest.schemaVersion, 'ui-route-manifest/0.1');
  assert.ok(output.collision.diagnostics.some(d => d.code === 'method-conflict'));
  const fixture = fileURLToPath(new URL('../fixtures/editable-list', import.meta.url));
  const inspected = spawnSync(process.execPath, [cli, 'inspect', fixture, path.join(fixture, 'handlers.json')], {encoding:'utf8'});
  assert.equal(inspected.status, 0, inspected.stderr);
  assert.deepEqual(JSON.parse(inspected.stdout).manifest, output.manifest);
  const invalid = spawnSync(process.execPath, [cli, 'unknown'], {encoding:'utf8'});
  assert.equal(invalid.status, 1);
  assert.match(invalid.stderr, /Usage:/);
});


test('empty unsupported directories are diagnosed; filesystem order and root do not affect output', t => {
  const entries = files.map(p => [p, '// Inert fixture.\n']);
  const first = temporary(t, entries);
  const second = temporary(t, [...entries].reverse());
  assert.equal(JSON.stringify(discover(first, handlers)), JSON.stringify(discover(second, handlers)));
  mkdirSync(path.join(first, 'app/[...slug]'));
  const result = discover(first, handlers);
  assert.equal(result.manifest, null);
  assert.ok(result.diagnostics.some(d => d.code === 'unsupported-segment' && d.sources[0] === 'app/[...slug]'));
});
