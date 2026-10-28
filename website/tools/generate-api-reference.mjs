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
  { module: 'std:result', summary: 'Helpers for recoverable Result values.' },
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
];

// Members available without imports, grouped by what a reader looks for.
// Each group is [catalog group, heading prefix]; prefixes keep anchors unique.
const builtins = [
  { slug: 'strings', title: 'Strings', summary: 'String methods and properties, and the String namespace.', groups: { 'string methods': ['string', 'string'], String: ['StringConstructor', 'String'] } },
  { slug: 'arrays', title: 'Arrays and tuples', summary: 'Array methods, the Array namespace and tuple length.', groups: { 'Array methods': ['Array', 'array'], 'Array namespace': ['ArrayConstructor', 'Array'], Tuple: ['Tuple', 'tuple'] } },
  { slug: 'numbers', title: 'Numbers, ints and booleans', summary: 'Number and int methods, the Number namespace, Math and boolean methods.', groups: { 'number methods': ['number', 'number'], 'int methods': ['int', 'int'], 'Number namespace': ['Number', 'Number'], Math: ['Math', 'Math'], 'boolean methods': ['boolean', 'boolean'] } },
  { slug: 'map-and-set', title: 'Map and Set', summary: 'Built-in hash maps and sets.', groups: { Map: ['Map', 'map'], 'Map namespace': ['MapConstructor', 'Map'], Set: ['Set', 'set'] } },
  { slug: 'globals', title: 'Globals, Object, JSON and console', summary: 'Global functions and types, Object helpers, the JSON namespace and console output.', groups: { Globals: ['globals', ''], Object: ['Object', 'Object'], JSON: ['JSON', 'JSON'], console: ['console', 'console'] } },
];

const claimed = new Set([...excluded, ...modules.map((entry) => entry.module)]);
for (const entry of modules) for (const group of Object.values(entry.members ?? {})) claimed.add(group);
for (const page of builtins) for (const [group] of Object.values(page.groups)) claimed.add(group);
const missing = Object.keys(catalog).filter((group) => !claimed.has(group));
const unknown = [...claimed].filter((group) => !(group in catalog));
if (missing.length || unknown.length) {
  throw new Error(`API reference mapping is out of date. Unmapped: ${missing.join(', ') || 'none'}. Unknown: ${unknown.join(', ') || 'none'}.`);
}

// Catalog prose is Markdown written for editor hovers. Escape angle brackets and
// braces outside code so generic signatures in prose are not parsed as HTML.
function prose(text) {
  return text.split(/(```[\s\S]*?```)/).map((part, index) => index % 2 ? fence(part) : part
    .split(/(`[^`\n]*`)/)
    .map((piece, inner) => inner % 2 ? piece : piece.replaceAll('<', '&lt;').replaceAll('>', '&gt;'))
    .join('')).join('').trim();
}
function fence(block) { return block.replace(/^```(tsr|tsrust)?\n/, '```tsrust\n'); }
function code(text, lang = 'tsrust') { return `\`\`\`${lang}\n${text.replace(/\n+$/, '')}\n\`\`\``; }
function inline(text) { return text.includes('`') ? `\`\` ${text} \`\`` : `\`${text}\``; }
// Regression witnesses in the catalog are often minified; show only readable ones.
function readable(example) { return example.source.split('\n').filter((line) => line.trim()).length >= 3 && example.source.split('\n').every((line) => line.length <= 100); }

function entry(item, owner) {
  const heading = owner ? `${owner}.${item.name}` : item.name;
  const lines = [`### ${heading}`, '', code(item.signature)];
  if (item.parameters?.length) {
    lines.push('', '**Parameters**', '');
    // Parameters are either bare labels or { label, documentation } records.
    for (const parameter of item.parameters) {
      if (typeof parameter === 'string') { lines.push(`- ${inline(parameter)}`); continue; }
      // Some single-parameter entries reuse the operation summary; skip repeats.
      const repeated = !parameter.documentation || item.documentation.includes(parameter.documentation.slice(0, 80));
      lines.push(`- ${inline(parameter.label)}${repeated ? '' : ` — ${prose(parameter.documentation).replace(/\n+/g, ' ')}`}`);
    }
  }
  if (item.documentation) lines.push('', prose(item.documentation));
  const example = (item.examples ?? []).find((candidate) => readable(candidate) && !item.documentation.includes(candidate.source.trim()));
  if (example) {
    lines.push('', '**Example**', '', code(example.source));
    if (example.output) lines.push('', 'Output:', '', code(example.output, 'text'));
  }
  return `${lines.join('\n')}\n`;
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
    if (declaration && isType(declaration)) sections.push(prose(declaration.documentation), '', code(declaration.signature), '');
    else if (declaration) sections.push(entry({ ...declaration, name: `new ${owner}` }));
    sections.push(...catalog[group].map((item) => entry(item, prefix)), '');
  }
  const intro = `${summary}\n\n${code(`import { ${functions.concat(types).slice(0, 3).map((item) => item.name).filter((item) => /^[A-Za-z_$][\w$]*$/.test(item)).join(', ')} } from '${module}';`)}`;
  pages.push(page({ title: module, description: summary, slug: `library/${name}`, order: index + 1, intro, sections}));
});
builtins.forEach(({ slug, title, summary, groups }, index) => {
  const sections = [];
  for (const [owner, [group, prefix]] of Object.entries(groups)) {
    sections.push(`## ${owner}`, '', ...catalog[group].map((item) => entry(item, prefix)), '');
  }
  pages.push(page({ title, description: summary, slug: `builtins/${slug}`, order: index + 1, intro: `${summary} These are available in every module without an import.`, sections}));
});

rmSync(outputRoot, { recursive: true, force: true });
for (const [slug, body] of pages) {
  const file = join(outputRoot, `${slug}.md`);
  mkdirSync(dirname(file), { recursive: true });
  writeFileSync(file, body);
}
console.log(`Generated ${pages.length} API reference pages from ${Object.keys(catalog).length} catalog groups.`);
