import fs from 'node:fs';
import path from 'node:path';
import crypto from 'node:crypto';
import {fileURLToPath} from 'node:url';
import ts from '../runtime/node_modules/typescript/lib/typescript.js';

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, '../..');
const lib = path.join(root, 'benchmarks/runtime/node_modules/typescript/lib');
const pins = JSON.parse(fs.readFileSync(path.join(root, 'benchmarks/runtime/versions.json')));
if (ts.version !== pins.typescript) throw new Error('TypeScript declaration baseline drift');
const overrides = JSON.parse(fs.readFileSync(path.join(here, 'status.json')));
const language = JSON.parse(fs.readFileSync(path.join(here, 'language.json')));
const interfaces = {
 Numbers: ['Number', 'NumberConstructor', 'Math'],
 Booleans: ['Boolean', 'BooleanConstructor'],
 Strings: ['String', 'StringConstructor'],
 BigInt: ['BigInt', 'BigIntConstructor'],
 Symbols: ['Symbol', 'SymbolConstructor'],
 Arrays: ['Array', 'ArrayConstructor', 'ReadonlyArray'],
 Iterators: ['Iterator', 'IteratorObject', 'IteratorConstructor', 'Iterable', 'IterableIterator', 'AsyncIterator', 'AsyncIterable', 'AsyncIterableIterator', 'AsyncIteratorObject'],
 Generators: ['Generator', 'GeneratorFunction', 'GeneratorFunctionConstructor', 'AsyncGenerator', 'AsyncGeneratorFunction', 'AsyncGeneratorFunctionConstructor'],
 Objects: ['Object', 'ObjectConstructor', 'Function', 'FunctionConstructor', 'CallableFunction', 'NewableFunction', 'Reflect', 'ProxyHandler', 'ProxyConstructor'],
 Collections: ['Map', 'MapConstructor', 'ReadonlyMap', 'Set', 'SetConstructor', 'ReadonlySet', 'WeakMap', 'WeakMapConstructor', 'WeakSet', 'WeakSetConstructor', 'WeakRef', 'WeakRefConstructor', 'FinalizationRegistry', 'FinalizationRegistryConstructor'],
 Async: ['Promise', 'PromiseConstructor', 'PromiseLike'],
 Dates: ['Date', 'DateConstructor'],
 'Regular expressions': ['RegExp', 'RegExpConstructor', 'RegExpMatchArray', 'RegExpExecArray', 'RegExpIndicesArray'],
 Errors: ['Error', 'ErrorConstructor', 'EvalError', 'RangeError', 'ReferenceError', 'SyntaxError', 'TypeError', 'URIError', 'AggregateError', 'SuppressedError'],
 Serialization: ['JSON'],
 'Binary and typed arrays': ['ArrayBuffer', 'ArrayBufferConstructor', 'SharedArrayBuffer', 'SharedArrayBufferConstructor', 'DataView', 'DataViewConstructor', 'Atomics', ...['Int8', 'Uint8', 'Uint8Clamped', 'Int16', 'Uint16', 'Int32', 'Uint32', 'Float16', 'Float32', 'Float64', 'BigInt64', 'BigUint64'].flatMap(name => [`${name}Array`, `${name}ArrayConstructor`])],
 'Resource management': ['Disposable', 'AsyncDisposable', 'DisposableStack', 'DisposableStackConstructor', 'AsyncDisposableStack', 'AsyncDisposableStackConstructor'],
};
const categoryOf = new Map(Object.entries(interfaces).flatMap(([category, names]) => names.map(name => [name, category])));
const rows = new Map();
const declarations = [];
const digest = data => crypto.createHash('sha256').update(data).digest('hex');
for (const name of fs.readdirSync(lib).filter(name => /^lib\.es.*\.d\.ts$/.test(name)).sort()) {
 const bytes = fs.readFileSync(path.join(lib, name));
 const source = ts.createSourceFile(name, bytes.toString(), ts.ScriptTarget.Latest, true);
 let used = false;
 function visit(statements, namespace = '') {
 for (const statement of statements) {
  if (ts.isModuleDeclaration(statement) && ['Reflect', 'Intl'].includes(statement.name.text) && statement.body && ts.isModuleBlock(statement.body)) {
   visit(statement.body.statements, statement.name.text);
   continue;
  }
  if (namespace && ts.isFunctionDeclaration(statement) && statement.name) {
   used = true;
   const id = `${namespace}.${statement.name.text}`;
   if (!rows.has(id)) rows.set(id, {id, category: namespace === 'Intl' ? 'Internationalization' : 'Objects', kind: 'method', declarationSources: []});
   const entry = rows.get(id);
   if (!entry.declarationSources.includes(name)) entry.declarationSources.push(name);
   continue;
  }
  if (!ts.isInterfaceDeclaration(statement) || (!categoryOf.has(statement.name.text) && namespace !== 'Intl')) continue;
  used = true;
  const owner = namespace ? `${namespace}.${statement.name.text}` : statement.name.text;
  for (const member of statement.members) {
   const suffix = member.name ? member.name.getText(source) : ts.isConstructSignatureDeclaration(member) ? 'new()' : ts.isCallSignatureDeclaration(member) ? 'call()' : ts.isIndexSignatureDeclaration(member) ? '[index]' : undefined;
   if (!suffix) throw new Error(`Unclassified declaration member ${owner} in ${name}`);
   const id = `${owner}.${suffix}`;
   if (!rows.has(id)) rows.set(id, {id, category: namespace === 'Intl' ? 'Internationalization' : categoryOf.get(statement.name.text), kind: ts.isMethodSignature(member) ? 'method' : ts.isPropertySignature(member) ? 'property' : 'signature', declarationSources: []});
   const entry = rows.get(id);
   if (!entry.declarationSources.includes(name)) entry.declarationSources.push(name);
  }
 }
 }
 visit(source.statements);
 if (used) declarations.push({file: name, sha256: digest(bytes)});
}
for (const group of language) for (const operation of group.operations) {
 const [id, display = id] = Array.isArray(operation) ? operation : [operation];
 if (rows.has(id)) throw new Error(`Duplicate operation ${id}`);
 rows.set(id, {id, display, category: group.category, kind: 'language', declarationSources: []});
}
for (const id of Object.keys(overrides)) if (!rows.has(id)) throw new Error(`Unknown status override ${id}`);
const entries = [...rows.values()].map(row => ({...row, implementation: 'pending', comparison: 'missing', evidence: [], ...overrides[row.id]}));
const implementationStates = new Set(['pending', 'in progress', 'implemented', 'native variant', 'policy']);
for (const row of entries) if (!implementationStates.has(row.implementation)) throw new Error(`Unknown implementation state ${row.id}: ${row.implementation}`);
entries.sort((a, b) => a.category.localeCompare(b.category) || a.id.localeCompare(b.id));
if (new Set(entries.map(row => row.id)).size !== entries.length) throw new Error('Duplicate coverage IDs');
const workloadIDs = new Set(JSON.parse(fs.readFileSync(path.join(root, 'benchmarks/runtime/manifest.json'))).workloads.map(row => row.id));
for (const row of entries) for (const id of row.workloads ?? []) if (!workloadIDs.has(id)) throw new Error(`Unknown comparison ${row.id}: ${id}`);
const probeIDs = new Set(JSON.parse(fs.readFileSync(path.join(root, 'benchmarks/runtime/allocations/manifest.json'))).map(row => row.id));
for (const row of entries) {
 for (const id of row.allocationProbes ?? []) if (!probeIDs.has(id)) throw new Error(`Unknown allocation probe ${row.id}: ${id}`);
 // Closed runtime records live in SQLite; their loose copies were removed after verified import.
 for (const link of row.evidence ?? []) if (!/^\.\.\/benchmarks\/runtime\/(diagnostics|results)\//.test(link) && !fs.existsSync(path.resolve(root, 'docs', link))) throw new Error(`Missing evidence ${row.id}: ${link}`);
}
const data = {schemaVersion: 1, baseline: `TypeScript ${ts.version} ECMAScript declarations, including explicitly identified esnext declarations`, sourcePackage: 'https://github.com/microsoft/TypeScript/tree/v5.9.3/src/lib', declarationSources: declarations, entries};
fs.writeFileSync(path.join(here, 'inventory.json'), JSON.stringify(data, null, 2) + '\n');
const categories = [...new Set(entries.map(row => row.category))];
const slug = value => value.toLowerCase().replace(/[^a-z0-9]+/g, '-');
const lines = ['# Language and comparison coverage', '', 'This inventory tracks the complete selected TypeScript declaration surface plus explicit language constructs. It is a backlog, not a claim of JavaScript compatibility or completed benchmarks. Native Rust behavior and documented quirks remain the implementation direction.', '', `Baseline: pinned TypeScript ${ts.version}. Static declarations do not prove runtime availability. APIs found only in \`lib.esnext.*\` are proposal/declaration candidates requiring a runtime and specification check before implementation. Inherited operations appear under their declaring interface; method overloads share one row. Host/browser/Node APIs require separate inventories; Intl declarations are included.`, '', '**Implementation:** pending, in progress, implemented, native variant, or policy. **Comparison:** missing, compound (covered inside a broader workload), paired group (matched named workloads; measurements require separate checkpoint evidence), or compile-only. These states are independent. Pending operations require edge tests and independent counterparts before acceptance; a compound workload does not close an isolated operation comparison.', '', 'Full compiler regression and accumulated performance comparisons run at completed feature-family milestones after compiler, editor and benchmark support settles. Development uses focused checks; repairs rerun affected cases. SQLite retains accepted analytics and exact original records; Markdown reports are optional exports. Presentation changes do not rerun measurements.', '', '[Analytics storage](../benchmarks/storage/README.md) · [Historical benchmark snapshot](../BENCHMARKS.md) · [Machine-readable inventory](../benchmarks/coverage/inventory.json) · [Sources and update procedure](../benchmarks/coverage/README.md)', '', '## Contents', '', categories.map(category => `[${category}](#${slug(category)})`).join(' · '), '', '## Coverage summary', '', '| Family | Operations | Implemented or native variant | In progress | Comparison gaps |', '| --- | ---: | ---: | ---: | ---: |'];
for (const category of categories) {
 const values = entries.filter(row => row.category === category);
 lines.push(`| ${category} | ${values.length} | ${values.filter(row => ['implemented', 'native variant'].includes(row.implementation)).length} | ${values.filter(row => row.implementation === 'in progress').length} | ${values.filter(row => row.comparison === 'missing').length} |`);
}
for (const category of categories) {
 lines.push('', `## ${category}`, '', '| Operation | Implementation | Comparison | Scope / evidence |', '| --- | --- | --- | --- |');
 for (const row of entries.filter(row => row.category === category)) {
  const evidence = [...(row.evidence ?? []).map(link => `[evidence](${link})`), ...(row.workloads ?? []).map(id => `[${id}](../benchmarks/runtime/workloads/${id}.tsr)`), ...(row.allocationProbes ?? []).map(id => `[allocation: ${id}](../benchmarks/runtime/allocations/${id}.tsr)`)].join(' · ');
  const note = [row.note, row.declarationSources.length && row.declarationSources.every(name => name.startsWith('lib.esnext.')) ? 'esnext declaration; availability unverified' : '', evidence].filter(Boolean).join('; ').replaceAll('|', '\\|');
  lines.push(`| \`${(row.display ?? row.id).replaceAll('|', '\\|')}\` | ${row.implementation} | ${row.comparison} | ${note} |`);
 }
}
fs.writeFileSync(path.join(root, 'docs/guides/language/feature-coverage.md'), lines.join('\n') + '\n');
console.log(`Coverage updated: ${entries.length} operations across ${categories.length} families; unique IDs and comparison references verified.`);
