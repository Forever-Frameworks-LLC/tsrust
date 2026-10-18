const { test } = require('node:test');
const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const { Registry, parseRawGrammar } = require('vscode-textmate');
const { loadWASM, OnigScanner, OnigString } = require('vscode-oniguruma');

test('real TextMate tokenization covers lexer syntax and multiline state', async () => {
  const wasm = fs.readFileSync(require.resolve('vscode-oniguruma/release/onig.wasm'));
  await loadWASM(wasm.buffer.slice(wasm.byteOffset, wasm.byteOffset + wasm.byteLength));
  const registry = new Registry({
    onigLib: Promise.resolve({ createOnigScanner: p => new OnigScanner(p), createOnigString: s => new OnigString(s) }),
    loadGrammar: async () => parseRawGrammar(fs.readFileSync(path.join(__dirname, '../syntaxes/tsrust.tmLanguage.json'), 'utf8'), 'tsrust.json'),
  });
  const grammar = await registry.loadGrammar('source.tsrust');
  const cases = [
    ['const node = <View className="p-4"><Text>{count}</Text></View>; const next = 1;', [['View','entity.name.function'],['className','entity.other.attribute-name'],['"p-4"','string.quoted.double'],['next','source.tsrust']]],
    ['async function answer(): Task<int> { return await work(); }', [['async','keyword.control.async'],['function','storage.type'],['Task','support.type.builtin'],['await','keyword.control']]],
    ['match (result) { Ok(value) => {} Err(error) => {} }', [['match', 'keyword.control'], ['Ok', 'support.function'], ['Err', 'support.function']]],
    ['const result: Result<int, string> = Ok<int, string>(5);', [['Result', 'support.type.builtin'], ['Ok', 'support.function']]],
    ["const whole=7_654_321;const bits=0b1010_0011;const oct=0o7_7;const hex=0xDE_AD;const fraction=.1_2e-0_1;", [["7_654_321","constant.numeric"],["0b1010_0011","constant.numeric"],["0o7_7","constant.numeric"],["0xDE_AD","constant.numeric"],[".1_2e-0_1","constant.numeric"]]],
    ['function identity<T>(value: T): T { return value; }', [['function', 'storage.type'], ['identity', 'entity.name.function'], ['<', 'keyword.operator']]],
    ['interface Box<T> { value: T; }', [['interface', 'storage.type'], ['Box', 'entity.name.type']]],
    ['type Box<T> = { value: T; };', [['type', 'storage.type'], ['<', 'keyword.operator']]],
    ["import { greet as hello, User } from './dep.tsr';", [['import', 'keyword.control.import'], ['as', 'keyword.control.import'], ['from', 'keyword.control.import'], ["'./dep.tsr'", 'string.quoted.single']]],
    ['export function greet(): int { return 1; }', [['export', 'keyword.control.export'], ['function', 'storage.type'], ['greet', 'entity.name.function']]],
    ['export type User = { name: string; };', [['export', 'keyword.control.export'], ['type', 'storage.type']]],
    ['const text = "import export from as"; // export', [['import', 'string.quoted.double'], ['// export', 'comment.line']]],
    ['if(typeof value === \"string\") {}', [['typeof','keyword.operator.expression']]],
    ['const kind = typeof(value); const empty = void 0;', [['typeof','keyword.operator.expression'],['void','keyword.operator.expression']]],
    ['function empty(): void {}', [['void','support.type.primitive']]],
    ['const c=-Infinity; const e=NaN; const a=parseInt("4")+parseFloat("1.5"); const b=isNaN(a)||isFinite(e); const d=Number.isNaN(a);', [['parseInt','support.function'],['parseFloat','support.function'],['isNaN','support.function'],['isFinite','support.function'],['NaN','constant.language'],['Infinity','constant.language']]],
    ['const n:null=null; const u=undefined; const t=typeof n; const v=void 0; Boolean(u); Array.isArray([]);', [['null','constant.language'],['undefined','constant.language'],['typeof','keyword.operator'],['void','keyword.operator'],['Boolean','support.class'],['Array','support.class']]],
    ['function main() { let n: number = .5e-2; }', [['function', 'storage.type'], ['main', 'entity.name.function'], ['number', 'support.type.primitive'], ['.5e-2', 'constant.numeric']]],
    ['const message = "😀 \\" hello"; // owned', [['\\"', 'constant.character.escape'], ['// owned', 'comment.line']]],
    ['const action: Mutable<Function<string, [user: ReadOnly<User>, text: Owned<string>]>> = make;', [['Mutable', 'support.type.builtin'], ['Function', 'support.type.builtin'], ['ReadOnly', 'support.type.builtin'], ['Owned', 'support.type.builtin'], ['string', 'support.type.primitive']]],
    ['const last: Once<Function<int>> = finish; const Function = 1;', [['Once', 'support.type.builtin'], ['int', 'support.type.primitive']]],
    ['try { throw 1; } catch (error) { switch (error) { case 1: break; default: } } finally { }', [['try', 'keyword.control'], ['throw', 'keyword.control'], ['catch', 'keyword.control'], ['switch', 'keyword.control'], ['case', 'keyword.control'], ['default', 'keyword.control'], ['finally', 'keyword.control']]],
    ['interface Player extends Named { score: int; }', [['interface', 'storage.type'], ['extends', 'storage.modifier'], ['Player', 'entity.name.type'], ['Named', 'entity.other.inherited-class'], ['score', 'source.tsrust']]],
    ['for (;;) { if (true) count &&= false; }', [['for', 'keyword.control'], ['true', 'constant.language'], ['&&=', 'keyword.operator']]],
  ];
  for (const [line, expectations] of cases) {
    const tokens = grammar.tokenizeLine(line).tokens;
    for (const [word, scope] of expectations) {
      const offset = line.indexOf(word);
      const token = tokens.find(t => t.startIndex <= offset && t.endIndex > offset);
      assert.ok(token?.scopes.some(s => s.startsWith(scope)), `${word} should have ${scope}: ${JSON.stringify(tokens)}`);
      if(scope === "constant.numeric") assert.ok(token.endIndex >= offset + word.length, `Numeric token must cover ${word}`);
    }
  }
  const first = grammar.tokenizeLine('/* function main');
  const second = grammar.tokenizeLine('still comment */ const x = 1;', first.ruleStack);
  assert.ok(second.tokens[0].scopes.includes('comment.block.tsrust'));
  const constToken = second.tokens.find(t => t.startIndex === 17);
  assert.ok(constToken?.scopes.includes('storage.type.tsrust'));
  let state;
  for (const line of ['import {', '  greet as hello,', '} from "./dep.tsr";']) {
    const result = grammar.tokenizeLine(line, state);
    state = result.ruleStack;
    for (const word of ['import', 'as', 'from']) {
      const offset = line.indexOf(word);
      if (offset < 0) continue;
      assert.ok(result.tokens.find(t => t.startIndex <= offset && t.endIndex > offset)
        ?.scopes.includes('keyword.control.import.tsrust'));
    }
  }
  const following = grammar.tokenizeLine('const from = 1; const as = 2;', state);
  assert.ok(following.tokens.every(t => !t.scopes.includes('meta.import.tsrust')
    && !t.scopes.includes('keyword.control.import.tsrust')));
  let interfaceState;
  for (const line of ['interface Player', '  extends Named, Positioned {', '  score: int;', '}']) {
    const result = grammar.tokenizeLine(line, interfaceState);
    interfaceState = result.ruleStack;
    for (const word of ['Player', 'Named', 'Positioned']) {
      const offset = line.indexOf(word);
      if (offset < 0) continue;
      const scope = word === 'Player' ? 'entity.name.type.interface.tsrust' : 'entity.other.inherited-class.tsrust';
      assert.ok(result.tokens.find(t => t.startIndex <= offset && t.endIndex > offset)?.scopes.includes(scope));
    }
  }
  assert.ok(grammar.tokenizeLine('const interfaceName = 1;', interfaceState).tokens.every(t => !t.scopes.includes('meta.interface.tsrust')));
  const referenceLines = [
    'interface Box<T> { value: T; label?: string; }',
    'interface Team { lead: Player; members: Player[]; }',
    'type Scores = Map<string, int>;',
    'interface Board extends Named { scores: Scores; }',
    'function describe(player: Player): Scores { return player; }',
    'const team: Box<Map<string, Player[]>> = source;',
  ];
  for (const line of referenceLines) {
    const tokens = grammar.tokenizeLine(line).tokens;
    for (const match of line.matchAll(/\b(?:Box|Team|Player|Scores|Board|T)\b/g)) {
      const token = tokens.find(t => t.startIndex <= match.index && t.endIndex > match.index);
      assert.ok(token?.scopes.some(s => s.startsWith('entity.name.type')), `${line}: ${match[0]} must be a type: ${JSON.stringify(tokens)}`);
    }
  }
  for (const line of ['const Player = value; console.log(Player);', 'const obj = { lead: Player, scores: Scores };', 'const choice = ready ? Player : Scores;']) {
    assert.ok(grammar.tokenizeLine(line).tokens.every(t => !t.scopes.some(s => s.startsWith('entity.name.type'))), `Value identifiers must stay values: ${line}`);
  }
  let typeState;
  for (const line of ['interface Team {', '  members: Map<string,', '    Box<Player[]>>;', '}', 'function describe(', '  player: Player', '): Scores {', '  const Player = 1;', '  return Player;', '}']) {
    const result = grammar.tokenizeLine(line, typeState);
    typeState = result.ruleStack;
    for (const match of line.matchAll(/\b(?:Team|Box|Player|Scores)\b/g)) {
      const token = result.tokens.find(t => t.startIndex <= match.index && t.endIndex > match.index);
      const isType = token?.scopes.some(s => s.startsWith('entity.name.type'));
      assert.equal(isType, !line.includes('const') && !line.includes('return'), `Multiline context: ${line}`);
    }
  }
  assert.ok(grammar.tokenizeLine('console.log(Player);', typeState).tokens.every(t => !t.scopes.some(s => s.startsWith('meta.type') || s.startsWith('entity.name.type'))));

  const scopeAt = (result, line, text) => {
    const offset = line.indexOf(text);
    assert.notEqual(offset, -1, `Missing witness ${text}`);
    return result.tokens.find(t => t.startIndex <= offset && t.endIndex > offset).scopes;
  };
  for (const line of [
    'User(user) => `Welcome, ${user.name}`,',
    'Guest(guest) => `Invitation: ${guest.invitation}`,',
  ]) {
    const result = grammar.tokenizeLine(line);
    assert.ok(scopeAt(result, line, '`').includes('string.template.tsrust'));
    assert.ok(scopeAt(result, line, '${').includes('punctuation.section.embedded.begin.tsrust'));
    assert.ok(scopeAt(result, line, '.').includes('meta.embedded.line.tsrust'));
    assert.ok(scopeAt(result, line, '}').includes('punctuation.section.embedded.end.tsrust'));
    assert.ok(!scopeAt(result, line, '`,').includes('meta.embedded.line.tsrust'));
    const label = line.includes('Welcome') ? 'Welcome' : 'Invitation';
    assert.deepEqual(scopeAt(result, line, label), ['source.tsrust', 'string.template.tsrust']);
    assert.ok(!result.tokens.at(-1).scopes.includes('string.template.tsrust'));
  }
  // Braces in records, blocks, strings and comments must not end interpolation.
  const nested = 'const text = `outer ${format({value: `inner ${1 + 2}`}, "}", /* } */ 3)} tail`; const after = 4;';
  const nestedResult = grammar.tokenizeLine(nested);
  assert.ok(scopeAt(nestedResult, nested, '1 +').includes('constant.numeric.tsrust'));
  assert.ok(scopeAt(nestedResult, nested, '+').includes('keyword.operator.tsrust'));
  assert.ok(scopeAt(nestedResult, nested, '3)').includes('meta.embedded.line.tsrust'));
  assert.deepEqual(scopeAt(nestedResult, nested, ' tail'), ['source.tsrust', 'string.template.tsrust']);
  assert.ok(!scopeAt(nestedResult, nested, 'after').includes('string.template.tsrust'));
  const escaped = 'const text = `escaped \\` and \\${literal} and \\\\${2} done`;';
  const escapedResult = grammar.tokenizeLine(escaped);
  assert.ok(scopeAt(escapedResult, escaped, '\\`').includes('constant.character.escape.tsrust'));
  assert.ok(!scopeAt(escapedResult, escaped, 'literal').includes('meta.embedded.line.tsrust'));
  assert.ok(scopeAt(escapedResult, escaped, '2}').includes('constant.numeric.tsrust'));
  let templateState;
  for (const [line, text, scope] of [
    ['const text = `first', 'first', 'string.template.tsrust'],
    ['${(() => {', '=>', 'keyword.operator.tsrust'],
    ['return "}";', 'return', 'keyword.control.tsrust'],
    ['})()} last', 'last', 'string.template.tsrust'],
    ['end`; const after = 5;', '5', 'constant.numeric.tsrust'],
  ]) {
    const result = grammar.tokenizeLine(line, templateState);
    templateState = result.ruleStack;
    assert.ok(scopeAt(result, line, text).includes(scope), line);
    if (text === 'last') assert.ok(!scopeAt(result, line, text).includes('meta.embedded.line.tsrust'));
    if (text === '5') assert.ok(!scopeAt(result, line, text).includes('string.template.tsrust'));
  }
  let loopState;
  for (const [line, word, scope] of [
    ["for (const pair of header.split(';')) {", "';'", 'string.quoted.single.tsrust'],
    ["  const parts = pair.trim().split('=');", 'const', 'storage.type.tsrust'],
    ['  return undefined;', 'return', 'keyword.control.tsrust'],
    ['}', '}', 'source.tsrust'],
    ['export function setSessionCookie(token: string): string {', 'export', 'keyword.control.export.tsrust'],
    ["  return '__Host-session=' + token;", 'return', 'keyword.control.tsrust'],
  ]) {
    const result = grammar.tokenizeLine(line, loopState);
    loopState = result.ruleStack;
    const offset = line.indexOf(word);
    const scopes = result.tokens.find(token => token.startIndex <= offset && token.endIndex > offset)?.scopes ?? [];
    assert.ok(scopes.includes(scope), `${line}: ${scopes}`);
    if (word !== "';'") assert.ok(!scopes.includes('string.quoted.single.tsrust'), line);
  }
  registry.dispose();
});
