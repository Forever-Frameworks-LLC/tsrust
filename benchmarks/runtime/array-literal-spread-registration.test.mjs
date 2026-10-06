import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {createHash} from 'node:crypto';
import {assertArrayLiteralSpreadSource} from './array-literal-spread-source-contract.mjs';
import {arrayLiteralSpreadIds,expectedArrayLiteralSpreadObservation} from './array-literal-spread-oracle.mjs';
import {hands} from './call-spread-values-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const manifest=JSON.parse(read('./manifest.json'));
const rows=manifest.workloads.filter(row=>row.family==='array-literal-spread');
test('Gate101 appends guarded literal spread pairs while retaining every prior manifest object',()=>{
 
 assert.deepEqual(rows.map(row=>row.id),arrayLiteralSpreadIds);
 for(const row of rows){
  assert.equal(row.pendingCheckpoint,101);assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'arrays');
  for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){
   const source=read('./'+name);assertArrayLiteralSpreadSource(source,row,kind);
   assert.throws(()=>assertArrayLiteralSpreadSource(source.replace('...',''),row,kind));
   assert.throws(()=>assertArrayLiteralSpreadSource(source.replace('return output;','return [];'),row,kind));
   assert.throws(()=>assertArrayLiteralSpreadSource(source,{...row,operationsPerIteration:2},kind));
  }
 }
 for(const name of ['./run.mjs','./profile-native.mjs']){const source=read(name);assert.match(source,/assertArrayLiteralSpreadSource\(source/);assert.ok(source.includes("'array-literal-spread-oracle.mjs'"));}
});
test('Gate101 independent literal results retain exact shape and signed zero',()=>{
 const cells=hands.filter(([id])=>arrayLiteralSpreadIds.includes(id.replace('call_spread_','array_literal_spread_'))).map(([id,n,seed,input,values])=>[id.replace('call_spread_','array_literal_spread_'),n,seed,input,values]);
 cells.push(['array_literal_spread_number',1,0,'x',[[1.5,-2.5]]],['array_literal_spread_number',1,2,'x',[[1.5,0,-0,-2.5]]],['array_literal_spread_callable',1,7,'x',[[7,8,9]]],['array_literal_spread_source_order',1,7,'x',[[7,8,3,7,8,9],[7,8,9]]]);
 for(const [id,n,seed,input,values] of cells){const want=observeSliceTree(values,{numbers:id.endsWith('_number')});assert.deepEqual(expectedArrayLiteralSpreadObservation(id,n,seed,input),want);assert.deepEqual(expectedExactResult(id,n,seed,input),want);assert.equal(expectedChecksum(id,n,seed,input),want.checksum);}
 for(const row of rows){const want=expectedArrayLiteralSpreadObservation(row.id,2048,7,row.inputText);assert.deepEqual(expectedExactResult(row.id,2048,7,row.inputText),want);assert.equal(expectedChecksum(row.id,2048,7,row.inputText),want.checksum);}
});
