import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {expectedTestingExpectationChecksum} from './testing-assertions-oracle.mjs';
import {assertTestingAssertionSource} from './testing-assertions-source-contract.mjs';
import {expectedChecksum} from './oracle.mjs';
const row=JSON.parse(fs.readFileSync(new URL('./testing-assertions-manifest-entries.json',import.meta.url))).find(row=>row.id==='testing_expectation_predicates');
test('expectation oracle has independent hand-checked category cells',()=>{
 assert.equal(expectedTestingExpectationChecksum(0,0),0);
 assert.equal(expectedTestingExpectationChecksum(1,0),1);
 assert.equal(expectedTestingExpectationChecksum(2,0),133);
 assert.equal(expectedTestingExpectationChecksum(1,5),4);
 assert.equal(expectedChecksum(row.id,2,0),133);
});
test('matched expectation sources have exact hashes and real API calls',()=>{
 for(const [engine,extension] of [['native','tsr'],['typescript','ts']]){
  const source=fs.readFileSync(new URL('./workloads/'+row.id+'.'+extension,import.meta.url),'utf8');
  assertTestingAssertionSource(source,row,engine);
  assert.throws(()=>assertTestingAssertionSource(source.replace(row.methodExpressions[engine][0],'standIn()'),row,engine));
 }
});
