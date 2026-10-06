import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {expectedTestingAssertionChecksum} from './testing-assertions-oracle.mjs';
import {assertTestingAssertionSource} from './testing-assertions-source-contract.mjs';
import {assertABI,adaptRustDriver} from './adapters.mjs';
test('assertion observer preserves independent hand cells',()=>{
 assert.equal(expectedTestingAssertionChecksum(0,0),0);
 assert.equal(expectedTestingAssertionChecksum(1,3),9);
 assert.equal(expectedTestingAssertionChecksum(2,3),1179);
 assert.throws(()=>expectedTestingAssertionChecksum(-1,3));
});
test('assertion workload uses real APIs and exact current sources',()=>{
 const [row]=JSON.parse(fs.readFileSync(new URL('testing-assertions-manifest-entries.json',import.meta.url)));
 for(const [engine,extension] of [['native','tsr'],['typescript','ts']]){
  const source=fs.readFileSync(new URL('workloads/'+row.id+'.'+extension,import.meta.url),'utf8');
  assert.doesNotThrow(()=>assertTestingAssertionSource(source,row,engine));
  assert.throws(()=>assertTestingAssertionSource(source.replace('strictEqual(actual, expected);',''),row,engine));
 }
 const emitted='fn function12(value0: i64, value1: i64) -> i64 {0}\nfn main() {}';
 assert.equal(assertABI(emitted,row).functionName,'function12');
 assert.match(adaptRustDriver('fn benchmark_checksum(value: i64) -> i64 { value }\nfn main(){function0(1,2);}',row,{emitted}),/function12\(1,2\)/);
});
