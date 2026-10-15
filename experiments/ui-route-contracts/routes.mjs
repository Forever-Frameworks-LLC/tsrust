import {lstatSync, readdirSync} from 'node:fs';
import path from 'node:path';

const special = ['layout.tsrx', 'template.tsrx', 'page.tsrx', 'loader.tsr', 'not-found.tsrx', 'error.tsrx', 'route.tsr', 'middleware.tsr'];
const wrappers = ['middleware', 'layout', 'template', 'not-found', 'error'];
const methods = new Set(['GET', 'HEAD', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS', 'TRACE', 'CONNECT']);
const compare = (a, b) => a < b ? -1 : a > b ? 1 : 0;
const record = value => value !== null && typeof value === 'object' && !Array.isArray(value);
const parameter = /^\[([A-Za-z_][A-Za-z0-9_]*)\]$/;

function validValue(value) {
  if (typeof value !== 'string' || !value || value === '.' || value === '..' || /[/\\%\u0000-\u001f\u007f-\u009f]/u.test(value)) return false;
  try { encodeURIComponent(value); return true; } catch { return false; }
}

function segmentPart(name) {
  const match = parameter.exec(name);
  if (match) return {parameter: match[1]};
  if (!validValue(name) || /[\[\]()@]/u.test(name)) throw new Error('Unsupported route segment');
  return {literal: name};
}

// shortcut: Discovery assumes a stable local tree; use descriptor-safe traversal before accepting hostile concurrent writes.
export function discover(projectRoot, declarations = {}) {
  const files = [];
  const diagnostics = [];
  const visit = relative => {
    const absolute = path.join(projectRoot, relative);
    let stat;
    try { stat = lstatSync(absolute); } catch {
      diagnostics.push({code: 'unreadable-entry', sources: [relative], message: 'Cannot inspect app entry'});
      return;
    }
    if (stat.isSymbolicLink() || (!stat.isDirectory() && !stat.isFile())) {
      diagnostics.push({code: 'unsupported-entry', sources: [relative], message: 'Symlinks and special filesystem entries are unsupported'});
    } else if (stat.isDirectory()) {
      if (relative !== 'app') {
        try { segmentPart(path.basename(relative)); } catch {
          diagnostics.push({code: 'unsupported-segment', sources: [relative], message: 'Only literal and [name] segments are supported'});
          return;
        }
      }
      let names;
      try { names = readdirSync(absolute).sort(compare); } catch {
        diagnostics.push({code: 'unreadable-entry', sources: [relative], message: 'Cannot read app directory'});
        return;
      }
      for (const name of names) visit(`${relative}/${name}`);
    } else if (relative === 'app') {
      diagnostics.push({code: 'invalid-root', sources: ['app'], message: 'app must be a directory'});
    } else files.push(relative);
  };
  visit('app');
  return compileManifest(files, declarations, diagnostics);
}

export function compileManifest(files, declarations = {}, initialDiagnostics = []) {
  const diagnostics = [...initialDiagnostics];
  const issue = (code, sources, message) => diagnostics.push({code, sources: [...sources].sort(compare), message});
  const directories = new Map([['app', {}]]);
  const seen = new Set();
  for (const source of [...files].sort(compare)) {
    if (typeof source !== 'string' || !source.startsWith('app/') || source.includes('\\') || source.split('/').some(p => !p || p === '.' || p === '..')) {
      issue('invalid-source', [String(source)], 'Expected an app-relative POSIX file path');
      continue;
    }
    if (seen.has(source)) { issue('duplicate-source', [source], 'Duplicate source identity'); continue; }
    seen.add(source);
    const parts = source.split('/');
    const filename = parts.pop();
    for (let i = 1; i <= parts.length; i++) {
      const id = parts.slice(0, i).join('/');
      if (!directories.has(id)) directories.set(id, {});
    }
    if (special.includes(filename)) directories.get(parts.join('/'))[filename.split('.')[0]] = source;
    else if (/^(layout|template|page|loader|not-found|error|route|middleware|loading|default|global-error)\.(tsr|tsrx)$/.test(filename)) {
      issue('unsupported-convention', [source], 'Unsupported special file or extension');
    }
  }
  if (!directories.get('app')['not-found']) issue('missing-not-found', ['app/not-found.tsrx'], 'Create the visible root not-found source');
  const segments = [];
  const shapes = new Map();
  for (const [id, sources] of [...directories].sort(([a], [b]) => compare(a, b))) {
    let parts;
    try { parts = id.split('/').slice(1).map(segmentPart); } catch {
      issue('unsupported-segment', [id], 'Only literal and [name] segments are supported'); continue;
    }
    const params = parts.filter(p => p.parameter).map(p => p.parameter);
    if (new Set(params).size !== params.length) issue('duplicate-parameter', [id], 'Parameter names must be unique within a path');
    const pattern = '/' + parts.map(p => p.parameter ? `[${p.parameter}]` : encodeURIComponent(p.literal)).join('/');
    const shape = '/' + parts.map(p => p.parameter ? '[]' : encodeURIComponent(p.literal)).join('/');
    if (shapes.has(shape)) issue('equivalent-pattern', [shapes.get(shape), id], 'Equivalent parameter patterns conflict');
    else shapes.set(shape, id);
    const ancestors = id.split('/').map((_, i, all) => all.slice(0, i + 1).join('/'));
    const ancestry = Object.fromEntries(wrappers.map(kind => [kind, ancestors.map(a => directories.get(a)?.[kind]).filter(Boolean)]));
    if (sources.loader && !sources.page) issue('unpaired-loader', [sources.loader], 'loader.tsr requires a colocated page.tsrx');
    segments.push({id, pattern, parts, params, sources: Object.fromEntries(Object.entries(sources).sort(([a], [b]) => compare(a, b))), ancestry,
      page: sources.page ? {source: sources.page, loader: sources.loader ?? null} : null, endpoints: []});
  }
  if (!record(declarations)) {
    issue('invalid-declarations', [], 'Handler declarations must be an object keyed by route source');
    declarations = {};
  }
  const routeFiles = new Set(segments.map(s => s.sources.route).filter(Boolean));
  for (const source of Object.keys(declarations).sort(compare)) {
    if (!routeFiles.has(source)) issue('unknown-handler-source', [source], 'Handler declarations reference no discovered route.tsr');
  }
  for (const segment of segments) {
    const owners = new Map();
    const add = (method, kind, source) => {
      if (owners.has(method)) issue('method-conflict', [owners.get(method).source, source], `${method} ${segment.pattern} has multiple owners`);
      else owners.set(method, {method, kind, source});
    };
    if (segment.page) for (const method of ['GET', 'HEAD']) add(method, 'page', segment.page.source);
    const source = segment.sources.route;
    if (source) {
      if (!Object.hasOwn(declarations, source)) issue('missing-handlers', [source], 'Supply explicit mock checked handler declarations');
      else if (!Array.isArray(declarations[source])) issue('invalid-handlers', [source], 'Expected an array of {method, path} declarations');
      else for (const handler of declarations[source]) {
        if (!record(handler) || Object.keys(handler).sort().join(',') !== 'method,path' || !methods.has(handler.method) || handler.path !== '/') {
          issue('invalid-handler', [source], 'Expected an uppercase HTTP method and current-segment path / only');
        } else add(handler.method, 'route', source);
      }
    }
    segment.endpoints = [...owners.values()].sort((a, b) => compare(a.method, b.method));
  }
  diagnostics.sort((a, b) => compare(JSON.stringify(a), JSON.stringify(b)));
  return {manifest: diagnostics.length ? null : {schemaVersion: 'ui-route-manifest/0.1', appRoot: 'app', notFound: 'app/not-found.tsrx', segments}, diagnostics};
}

export function parsePath(url) {
  if (typeof url !== 'string' || !url.startsWith('/') || /[\\\u0000-\u001f\u007f-\u009f]/u.test(url)) throw new Error('Invalid URL path');
  let pathname = url.split(/[?#]/, 1)[0];
  if (pathname.includes('//')) throw new Error('Repeated path separator');
  if (pathname.length > 1 && pathname.endsWith('/')) pathname = pathname.slice(0, -1);
  if (pathname === '/') return [];
  return pathname.slice(1).split('/').map(raw => {
    let value;
    try { value = decodeURIComponent(raw); } catch { throw new Error('Malformed path encoding'); }
    if (!validValue(value)) throw new Error('Invalid decoded path segment');
    return value;
  });
}

function selectSegment(manifest, values) {
  const candidates = manifest.segments.filter(s => s.endpoints.length && s.parts.length === values.length && s.parts.every((p, i) => p.parameter || p.literal === values[i]));
  candidates.sort((a, b) => {
    for (let i = 0; i < a.parts.length; i++) {
      const difference = Number(Boolean(a.parts[i].parameter)) - Number(Boolean(b.parts[i].parameter));
      if (difference) return difference;
    }
    return compare(a.id, b.id);
  });
  return candidates[0] ?? null;
}

export function matchRoute(manifest, url, method = 'GET') {
  if (!methods.has(method)) throw new Error('Expected an uppercase HTTP method');
  const values = parsePath(url);
  const segment = selectSegment(manifest, values);
  const endpoint = segment?.endpoints.find(e => e.method === method);
  if (!endpoint) return null;
  return {segmentId: segment.id, pattern: segment.pattern, endpoint,
    params: Object.fromEntries(segment.parts.flatMap((p, i) => p.parameter ? [[p.parameter, values[i]]] : []))};
}

export function buildPath(manifest, segmentId, params = {}) {
  const segment = manifest.segments.find(s => s.id === segmentId && s.endpoints.length);
  if (!segment) throw new Error('Unknown endpoint segment');
  if (!record(params) || Object.keys(params).length !== segment.params.length || Object.keys(params).some(key => !segment.params.includes(key))) throw new Error('Missing or extra path parameters');
  const values = segment.parts.map(p => p.parameter ? params[p.parameter] : p.literal);
  if (!values.every(validValue)) throw new Error('Invalid path parameter');
  if (selectSegment(manifest, values)?.id !== segment.id) throw new Error('Path is shadowed by a more specific route');
  return '/' + values.map(encodeURIComponent).join('/');
}
