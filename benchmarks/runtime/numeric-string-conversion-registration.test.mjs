import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertNumericStringConversionSource} from './numeric-string-conversion-source-contract.mjs';
import {expectedNumericStringConversionValues,expectedNumericStringConversionObservation} from './numeric-string-conversion-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {assertABI} from './adapters.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='numeric_string_conversion');
test('Numeric text retains sources, independent results and execution hooks',()=>{
 const emitted='fn function0(value0: i64, value1: i64, value2: &String) -> Vec<Array<String>> { todo!() }\nfn main() {}';
 assert.equal(assertABI(emitted,row,read('./'+row.source)).fallible,false);
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'numbers');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertNumericStringConversionSource(source,row,kind);
  assert.throws(()=>assertNumericStringConversionSource(source+' ',row,kind));
  assert.throws(()=>assertNumericStringConversionSource(source,{...row,operationsPerIteration:2},kind));
  const changed=replaceCodeSnippet(source, 'String(+text)','String(text)');
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  assert.throws(()=>assertNumericStringConversionSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
 }
 assert.deepEqual(expectedNumericStringConversionValues(row.id,2,0,'0x10'),[['16','16','16','false'],['5','5','0','false']]);
 for(const input of ['0x10'])for(const n of [0,1,37,2048]){
  const expected=expectedNumericStringConversionObservation(row.id,n,7,input);
  assert.deepEqual(expectedExactResult(row.id,n,7,input),expected);assert.equal(expectedChecksum(row.id,n,7,input),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertNumericStringConversionSource(source'));assert.ok(source.includes("'numeric-string-conversion-oracle.mjs'"));
 }
});
