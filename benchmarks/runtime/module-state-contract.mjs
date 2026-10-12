import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';

export function moduleStateChecksum(n, seed) {
  if (!Number.isSafeInteger(n) || n < 0 || !Number.isSafeInteger(seed) || !Number.isSafeInteger(seed + 3 * n)) throw Error('Module state oracle domain');
  return seed + 3 * n;
}

export function assertModuleStateSource(source, row, kind) {
  if (row.id !== 'native_module_state' || row.family !== 'module-state' || row.result !== 'int' || row.operationsPerIteration !== 1) throw Error('Module state identity');
  const key = kind === 'native' ? 'sourceSha256' : kind === 'typescript' ? 'counterpartSha256' : undefined;
  if (!key || createHash('sha256').update(source).digest('hex') !== row[key]) throw Error('Module state fingerprint');
  for (const snippet of ['let completed = 0;', 'function increment() { completed++; }', 'const before = completed;', 'while (i < n)', 'increment();', 'checksum += 3;', 'i++;', 'if (completed - before !== n)', "throw 'Module state was not retained';", 'return checksum;']) {
    if (!containsCodeSnippet(source, snippet)) throw Error('Module state operation: ' + snippet);
  }
  if ((source.match(/completed\s*=/g) ?? []).length !== 1) throw Error('Module state may only initialize once');
}

export function moduleStateABI(emitted, row, source) {
  assertModuleStateSource(source, row, 'native');
  const parameters = 'value0: i64, value1: i64, value2: &mut i64';
  const matches = [...emitted.matchAll(/fn (tsr_result_function\d+)\(([^\n)]*)\) -> (Result<i64,TsrFunctionError\d+<[^\n]+>>) \{/g)].filter(match => match[2] === parameters);
  if (matches.length !== 1 || !/let (?:mut )?value0: i64 = 0i64;/.test(emitted)) throw Error('Module state owner ABI changed');
  return {functionName: matches[0][1], parameters, result: 'i64', fallible: true};
}

export function moduleStateDriver(driver, row, emitted, source) {
  const {functionName} = moduleStateABI(emitted, row, source);
  if (!driver.includes('fn main() {') || !/\bfunction0\(/.test(driver)) throw Error('Module state driver hooks changed');
  // This scalar-only workload initializes one zero counter. Initialization is
  // outside measurement, as in the TypeScript module. Workload bodies are intact.
  const setup = `fn main() {\n    let mut module_completed = 0i64;\n    let mut benchmark_module_workload = |n: i64, seed: i64| -> i64 {\n        match ${functionName}(n, seed, &mut module_completed) {\n            Ok(value) => value,\n            Err(_) => panic!("module state benchmark failed"),\n        }\n    };`;
  return driver.replace('fn main() {', setup).replace(/\bfunction0(?=\()/g, 'benchmark_module_workload');
}
