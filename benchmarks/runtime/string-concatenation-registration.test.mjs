import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertStringConcatenationSource} from './string-concatenation-source-contract.mjs';
import {expectedStringConcatenationValues,expectedStringConcatenationObservation} from './string-concatenation-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='string_concatenation');
test('Scalar concatenation retains sources, independent results and execution hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'strings');
 for(const [kind,path] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+path);assertStringConcatenationSource(source,row,kind);
  assert.throws(()=>assertStringConcatenationSource(source+' ',row,kind));
  assert.throws(()=>assertStringConcatenationSource(source,{...row,operationsPerIteration:2},kind));
  const changed=source.replace('text+=k;text+=flag;','text+=k;');
  const key=kind==='native'?'sourceSha256':'counterpartSha256';
  assert.throws(()=>assertStringConcatenationSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
 }
 assert.deepEqual(expectedStringConcatenationValues(row.id,2,0,'é'),[['é0','0é','fraction:0','flag:true','absent:nullundefined','union:0','é0true'],['é1','1é','fraction:0.5','flag:false','absent:nullundefined','union:undefined','é1false']]);
 for(const input of ['', 'ascii', 'é🙂'])for(const n of [0,1,37,2048]){
  const expected=expectedStringConcatenationObservation(row.id,n,7,input);
  assert.deepEqual(expectedExactResult(row.id,n,7,input),expected);assert.equal(expectedChecksum(row.id,n,7,input),expected.checksum);
 }
 for(const path of ['./run.mjs','./profile-native.mjs']){
  const source=read(path);assert.ok(source.includes('assertStringConcatenationSource(source'));assert.ok(source.includes("'string-concatenation-oracle.mjs'"));
 }
});
