import fs from 'node:fs';
import vm from 'node:vm';
import util from 'node:util';
import assert from 'node:assert/strict';
import {createRequire} from 'node:module';
const require=createRequire(new URL('../../benchmarks/runtime/package.json',import.meta.url));
const ts=require('typescript');
const source=fs.readFileSync(new URL('../../crates/tsrust-compiler/tests/object_keys.rs',import.meta.url),'utf8');
const matches=[...source.matchAll(/execute\(\s*r#"([\s\S]*?)"#\s*,\s*("(?:[^"\\]|\\.)*")\s*,\s*None/g)];
assert.equal(matches.length,4);
for(const [index,match] of matches.entries()){
 const input='type int=number;\n'+match[1].replace(/(?:ReadOnly|Mutable|Owned)<([^<>]+)>/g,'$1');
 const code=ts.transpileModule(input,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText;
 const lines=[];
 // Use Node Object.keys directly; adapt only explicit native clone().
 vm.runInNewContext(code+'\nmain();',{clone:value=>structuredClone(value),console:{log:(...values)=>lines.push(util.format(...values))}});
 assert.equal(lines.join('\n')+'\n',JSON.parse(match[2]),`fixture ${index+1}`);
}
console.log(`${matches.length} fixtures match Node ${process.version} with unadapted Object.keys`);
