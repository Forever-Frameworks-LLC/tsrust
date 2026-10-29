// Generates the standard-library and built-in API reference from the catalog the
// language server uses for completion, hover and parameter help. Output is ignored
// by Git and rebuilt before every site build, so the reference cannot drift from
// editor documentation. An unmapped catalog group fails the build instead of
// silently disappearing from the public reference.
import { mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const website = join(dirname(fileURLToPath(import.meta.url)), '..');
const catalogPath = join(website, '..', 'crates', 'tsrust-lsp', 'src', 'builtins.json');
const outputRoot = join(website, 'src', 'content', 'docs', 'reference');
const catalog = JSON.parse(readFileSync(catalogPath, 'utf8'));

// Groups that are not part of the backend public surface. std:ui belongs to the
// frozen UI product; the testing runtime is compiler-internal.
const excluded = new Set(['std:ui', 'UiComputed', 'UiSignal', 'std:testing-runtime', 'HttpContextNext']);

// Public modules in reading order. `members` maps a public type name to the
// catalog group holding its methods and properties.
const modules = [
  { module: 'std:env', summary: 'Read command-line arguments and environment variables.' },
  { module: 'std:path', summary: 'Join, split and normalize path text without touching the filesystem.' },
  { module: 'std:fs', summary: 'Open, read, write and buffer files with explicit errors.' },
  { module: 'std:bytes', summary: 'Owned byte buffers and borrowed byte views.' },
  { module: 'std:json', summary: 'Owned JSON values, borrowed views, raw fragments and JSON Lines files.' },
  { module: 'std:schema', summary: 'Build schemas that validate, parse and transform values.', members: {
    Schema: 'Schema', 'Schema pipeline methods': 'SchemaPipeline', ValidationContext: 'ValidationContext',
    's (schema builder)': 'schemaBuilder', 'Array schemas': 'SchemaArray', 'Boolean schemas': 'SchemaBoolean',
    'Int schemas': 'SchemaInt', 'Number schemas': 'SchemaNumber', 'Null schemas': 'SchemaNull',
    'Object schemas': 'SchemaObject', 'Retained schemas': 'SchemaRetained', 'Presence schemas': 'SchemaPresence',
    'Literal schemas': 'SchemaLiteral', 'Enum schemas': 'SchemaEnum', 'Declared enum schemas': 'SchemaDeclaredEnum',
    'Union schemas': 'SchemaUnion',
  } },
  { module: 'std:http', summary: 'Serve HTTP applications and send HTTP requests.', members: {
    Router: 'HttpRouter', Context: 'HttpContext', App: 'HttpApp', Request: 'HttpRequest', Response: 'HttpResponse',
    Body: 'HttpBody', Server: 'HttpServer', Client: 'HttpClient', Next: 'HttpNext', ServerOptions: 'HttpServerOptions',
    FormField: 'HttpFormField', MultipartPart: 'HttpMultipartPart',
  } },
  { module: 'std:sql', summary: 'Pooled PostgreSQL, MySQL and SQLite access with bound parameters.', members: {
    Pool: 'SqlPool', Transaction: 'SqlTransaction', Row: 'SqlRow', Cursor: 'SqlCursor', SqlError: 'SqlSqlError',
  } },
  { module: 'std:sqlite', summary: 'Synchronous embedded SQLite connections and statements.', members: {
    Connection: 'SqliteConnection', Transaction: 'SqliteTransaction', Statement: 'SqliteStatement', Row: 'SqliteRow',
    SqliteError: 'SqliteSqliteError',
  } },
  { module: 'std:futures', summary: 'Delay, race and cancel async tasks; bounded async channels.', members: {
    'Task methods': 'TaskMethods', Job: 'Job', Channel: 'AsyncChannel', Sender: 'AsyncSender',
  } },
  { module: 'std:threads', summary: 'Native threads, scoped threads, thread pools and blocking channels.', members: {
    Thread: 'Thread', ScopedThread: 'ScopedThread', ThreadPool: 'ThreadPool', Channel: 'Channel', Sender: 'Sender',
    TaskContext: 'TaskContext',
  } },
  { module: 'std:testing', summary: 'Register tests, hooks and assertions for `tsrust test`.', members: {
    Expectation: 'Expectation', 'Negated expectation': 'NegatedExpectation',
  } },
  { module: 'std:collections', summary: 'Ordered maps and sets, double-ended queues and heaps.', members: {
    OrderedMap: 'OrderedMap', OrderedSet: 'OrderedSet', Deque: 'Deque', Heap: 'Heap',
  } },
  { module: 'std:cache', summary: 'Bounded least-recently-used caches.', members: {
    LruCache: 'LruCache', SharedLruCache: 'SharedLruCache', CacheEntry: 'CacheEntry', CacheInsertError: 'CacheInsertError',
  } },
  { module: 'std:time', summary: 'Monotonic clock readings for measuring elapsed time.' },
  { module: 'std:date', summary: 'Calendar dates, times of day, instants, durations and time zones.' },
  { module: 'std:numeric', summary: 'Checked conversions between int and number widths.' },
  { module: 'std:decimal', summary: 'Exact decimal arithmetic and formatting.' },
  { module: 'std:uuid', summary: 'Parse, format, compare and generate UUIDs.' },
  { module: 'std:crypto', summary: 'Secure random bytes, SHA-256 and constant-time comparison.' },
  { module: 'std:os', summary: 'Inspect the host platform, architecture and byte order.' },
  { module: 'std:signals', summary: 'Receive process termination signals for graceful shutdown.' },
  // unwrap is global; this module only keeps an older import spelling working.
  { module: 'std:result', summary: 'Compatibility import of the global `unwrap`; new code needs no import.' },
];

// Members available without imports. Built-in types hold methods of values and
// the static helpers named after those types; globals hold free-standing names
// and namespaces. Each group is [catalog group, heading prefix, names?]; the
// optional names select part of a catalog group, and prefixes keep anchors unique.
const FUNCTIONS = ['clone', 'Ok', 'Err', 'unwrap', 'Number', 'String', 'Boolean', 'parseInt', 'parseFloat', 'isNaN', 'isFinite'];
const TYPES = ['Result', 'Task', 'Function', 'ReadOnly', 'Mutable', 'Owned', 'Once'];
const VALUES = ['null', 'undefined', 'NaN', 'Infinity', 'typeof', 'void'];
const builtins = [
  { dir: 'builtins', slug: 'strings', title: 'Strings', summary: 'String methods and properties, and static String helpers.', groups: { 'string methods': ['string', 'string'], 'Static String helpers': ['StringConstructor', 'String'] } },
  { dir: 'builtins', slug: 'arrays', title: 'Arrays and tuples', summary: 'Array methods, static Array helpers and tuple length.', groups: { 'Array methods': ['Array', 'array'], 'Static Array helpers': ['ArrayConstructor', 'Array'], 'Array namespace': ['globals', 'Array', ['Array']], Tuple: ['Tuple', 'tuple'] } },
  { dir: 'builtins', slug: 'numbers', title: 'Numbers, ints and booleans', summary: 'Methods on number, int and boolean values.', groups: { 'number methods': ['number', 'number'], 'int methods': ['int', 'int'], 'boolean methods': ['boolean', 'boolean'] } },
  { dir: 'builtins', slug: 'map-and-set', title: 'Map and Set', summary: 'Built-in hash maps and sets.', groups: { 'Map constructor': ['globals', 'Map', ['Map']], Map: ['Map', 'map'], 'Static Map helpers': ['MapConstructor', 'Map'], 'Set constructor': ['globals', 'Set', ['Set']], Set: ['Set', 'set'] } },
  { dir: 'globals', slug: 'functions', title: 'Global functions', summary: 'clone, Result constructors, unwrap, conversions and number parsing.', groups: { Functions: ['globals', '', FUNCTIONS] } },
  { dir: 'globals', slug: 'types', title: 'Global types', summary: 'Result, Task, Function and the ownership wrappers ReadOnly, Mutable, Owned and Once.', groups: { Types: ['globals', '', TYPES] } },
  { dir: 'globals', slug: 'values-and-operators', title: 'Values and operators', summary: 'null, undefined, NaN, Infinity, typeof and void.', groups: { 'Values and operators': ['globals', '', VALUES] } },
  { dir: 'globals', slug: 'console', title: 'console', summary: 'Write to standard output and standard error, count, time and group output.', groups: { console: ['console', 'console'] } },
  { dir: 'globals', slug: 'json', title: 'JSON', summary: 'Parse and serialize JSON text, with optional checked typed decoding.', groups: { JSON: ['JSON', 'JSON'] } },
  { dir: 'globals', slug: 'math', title: 'Math', summary: 'Mathematical constants and functions on numbers.', groups: { Math: ['Math', 'Math'] } },
  { dir: 'globals', slug: 'number', title: 'Number', summary: 'Number constants and static checks such as Number.isInteger.', groups: { Number: ['Number', 'Number'] } },
  { dir: 'globals', slug: 'object', title: 'Object', summary: 'Inspect record fields and compare values with Object helpers.', groups: { 'Object namespace': ['globals', 'Object', ['Object']], Object: ['Object', 'Object'] } },
];

const claimed = new Set([...excluded, ...modules.map((entry) => entry.module)]);
for (const entry of modules) for (const group of Object.values(entry.members ?? {})) claimed.add(group);
for (const page of builtins) for (const [group] of Object.values(page.groups)) claimed.add(group);
const globalsUsed = builtins.flatMap((page) => Object.values(page.groups).filter(([group]) => group === 'globals').flatMap(([, , names]) => names ?? []));
const globalsMissing = catalog.globals.map((item) => item.name).filter((name) => globalsUsed.filter((used) => used === name).length !== 1);
if (globalsMissing.length) throw new Error(`Each global must appear on exactly one reference page: ${globalsMissing.join(', ')}`);
const missing = Object.keys(catalog).filter((group) => !claimed.has(group));
const unknown = [...claimed].filter((group) => !(group in catalog));
if (missing.length || unknown.length) {
  throw new Error(`API reference mapping is out of date. Unmapped: ${missing.join(', ') || 'none'}. Unknown: ${unknown.join(', ') || 'none'}.`);
}

// Catalog prose is Markdown written for editor hovers. Escape angle brackets and
// braces outside code so generic signatures in prose are not parsed as HTML.
// Many entries share one catalog example. Show each example once per page and
// link back to it afterwards. `shown` is reset for every page.
let shown = new Map();
let rendered = new Map();
let anchor = '';
const anchorOf = (heading) => heading.toLowerCase().replace(/`/g, '').replace(/[^\w\s-]/g, '').trim().replace(/\s/g, '-');
const exampleKey = (source) => source.replace(/^```\w*\n|```$/g, '').replace(/\s+/g, '');
function once(source, render) {
  const key = exampleKey(source);
  if (!shown.has(key)) { shown.set(key, anchor); return render(); }
  const [target, label] = shown.get(key);
  return `See the example under [${label}](#${target}).`;
}
function prose(text) {
  return text.split(/(```[\s\S]*?```)/).map((part, index) => index % 2 ? once(part, () => fence(part)) : part
    .split(/(`[^`\n]*`)/)
    .map((piece, inner) => inner % 2 ? piece : piece.replaceAll('<', '&lt;').replaceAll('>', '&gt;'))
    .join('')).join('').trim();
}
function fence(block) { return block.replace(/^```(tsr|tsrust)?\n/, '```tsrust\n'); }
function code(text, lang = 'tsrust') { return `\`\`\`${lang}\n${text.replace(/\n+$/, '')}\n\`\`\``; }
function inline(text) { return text.includes('`') ? `\`\` ${text} \`\`` : `\`${text}\``; }
// Regression witnesses in the catalog are often minified; show only readable ones.
function readable(example) { return example.source.split('\n').filter((line) => line.trim()).length >= 3 && example.source.split('\n').every((line) => line.length <= 100); }

// Hover text may carry JSDoc-style `@param name - text` and `@returns text`
// paragraphs. Pull them out so they render as parameter and return lists.
function tags(documentation) {
  const params = new Map();
  let returns = '';
  const kept = [];
  let current = null;
  const blocks = documentation.split(/(```[\s\S]*?```)/);
  for (const [index, block] of blocks.entries()) {
    if (index % 2) { kept.push(block); current = null; continue; }
    const out = [];
    for (const line of block.split('\n')) {
      const param = line.match(/^@param\s+(?:\.\.\.)?([\w$]+)\??\s*(?:-\s*)?(.*)$/);
      const result = line.match(/^@returns?\s+(.*)$/);
      if (param) { current = { kind: 'param', name: param[1] }; params.set(param[1], param[2]); }
      else if (result) { current = { kind: 'returns' }; returns = result[1]; }
      else if (current && line.trim()) {
        if (current.kind === 'param') params.set(current.name, `${params.get(current.name)} ${line.trim()}`);
        else returns += ` ${line.trim()}`;
      } else { current = null; out.push(line); }
    }
    kept.push(out.join('\n'));
  }
  return { params, returns: returns.trim(), documentation: kept.join('').replace(/\n{3,}/g, '\n\n').trim() };
}
const parameterName = (label) => label.replace(/^\.\.\./, '').match(/^[\w$]+/)?.[0];

function entry(item, owner) {
  const heading = owner ? `${owner}.${item.name}` : item.name;
  anchor = [anchorOf(heading), heading];
  const lines = [`### ${heading}`, '', code(item.signature)];
  const parsed = tags(item.documentation ?? '');
  // Lead with the summary paragraph, then parameters and return value.
  const [lead, ...rest] = parsed.documentation.startsWith('```') ? ['', parsed.documentation] : parsed.documentation.split(/\n\n(.*)/s);
  if (lead) lines.push('', prose(lead));
  if (item.parameters?.length) {
    lines.push('', '**Parameters**', '');
    // Parameters are either bare labels or { label, documentation } records.
    for (const parameter of item.parameters) {
      const label = typeof parameter === 'string' ? parameter : parameter.label;
      // Some single-parameter entries reuse the operation summary; skip repeats.
      let text = typeof parameter === 'string' || !parameter.documentation || parsed.documentation.includes(parameter.documentation.slice(0, 80)) ? '' : parameter.documentation;
      text ||= parsed.params.get(parameterName(label)) ?? '';
      lines.push(`- ${inline(label)}${text ? ` — ${prose(text).replace(/\n+/g, ' ')}` : ''}`);
    }
  }
  if (parsed.returns) lines.push('', `**Returns:** ${prose(parsed.returns)}`);
  item = { ...item, documentation: rest.join('') };
  if (item.documentation) lines.push('', prose(item.documentation));
  const example = (item.examples ?? []).find((candidate) => readable(candidate) && !shown.has(exampleKey(candidate.source)));
  if (example) {
    shown.set(exampleKey(example.source), anchor);
    lines.push('', '**Example**', '', code(example.source));
    if (example.output) lines.push('', 'Output:', '', code(example.output, 'text'));
  }
  return `${lines.join('\n')}\n`;
}

// Schema and similar families repeat inherited methods with identical text on
// every type. Render each once per page and list the repeats as links.
function members_(group, prefix) {
  const out = [];
  const inherited = [];
  for (const item of catalog[group]) {
    const key = `${item.name}\u0000${item.signature}\u0000${item.documentation}`;
    if (rendered.has(key)) { inherited.push(`[\`${item.name}\`](#${rendered.get(key)})`); continue; }
    rendered.set(key, anchorOf(prefix ? `${prefix}.${item.name}` : item.name));
    out.push(entry(item, prefix));
  }
  if (inherited.length) out.push(`Also available, with the same behavior as documented above: ${inherited.join(', ')}.\n`);
  return out;
}

function isType(item) { return /^(type|interface|class|enum)\s/.test(item.signature); }

function page({ title, description, slug, order, intro, sections }) {
  const body = [
    '---', `title: ${JSON.stringify(title)}`, `description: ${JSON.stringify(description)}`,
    'editUrl: false', `sidebar:\n  order: ${order}`, '---', '',
    intro, '',
    ':::note\nThese entries match editor hover and parameter help for the same compiler version.\n:::',
    '', ...sections, '',
  ].join('\n');
  return [slug, body];
}

const pages = [];
modules.forEach(({ module, summary, members = {} }, index) => {
  const items = catalog[module];
  // Declarations with a member section (types and constructors) render there.
  const functions = items.filter((item) => !isType(item) && !(item.name in members));
  const types = items.filter(isType);
  const sections = [];
  shown = new Map();
  rendered = new Map();
  const name = module.slice(4);
  if (functions.length) sections.push('## Exports', '', ...functions.map((item) => entry(item)), '');
  // A type with methods gets its own section: declaration first, then members.
  const standalone = types.filter((item) => !(item.name in members));
  if (standalone.length) sections.push('## Types', '', ...standalone.map((item) => entry(item)), '');
  for (const [owner, group] of Object.entries(members)) {
    const declaration = items.find((item) => item.name === owner);
    if (!catalog[group].length && !declaration) continue;
    const prefix = /^[A-Za-z]+$/.test(owner) ? owner : '';
    sections.push(`## ${owner}`, '');
    anchor = [anchorOf(owner), owner];
    if (declaration && isType(declaration)) sections.push(prose(declaration.documentation), '', code(declaration.signature), '');
    else if (declaration) sections.push(entry({ ...declaration, name: `new ${owner}` }));
    sections.push(...members_(group, prefix), '');
  }
  const intro = `${summary}\n\n${code(`import { ${functions.concat(types).slice(0, 3).map((item) => item.name).filter((item) => /^[A-Za-z_$][\w$]*$/.test(item)).join(', ')} } from '${module}';`)}`;
  pages.push(page({ title: module, description: summary, slug: `library/${name}`, order: index + 1, intro, sections}));
});
builtins.forEach(({ dir, slug, title, summary, groups }, index) => {
  const sections = [];
  shown = new Map();
  rendered = new Map();
  for (const [owner, [group, prefix, names]] of Object.entries(groups)) {
    const items = names ? names.map((name) => catalog[group].find((item) => item.name === name)) : catalog[group];
    // A single-section page needs no repeated section heading.
    if (Object.keys(groups).length > 1) sections.push(`## ${owner}`, '');
    sections.push(...items.map((item) => entry(item, prefix)), '');
  }
  const intro = dir === 'globals'
    ? `${summary} These names are available in every module without an import. The [global functions and types](../../../library/globals/) guide introduces the most common ones.`
    : `${summary} These methods are available on every value of the type without an import.`;
  pages.push(page({ title, description: summary, slug: `${dir}/${slug}`, order: index + 1, intro, sections }));
});

rmSync(outputRoot, { recursive: true, force: true });
for (const [slug, body] of pages) {
  const file = join(outputRoot, `${slug}.md`);
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, body);
}
console.log(`Generated ${pages.length} API reference pages from ${Object.keys(catalog).length} catalog groups.`);
