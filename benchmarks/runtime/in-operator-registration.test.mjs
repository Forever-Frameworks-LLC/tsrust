import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertInOperatorSource} from './in-operator-source-contract.mjs';
import {inOperatorIds,expectedInOperatorValues} from './in-operator-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='in-operator');
test('in registration preserves prior rows and guards complete scenarios',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),inOperatorIds);
 for(const row of rows){assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertInOperatorSource(source,row,kind);
   for(const text of ['const present=wanted in sample;','const positioned=index in values;','const fractional=(Number(index)+0.5) in values;','const length="length" in values;','for(const key in sample){names+=key+",";}','for(const key in values){indices+=key;}','return output;']){
    const changed=replaceCodeSnippet(source, text,'');const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertInOperatorSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertInOperatorSource(source,{...row,operationsPerIteration:8},kind));assert.throws(()=>assertInOperatorSource(source+' ',row,kind));
  }
 }
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.match(source,/assertInOperatorSource\(source/);assert.ok(source.includes("'in-operator-oracle.mjs'"));}
});
test('independent in oracle retains cells and agrees with platform',()=>{
 const expected=[['true','true','false','true','name,amount,','012','ab0','0'],['false','false','false','true','name,amount,','012','ab13','13']];
 assert.deepEqual(expectedInOperatorValues('in_operator',2,0,'ab'),expected);
 const observation=observeSliceTree(expected);assert.deepEqual(expectedExactResult('in_operator',2,0,'ab'),observation);assert.equal(expectedChecksum('in_operator',2,0,'ab'),observation.checksum);
 assert.deepEqual(expectedInOperatorValues('in_operator',0,0,''),[]);
 for(const input of ['dynamic é🙂 text','abc','']){const expected=expectedInOperatorValues('in_operator',2048,7,input);
  for(let i=0;i<2048;i++){const base=(i*13+7)%97,sample={name:input+base,amount:base},values=[base,base+1,base+2],wanted=base%2===0?'name':'missing',index=base%5;
   let names='',indices='';for(const key in sample)names+=key+',';for(const key in values)indices+=key;
   assert.deepEqual(expected[i],[String(wanted in sample),String(index in values),String((index+0.5) in values),String('length' in values),names,indices,sample.name,String(sample.amount)]);
  }
 }
});
