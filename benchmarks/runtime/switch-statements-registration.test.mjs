import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertSwitchStatementsSource} from './switch-statements-source-contract.mjs';
import {switchStatementsIds,expectedSwitchStatementsValues} from './switch-statements-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='switch-statements');
test('switch registration binds complete scenarios and preserves prior manifest rows',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),switchStatementsIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertSwitchStatementsSource(source,row,kind);
   for(const changed of [source.replace('case 0:continue;','case 0:break;'),source.replace('default:steps+=1;','default:steps+=2;'),source.replace('return output;','return [];')]){
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertSwitchStatementsSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertSwitchStatementsSource(source,{...row,operationsPerIteration:4},kind));
   assert.throws(()=>assertSwitchStatementsSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertSwitchStatementsSource\(source/);assert.ok(source.includes("'switch-statements-oracle.mjs'"));}
});
test('independent switch oracle matches hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0, 13, 26, 39.
 const expected=[['zero','422','short','THREE'],['other','422','number','thirteen'],['five','422','short','none'],['other','422','number','THREE']];
 assert.deepEqual(expectedSwitchStatementsValues('switch_statements',4,0,'abc'),expected);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('switch_statements',4,0,'abc'),observation);
 assert.equal(expectedChecksum('switch_statements',4,0,'abc'),observation.checksum);
 assert.deepEqual(expectedSwitchStatementsValues('switch_statements',0,0,''),[]);
 // The tables agree with the platform's switch statements on every registered case and input.
 for(const input of ['dynamic é🙂 text','abc','other']){
  const all=expectedSwitchStatementsValues('switch_statements',2048,7,input);
  for(let i=0;i<2048;i++){
   const base=(i*13+7)%97;let tier;switch(base%7){case 0:tier='zero';break;case 1:case 2:tier='low';break;case 5:tier='five';break;default:tier='other';}
   let steps=0;for(let k=0;k<6;k++){switch((base+k)%3){case 0:continue;case 1:steps+=10;break;default:steps+=1;}steps+=100;}
   const word=base%2===0?input:String(base);let text;switch(word){default:text='number';break;case 'abc':text='short';break;case 'dynamic é🙂 text':text='long';break;}
   const mixed=base%3===0?'three':base;let seen;switch(mixed){case 'three':seen=mixed.toUpperCase();break;case 13:seen='thirteen';break;default:seen='none';}
   assert.deepEqual(all[i],[tier,String(steps),text,seen],input+':'+i);}
 }
});
