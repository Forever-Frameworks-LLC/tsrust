import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertScalarUnaryPlusSource} from './scalar-unary-plus-source-contract.mjs';
import {expectedScalarUnaryPlusValues,expectedScalarUnaryPlusObservation} from './scalar-unary-plus-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='scalar_unary_plus');
test('Scalar unary plus retains sources, independent results and execution hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'numbers');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertScalarUnaryPlusSource(source,row,kind);
  assert.throws(()=>assertScalarUnaryPlusSource(source+' ',row,kind));
  assert.throws(()=>assertScalarUnaryPlusSource(source,{...row,operationsPerIteration:2},kind));
  const changed=source.replace('String(+flag)','String(flag)');
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  assert.throws(()=>assertScalarUnaryPlusSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
 }
 assert.deepEqual(expectedScalarUnaryPlusValues(row.id,2,0,'invalid'),[['0.5','1','0','NaN','0.5','0.5','true'],['1.5','0','0','NaN','NaN','NaN','true']]);
 for(const input of ['invalid'])for(const n of [0,1,37,2048]){
  const expected=expectedScalarUnaryPlusObservation(row.id,n,7,input);
  assert.deepEqual(expectedExactResult(row.id,n,7,input),expected);assert.equal(expectedChecksum(row.id,n,7,input),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertScalarUnaryPlusSource(source'));assert.ok(source.includes("'scalar-unary-plus-oracle.mjs'"));
 }
});
