import fs from 'node:fs';
import vm from 'node:vm';
import assert from 'node:assert/strict';
import util from 'node:util';
import {createRequire} from 'node:module';
const require=createRequire(new URL('../../benchmarks/runtime/package.json',import.meta.url));
const ts=require('typescript');
const source=fs.readFileSync(new URL('../../crates/tsrust-compiler/tests/tuple_types.rs',import.meta.url),'utf8');
const matches=[...source.matchAll(/execute\(\s*r#"([\s\S]*?)"#\s*,\s*("(?:[^"\\]|\\.)*")\s*,\s*None/g)];
assert.equal(matches.length,6);
for(const [index,match] of matches.entries()){
 let native=match[1].replace(/(?:ReadOnly|Mutable|Owned)<([^<>]+)>/g,'$1');
 const code=ts.transpileModule('type int=number;\n'+native,{compilerOptions:{target:ts.ScriptTarget.ES2022,module:ts.ModuleKind.None}}).outputText;
 const lines=[];
 vm.runInNewContext(code+'\nmain();',{clone:value=>structuredClone(value),console:{log:(...values)=>lines.push(util.format(...values))}});
 const actual=lines.join('\n')+'\n';
 assert.equal(actual,JSON.parse(match[2]),`native fixture ${index+1}`);
}
console.log(`${matches.length} compiler fixtures match Node ${process.version}`);
