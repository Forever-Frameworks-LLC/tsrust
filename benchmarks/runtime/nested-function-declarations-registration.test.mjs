import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNestedFunctionDeclarationsSource} from './nested-function-declarations-source-contract.mjs';
import {expectedNestedFunctionDeclarationValues,expectedNestedFunctionDeclarationObservation} from './nested-function-declarations-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='nested_function_declarations');
test('Nested declarations retain sources, independent observations and execution hooks',()=>{
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Array<NumericArray<i64>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'recursion');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertNestedFunctionDeclarationsSource(source,row,kind);
  assert.throws(()=>assertNestedFunctionDeclarationsSource(source+' ',row,kind));
  assert.throws(()=>assertNestedFunctionDeclarationsSource(source,{...row,operationsPerIteration:2},kind));
  const changed=replaceCodeSnippet(source, 'factorial(5)','factorial(4)');
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  assert.throws(()=>assertNestedFunctionDeclarationsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  assert.ok(source.includes('let value = first + 3;'), 'Variable shadow scenario must be retained');
  const missingShadow=replaceCodeSnippet(source, 'let value = first + 3;', 'let other = first + 3;');
  assert.throws(()=>assertNestedFunctionDeclarationsSource(missingShadow,{...row,[key]:createHash('sha256').update(missingShadow).digest('hex')},kind));
 }
 assert.deepEqual(expectedNestedFunctionDeclarationValues(row.id,2,0,'nested'),[[2,120,0,3,2,4,7],[4,120,1,6,4,6,10]]);
 for(const seed of [0,7,1000000])for(const n of [0,1,37,2048]){
  const expected=expectedNestedFunctionDeclarationObservation(row.id,n,seed,'nested');
  assert.deepEqual(expectedExactResult(row.id,n,seed,'nested'),expected);assert.equal(expectedChecksum(row.id,n,seed,'nested'),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertNestedFunctionDeclarationsSource(source'));assert.ok(source.includes("'nested-function-declarations-oracle.mjs'"));
 }
});
