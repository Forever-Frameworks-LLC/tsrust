import assert from 'node:assert/strict';
import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export const regexTextSpecs = {
  regex_compile: {input: "([a-z]+)=([0-9]+)", weight: 1, operations: 1, native: "const expression = unwrap(compile(input));", typescript: "const expression = new RegExp(input, 'du');"},
  regex_escape: {input: "a+b[é].*", weight: 14, operations: 1, native: "const value = escape(input);", typescript: "const value = input.replace(/[.*+?^${}()|[\\]\\\\]/g, '\\\\$&');"},
  regex_is_match: {input: "a=12 b=3", weight: 1, operations: 1, native: "checksum += expression.isMatch(input) ? 1 : 0;", typescript: "checksum += expression.test(input) ? 1 : 0;"},
  regex_find: {input: "a=12 b=3", weight: 5, operations: 1, native: "const value = expression.find(input);", typescript: "const value = expression.exec(input);"},
  regex_find_all: {input: "a=12 b=3", weight: 19, operations: 1, native: "const values = expression.findAll(input);", typescript: "const values = Array.from(input.matchAll(expression), value => ({start: value.index, end: value.index + value[0].length}));"},
  regex_captures: {input: "a=12 b=3", weight: 14, operations: 1, native: "const value = expression.captures(input);", typescript: "const found = expression.exec(input);"},
  regex_captures_all: {input: "a=12 b=3", weight: 56, operations: 1, native: "const values = expression.capturesAll(input);", typescript: "const values = Array.from(input.matchAll(expression), captureSnapshot);"},
  regex_replace: {input: "a=12 b=3", weight: 10, operations: 1, native: "const value = expression.replace(input, '$2:$1 $$');", typescript: "const value = input.replace(expression, '$2:$1 $$');"},
  regex_replace_all: {input: "a=12 b=3", weight: 12, operations: 1, native: "const value = expression.replaceAll(input, '$2:$1 $$');", typescript: "const value = input.replace(expression, '$2:$1 $$');"},
  regex_split: {input: "a=12 b=3", weight: 28, operations: 1, native: "const values = expression.split(input);", typescript: "const values = splitRanges(expression, input);"},
  string_regex_match: {input: "a=12 b=3", weight: 14, operations: 1, native: "const value = input.match(expression);", typescript: "const found = expression.exec(input);"},
  string_regex_match_all: {input: "a=12 b=3", weight: 56, operations: 1, native: "const values = input.matchAll(expression);", typescript: "const values = Array.from(input.matchAll(expression), captureSnapshot);"},
  string_regex_search: {input: "a=12 b=3", weight: 1, operations: 1, native: "checksum += input.search(expression) + 1;", typescript: "checksum += input.search(expression) + 1;"},
};
export function regexTextChecksum(id, n, seed, input) {
  const spec = regexTextSpecs[id];
  assert(spec, 'Unknown regex/text workload');
  assert.equal(input, spec.input);
  assert(Number.isSafeInteger(n) && n >= 0 && Number.isSafeInteger(seed));
  const value = seed + n * spec.weight;
  assert(Number.isSafeInteger(value));
  return value;
}
export function assertRegexTextSource(source, row, kind) {
  const spec = regexTextSpecs[row.id];
  assert(spec && ['native', 'typescript'].includes(kind));
  assert.equal(row.family, 'regex-text');
  assert.equal(createHash('sha256').update(source).digest('hex'), row[kind === 'native' ? 'sourceSha256' : 'counterpartSha256']);
  assert.equal(row.inputText, spec.input);
  assert.equal(row.operationsPerIteration, 1);
  assert.deepEqual(row.parameters, ['int', 'int', 'string']);
  assert.equal(row.result, 'int');
  for (const snippet of ['let checksum = seed;', 'while (i < n)', 'i++;', 'return checksum;']) assert(containsCodeSnippet(source, snippet), row.id + ': ' + snippet);
  assert(containsCodeSnippet(source, spec[kind]), row.id);
  if (!['regex_compile', 'regex_escape'].includes(row.id) && kind === 'typescript') assert(containsCodeSnippet(source, 'expression.lastIndex = 0;'), row.id);
}
