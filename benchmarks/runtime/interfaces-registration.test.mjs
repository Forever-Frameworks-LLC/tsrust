import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertInterfacesSource} from './interfaces-source-contract.mjs';
import {expectedInterfacesValues,expectedInterfacesObservation} from './interfaces-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='interfaces');
test('Interfaces retain paired source forms, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertInterfacesSource(source,row,kind);
  assert.throws(()=>assertInterfacesSource(source+' ',row,kind));
  assert.throws(()=>assertInterfacesSource(source,{...row,operationsPerIteration:2},kind));
  for(const changed of [replaceCodeSnippet(source, 'interface Player extends Named, Positioned { score: int, }','interface Player { name: string; x: int; y?: int; score: int, }'),replaceCodeSnippet(source, 'interface NamedValue<T> { name: T; }','type Named = { name: string; };'),replaceCodeSnippet(source, 'return output;','return [];')]){
   const key=kind==='native'?'sourceSha256':'counterpartSha256';
   assert.throws(()=>assertInterfacesSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 // Written by hand for seed 3: k is 3 then 4.
 assert.deepEqual(expectedInterfacesValues(row.id,2,3,'in'),[['in:3:3','p:3:1','3','4','0'],['in:4:4','p:5:1','4','5','1']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'in'),expectedInterfacesObservation(row.id,2,3,'in'));
 assert.equal(expectedChecksum(row.id,2,3,'in'),expectedInterfacesObservation(row.id,2,3,'in').checksum);
 // The direct forms agree with platform objects on every registered case.
 const describe=player=>player.name+':'+String(player.x+(player.y??0))+':'+String(player.score);
 for(const input of ['dynamic é🙂 text','abc','']){
  const all=expectedInterfacesValues(row.id,2048,7,input);
  for(let i=0;i<2048;i++){const k=i+7;const plain={name:String(input),x:k,score:k%5};const placed={name:'p',x:k,y:k%3,score:1};const named={name:String(k)};
   assert.deepEqual(all[i],[describe(plain),describe(placed),String(named.name),String(plain.score+placed.score),String(placed.y??-1)],input+':'+i);}
 }
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertInterfacesSource(source'));assert.ok(source.includes("'interfaces-oracle.mjs'"));}
});
