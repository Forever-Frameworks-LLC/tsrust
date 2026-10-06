import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertStringIndexingSource} from './string-indexing-source-contract.mjs';
import {expectedStringIndexingValues,expectedStringIndexingObservation} from './string-indexing-oracle.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='string_indexing');
test('String indexing retains paired source forms, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'strings');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
  const source=read('./'+name);assertStringIndexingSource(source,row,kind);
  assert.throws(()=>assertStringIndexingSource(source+' ',row,kind));
  assert.throws(()=>assertStringIndexingSource(source,{...row,operationsPerIteration:2},kind));
  for(const changed of [source.replace('}catch(error){later="out";}','}catch(error){later="in";}'),source.replace('return output;','return [];')]){
   const key=kind==='native'?'sourceSha256':'counterpartSha256';
   assert.throws(()=>assertStringIndexingSource(changed,{...row,[key]:createHash('sha256').update(changed).digest('hex')},kind));
  }
 }
 // Written by hand for seed 3: positions 3 and 4, then position 5 whose later position is out of range.
 assert.deepEqual(expectedStringIndexingValues(row.id,3,3,'tail'),[['d','dg','i','3','tail'],['e','ef','j','4','tail'],['f','fe','out','5','tail']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedStringIndexingObservation(row.id,2,3,'tail'));
 assert.equal(expectedChecksum(row.id,2,3,'tail'),expectedStringIndexingObservation(row.id,2,3,'tail').checksum);
 // The direct forms agree with platform bracket access plus an explicit range check.
 const at=(text,position)=>{const found=text[position];if(found===undefined)throw new RangeError('range');return found;};
 for(const input of ['dynamic é🙂 text','abc','']){
  const all=expectedStringIndexingValues(row.id,2048,7,input);
  for(let i=0;i<2048;i++){const position=(i+7)%10;let later='';try{later=at('abcdefghij',position+5);}catch{later='out';}
   assert.deepEqual(all[i],[at('abcdefghij',position),at('abcdefghij',position)+at('abcdefghij',9-position),later,String(position),input],input+':'+i);}
 }
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertStringIndexingSource(source'));assert.ok(source.includes("'string-indexing-oracle.mjs'"));}
});
