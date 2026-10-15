// Declaration discovery only. Native behavior and ownership require separate witnesses.
import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(import.meta.url);
const ts = require(path.join(root, 'benchmarks/runtime/node_modules/typescript'));
assert.equal(ts.version, '5.9.3', 'Review the reference inventory when changing TypeScript');
const filename = path.join(root, 'type-inventory-probe.ts');
const options = {strict: true, target: ts.ScriptTarget.ES2022};
let source = '', version = 0;
const host = {
  getScriptFileNames: () => [filename], getScriptVersion: () => String(version),
  getScriptSnapshot: file => {
    const text = file === filename ? source : ts.sys.readFile(file);
    return text === undefined ? undefined : ts.ScriptSnapshot.fromString(text);
  },
  getCurrentDirectory: () => root, getCompilationSettings: () => options,
  getDefaultLibFileName: settings => ts.getDefaultLibFilePath(settings),
  fileExists: ts.sys.fileExists, readFile: ts.sys.readFile, readDirectory: ts.sys.readDirectory,
};
const service = ts.createLanguageService(host);
const text = parts => ts.displayPartsToString(parts);
function set(probe) { source = probe; version++; }
const probes = [
  ['boolean', 'const value = true;', 'value'],
  ['int reference', 'const value = 1;', 'value'],
  ['number', 'const value = 1.5;', 'value'],
  ['bigint', 'const value = 1n;', 'value'],
  ['string', "const value = 'hello';", 'value'],
  ['Date', 'const value = new Date(0);', 'value'],
  ['Uint8Array', 'const value = new Uint8Array(4);', 'value'],
  ['Array', 'const value: number[] = [];', 'value'],
  ['tuple', 'const value: [number, string] = [1, "x"];', 'value'],
  ['Map', 'const value = new Map<string, number>();', 'value'],
  ['Set', 'const value = new Set<number>();', 'value'],
  ['Promise', 'const value = Promise.resolve(1);', 'value'],
  ['JSON', '', 'JSON'],
  ['null', 'const value = null;', 'value'],
  ['undefined', 'const value = undefined;', 'value'],
];
const rows = [];
try {
  for (const [type, declaration, expression] of probes) {
    set(`${declaration}\n${expression};\n`);
    const position = source.lastIndexOf(expression);
    const hover = service.getQuickInfoAtPosition(filename, position);
    const diagnostics = service.getSemanticDiagnostics(filename).map(d => ({code:d.code, message:ts.flattenDiagnosticMessageText(d.messageText, '\n')}));
    set(`${declaration}\n${expression}.`);
    const completion = service.getCompletionsAtPosition(filename, source.length, {});
    const members = (completion?.entries ?? []).map(entry => {
      const detail = service.getCompletionEntryDetails(filename, source.length, entry.name, {}, entry.source, {}, entry.data);
      return {name:entry.name, kind:entry.kind, signature:text(detail?.displayParts), documentation:text(detail?.documentation)};
    }).sort((a,b) => a.name.localeCompare(b.name));
    rows.push({type, hover:text(hover?.displayParts), diagnostics, members});
  }
  const signatures = [];
  for (const expression of ['new Date(', 'Number(', 'BigInt(', 'new Uint8Array(', 'JSON.parse(', 'JSON.stringify(']) {
    set(expression + ');');
    const help = service.getSignatureHelpItems(filename, expression.length, undefined);
    signatures.push({expression, overloads:(help?.items ?? []).map(item => ({
      prefix:text(item.prefixDisplayParts), parameters:item.parameters.map(p => ({name:p.name, display:text(p.displayParts), optional:p.isOptional})),
      suffix:text(item.suffixDisplayParts), variadic:item.isVariadic,
    }))});
  }
  assert.ok(rows.find(row => row.type === 'number').members.some(member => member.name === 'toFixed'));
  assert.ok(rows.find(row => row.type === 'Date').members.some(member => member.name === 'getTime'));
  assert.ok(signatures.find(row => row.expression === 'new Date(').overloads.length > 1);
  const library = ts.getDefaultLibFilePath(options);
  console.log(JSON.stringify({schemaVersion:1, status:'declaration discovery only; runtime and Tsrust classification require witnesses',
    engine:'TypeScript language service', version:ts.version, target:'ES2022',
    library, libraries:service.getProgram().getSourceFiles().filter(file => file.isDeclarationFile).map(file => ({path:file.fileName,sha256:createHash('sha256').update(file.text).digest('hex')})), rows, signatures}, null, 2));
} finally { service.dispose(); }
