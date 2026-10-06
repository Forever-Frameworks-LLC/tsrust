import {assertABI,adaptRustDriver} from './adapters.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertArrayCallbackErrorsSource} from './array-callback-errors-source-contract.mjs';
import {expectedArrayCallbackErrorsValues,expectedArrayCallbackErrorsObservation} from './array-callback-errors-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='array_callback_errors');
test('checked callbacks retain matched sources, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertArrayCallbackErrorsSource(source,row,kind);
  assert.throws(()=>assertArrayCallbackErrorsSource(source+' ',row,kind));
  assert.throws(()=>assertArrayCallbackErrorsSource(source,{...row,operationsPerIteration:2},kind));
 }
 assert.deepEqual(expectedArrayCallbackErrorsValues(row.id,2,3,'tail'),[['tail','1','5,3,4','3,4,5'],['tail','1','6,4,5','4,5,6']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedArrayCallbackErrorsObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertArrayCallbackErrorsSource(source'));assert.ok(source.includes("'array-callback-errors-oracle.mjs'"));}
});

test('checked callback workload Result ABI and driver stay explicit',()=>{
 const emitted='fn tsr_result_function0(value0: i64, value1: i64, value2: &String) -> Result<Vec<Array<String>>, TsrFunctionError0> { unimplemented!() }\nfn main() {}';
 const binding=assertABI(emitted,row,{source:read('./'+row.source)});assert.equal(binding.fallible,true);
 const driver=adaptRustDriver('fn main(){function0(1,2,&String::Borrowed("x"));}',row,{emitted,source:read('./'+row.source)});
 assert.ok(driver.includes('fn benchmark_checked_workload('));assert.ok(driver.includes('match tsr_result_function0('));
 assert.throws(()=>assertABI(emitted,{...row,family:'unknown'},{source:read('./'+row.source)}));
});
