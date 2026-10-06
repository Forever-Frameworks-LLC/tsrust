import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertTypeAliasesSource} from './type-aliases-source-contract.mjs';
import {expectedTypeAliasesValues,expectedTypeAliasesObservation} from './type-aliases-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='type_aliases');
test('Type aliases retain paired source forms, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertTypeAliasesSource(source,row,kind);
  assert.throws(()=>assertTypeAliasesSource(source+' ',row,kind));
  assert.throws(()=>assertTypeAliasesSource(source,{...row,operationsPerIteration:2},kind));
  for(const changed of [source.replace('const ids:Ids=','const ids:(int|string)[]='),source.replace('type Pair = [Count, string];','type Pair = [int, string];'),source.replace('return output;','return [];')]){
   const key=kind==='native'?'sourceSha256':'counterpartSha256';
   assert.throws(()=>assertTypeAliasesSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 // Written by hand for seed 3.
 assert.deepEqual(expectedTypeAliasesValues(row.id,2,3,'tail'),[['number','text','6','3','2'],['number','text','8','4','2']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedTypeAliasesObservation(row.id,2,3,'tail'));
 assert.equal(expectedChecksum(row.id,2,3,'tail'),expectedTypeAliasesObservation(row.id,2,3,'tail').checksum);
 // The direct forms agree with the platform on every registered case.
 const kind=id=>typeof id==='string'?'text':'number';
 for(const input of ['dynamic é🙂 text','abc','']){
  const all=expectedTypeAliasesValues(row.id,2048,7,input);
  for(let i=0;i<2048;i++){const k=i+7;const ids=[k,String(input)];const pair=[k*2,String(k)];const [doubled,label]=pair;
   assert.deepEqual(all[i],[kind(ids[0]),kind(ids[1]),String(doubled),String(label),String(ids.length)],input+':'+i);}
 }
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertTypeAliasesSource(source'));assert.ok(source.includes("'type-aliases-oracle.mjs'"));}
});
