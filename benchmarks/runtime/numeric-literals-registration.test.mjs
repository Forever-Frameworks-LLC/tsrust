import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertNumericLiteralsSource} from './numeric-literals-source-contract.mjs';
import {expectedNumericLiteralsValues,expectedNumericLiteralsObservation} from './numeric-literals-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='numeric_literals');
test('Numeric literals retain paired source forms, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'numbers');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){const source=read('./'+name);assertNumericLiteralsSource(source,row,kind);assert.throws(()=>assertNumericLiteralsSource(source+' ',row,kind));assert.throws(()=>assertNumericLiteralsSource(source,{...row,operationsPerIteration:2},kind));}
 assert.deepEqual(expectedNumericLiteralsValues(row.id,1,3,'tail'),[['3','57071','3000','4.2','18446744073709552000','tail']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedNumericLiteralsObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertNumericLiteralsSource(source'));assert.ok(source.includes("'numeric-literals-oracle.mjs'"));}
});
