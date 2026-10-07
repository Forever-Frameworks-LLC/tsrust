import {replaceCodeSnippet} from './source-contract-tokens.mjs';
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertBracelessBodiesSource} from './braceless-bodies-source-contract.mjs';
import {bracelessBodiesIds,expectedBracelessBodiesValues} from './braceless-bodies-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='braceless-bodies');
test('single-statement body registration binds complete scenarios and preserves prior manifest rows',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),bracelessBodiesIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertBracelessBodiesSource(source,row,kind);
   for(const changed of [replaceCodeSnippet(source, 'for(let k=0;k<base%5+1;k++)sum+=k;','for(let k=0;k<base%5+1;k++){sum+=k;}'),replaceCodeSnippet(source, 'else if(a+b===2)odd+=10;',''),replaceCodeSnippet(source, 'return output;','return [];')]){
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertBracelessBodiesSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertBracelessBodiesSource(source,{...row,operationsPerIteration:6},kind));
   assert.throws(()=>assertBracelessBodiesSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertBracelessBodiesSource\(source/);assert.ok(source.includes("'braceless-bodies-oracle.mjs'"));}
});
test('independent single-statement body oracle matches hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0, 13, 26, 39, 52, 65.
 const expected=[['low','0','ab0','23','5'],['low','6','ab13','23','5'],['mid','1','ab26','23','5'],['mid','10','ab39','23','5'],['mid','3','ab52','23','5'],['high','0','ab65','23','5']];
 assert.deepEqual(expectedBracelessBodiesValues('braceless_bodies',6,0,'ab'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('braceless_bodies',6,0,'ab'),observation);
 assert.equal(expectedChecksum('braceless_bodies',6,0,'ab'),observation.checksum);
 assert.deepEqual(expectedBracelessBodiesValues('braceless_bodies',0,0,''),[]);
 // The closed forms agree with the platform's braced loops on every registered case.
 const input='dynamic é🙂 text',all=expectedBracelessBodiesValues('braceless_bodies',2048,7,input);
 for(let i=0;i<2048;i++){
  const base=(i*13+7)%97;let sum=0,odd=0,text='';
  for(let k=0;k<base%5+1;k++){sum+=k;}
  for(const part of [input,String(base)]){text+=part;}
  for(let a=0;a<3;a++){for(let b=0;b<3;b++){if(a===b){odd+=1;}else if(a+b===2){odd+=10;}}}
  let down=base%4+1;while(down>1){down--;}do{down+=2;}while(down<4);
  const grade=base<20?'low':base<60?'mid':'high';
  assert.deepEqual(all[i],[grade,String(sum),text,String(odd),String(down)],String(i));}
});
