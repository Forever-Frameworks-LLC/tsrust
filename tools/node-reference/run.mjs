// Runs a Tsrust fixture in Node to produce reference output.
// Usage: node tools/node-reference/run.mjs <fixture.tsr>
// Type annotations are removed by the TypeScript transpiler. clone() is a deep
// copy, the ownership wrapper types are identities and std:testing imports
// resolve to node:assert/strict; nothing else is adapted.
import fs from 'node:fs';
import {createRequire} from 'node:module';
const ts = createRequire(new URL('../../benchmarks/runtime/package.json', import.meta.url))('typescript');
// std:testing assertions run as Node's strict assertions of the same names.
const source = fs.readFileSync(process.argv[2], 'utf8').replaceAll('"std:testing"', '"node:assert/strict"');
const prelude = 'type int = number; type ReadOnly<T> = T; type Mutable<T> = T; type Owned<T> = T;\nconst clone = <T,>(value: T): T => structuredClone(value);\n';
const output = ts.transpileModule(prelude + source + '\nmain();\n', {compilerOptions: {target: ts.ScriptTarget.ES2023, module: ts.ModuleKind.ESNext}}).outputText;
await import('data:text/javascript;base64,' + Buffer.from(output).toString('base64'));
