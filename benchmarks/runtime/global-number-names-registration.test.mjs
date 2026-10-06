import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertGlobalNumberNamesSource} from './global-number-names-source-contract.mjs';
import {globalNumberNamesIds,expectedGlobalNumberNamesValues} from './global-number-names-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='global-number-names');
test('global number name registration binds complete scenarios and preserves prior manifest rows',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),globalNumberNamesIds);
 for(const row of rows){
  assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'other');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertGlobalNumberNamesSource(source,row,kind);
   for(const changed of [source.replace('const missing=parseInt(input);','const missing=Number.parseInt(input);'),source.replace('let lowest=Infinity;','let lowest=1.0;'),source.replace('isNaN(missing)?NaN:','isNaN(missing)?0.0:'),source.replace('return output;','return [];')]){
    const key=kind==='native'?'sourceSha256':'counterpartSha256';
    assert.throws(()=>assertGlobalNumberNamesSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
   }
   assert.throws(()=>assertGlobalNumberNamesSource(source,{...row,operationsPerIteration:5},kind));
   assert.throws(()=>assertGlobalNumberNamesSource(source+' ',row,kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertGlobalNumberNamesSource\(source/);assert.ok(source.includes("'global-number-names-oracle.mjs'"));}
});
test('independent global number name oracle matches hand-written cells, the platform and oracle routing',()=>{
 // Written by hand: seed 0 gives bases 0 and 13; "13" in base sixteen is 19.
 const expected=[['0','0.25','true','true','NaN'],['32','13.25','true','true','NaN']];
 assert.deepEqual(expectedGlobalNumberNamesValues('global_number_names',2,0,'ab'),expected);
 assert.deepEqual(expectedGlobalNumberNamesValues('global_number_names',1,0,'  -42 text'),[['0','0.25','false','true','-41']]);
 const observation=observeSliceTree(expected);
 assert.deepEqual(expectedExactResult('global_number_names',2,0,'ab'),observation);
 assert.equal(expectedChecksum('global_number_names',2,0,'ab'),observation.checksum);
 assert.deepEqual(expectedGlobalNumberNamesValues('global_number_names',0,0,''),[]);
 // The direct forms agree with the platform's global names on every registered case.
 for(const input of ['dynamic é🙂 text','abc','','42 text','  -17','+8.9']){
  const all=expectedGlobalNumberNamesValues('global_number_names',2048,7,input);
  for(let i=0;i<2048;i++){
   const base=(i*13+7)%97;const whole=parseInt(String(base)+'px'),hex=parseInt(String(base),16),part=parseFloat(String(base)+'.25x'),missing=parseInt(input);
   let lowest=Infinity;if(part<lowest)lowest=part;
   const picked=isNaN(missing)?NaN:missing+1;
   assert.deepEqual(all[i],[String(whole+hex),String(part),String(isNaN(missing)),String(isFinite(lowest)&&!isFinite(-Infinity)),String(picked)],input+':'+i);}
 }
});
