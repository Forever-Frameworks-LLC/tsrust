import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {assertCommaExpressionsSource} from './comma-expressions-source-contract.mjs';
import {expectedCommaExpressionsValues,expectedCommaExpressionsObservation} from './comma-expressions-oracle.mjs';
import {expectedExactResult} from './oracle.mjs';
import {reportCategory} from './report.mjs';
const read=p=>fs.readFileSync(new URL(p,import.meta.url),'utf8');
const row=JSON.parse(read('./manifest.json')).workloads.find(x=>x.id==='comma_expressions');
test('Comma expressions retain paired source forms, independent oracle and runtime hooks',()=>{
 assert.equal(row.measured,false);assert.equal(reportCategory(row.id),'numbers');
 for(const [kind,name] of [['native',row.source],['typescript',row.counterpart]]){const source=read('./'+name);assertCommaExpressionsSource(source,row,kind);assert.throws(()=>assertCommaExpressionsSource(source+' ',row,kind));assert.throws(()=>assertCommaExpressionsSource(source,{...row,operationsPerIteration:2},kind));}
 assert.deepEqual(expectedCommaExpressionsValues(row.id,1,3,'tail'),[['9','10','3,5,9,7','tail','0','3']]);
 assert.deepEqual(expectedExactResult(row.id,2,3,'tail'),expectedCommaExpressionsObservation(row.id,2,3,'tail'));
 for(const file of ['./run.mjs','./profile-native.mjs']){const source=read(file);assert.ok(source.includes('assertCommaExpressionsSource(source'));assert.ok(source.includes("'comma-expressions-oracle.mjs'"));}
});
