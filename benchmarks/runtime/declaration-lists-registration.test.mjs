import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertDeclarationListsSource} from './declaration-lists-source-contract.mjs';
import {declarationListsIds,expectedDeclarationListsValues} from './declaration-lists-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='declaration-lists');
test('declaration list registration binds complete scenarios and preserves prior manifest rows',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),declarationListsIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertDeclarationListsSource(source,row,kind);
   for(const changed of [replaceCodeSnippet(source, 'a++,b--','a++'),replaceCodeSnippet(source, 'let sum=0,steps=0,text="";','let sum=0;let steps=0;let text="";'),replaceCodeSnippet(source, 'return output;','return [];')]){
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertDeclarationListsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertDeclarationListsSource(source,{...row,operationsPerIteration:2},kind));
   assert.throws(()=>assertDeclarationListsSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertDeclarationListsSource\(source/);assert.ok(source.includes("'declaration-lists-oracle.mjs'"));}
});
test('independent declaration list oracle matches hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0 and 13, so low is 0 and 3.
 const expected=[['12','3','abab','0','6'],['12','3','abab','13','12']];
 assert.deepEqual(expectedDeclarationListsValues('declaration_lists',2,0,'ab'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('declaration_lists',2,0,'ab'),observation);
 assert.equal(expectedChecksum('declaration_lists',2,0,'ab'),observation.checksum);
 assert.deepEqual(expectedDeclarationListsValues('declaration_lists',0,0,''),[]);
 // The closed forms agree with the platform's declarations and loops on every registered case.
 const input='dynamic é🙂 text',all=expectedDeclarationListsValues('declaration_lists',2048,7,input);
 for(let i=0;i<2048;i++){
  const base=(i*13+7)%97,low=base%5,high=low+6,words=[input,String(base)],[first,second]=words;
  let sum=0,steps=0,text='';
  for(let a=low,b=high;a<b;a++,b--){sum+=b-a;steps++;}
  for(let k=0,limit=2;k<limit;k++,text+=first){if(k===0)continue;}
  assert.deepEqual(all[i],[String(sum),String(steps),text,second,String(low+high)],String(i));}
});
