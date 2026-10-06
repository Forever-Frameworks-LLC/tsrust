import fs from 'node:fs';
import vm from 'node:vm';
import test from 'node:test';
import assert from 'node:assert/strict';
import ts from './node_modules/typescript/lib/typescript.js';
import {expectedFixedNumber,expectedFixedNumberTokens} from './fixed-number-oracle.mjs';
import {expectedChecksum,expectedExactResult} from './oracle.mjs';
import {observeNumber} from './number-observation.mjs';
import {validateExactObservation} from './result-observation.mjs';
import {adaptTSDriver,assertCLIInputs} from './adapters.mjs';
const rows=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url))).workloads.filter(w=>w.id==='number_direct_literal_index'||w.id==='number_fixed_local_operations');
function counterpart(row){const context=vm.createContext({console:{log(){}}});vm.runInContext(ts.transpileModule(fs.readFileSync(new URL(`./workloads/${row.id}.ts`,import.meta.url),'utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,context);return context;}
test('fixed Number counterparts match54 independent completeIEEE/effect-order cases',()=>{
 assert.equal(rows.length,2);let cases=0;
 for(const row of rows){const context=counterpart(row);for(const [n,seed] of [...[0,1,2,8,17,65].flatMap(n=>[0,1,7,19].map(seed=>[n,seed])),...[0,7,1000000].map(seed=>[2000,seed])]){const actual=context.workload(n,seed),expected=expectedFixedNumber(row.id,n,seed);validateExactObservation(observeNumber(actual),expected);assert.equal(actual.length,row.id==='number_direct_literal_index'?n+2:n*18);assert.equal(expectedChecksum(row.id,n,seed),expected.checksum);assert.deepEqual(expectedExactResult(row.id,n,seed),expected);cases++;}}
 assert.equal(cases,54);
});
test('fixed Number preservesIEEE boundaries, ordered effects, and the matched membership domain',()=>{
 let guards=0;for(const row of rows){const context=counterpart(row);for(const [n,seed]of[[-1,0],[2001,0],[0,-1],[0,1000001],[.5,0]]){assert.throws(()=>expectedFixedNumber(row.id,n,seed),/Bounded/);guards++;}
 if(row.id==='number_direct_literal_index'){const actual=context.workload(8,0),tokens=observeNumber(actual).exactBits.slice(2).split(',');assert.equal(new Set(tokens.slice(0,8)).size,8);assert.equal(actual.at(-1),8*9);assert.equal(Object.is(actual[3],-0),true);assert.equal(Number.isNaN(actual[6]),true);assert.ok(tokens.includes('0000000000000001'));assert.ok(tokens.includes('7fefffffffffffff'));const probe={trace:0,calls:0};context.mark(probe,-0,1);context.mark(probe,NaN,2);context.mark_index(probe,0);assert.deepEqual(probe,{trace:1*131*131+2*131+9,calls:3});guards+=6;}
 else{const actual=context.workload(1,0);assert.equal(Object.is(actual[1],-0),true);assert.equal(Object.is(actual[14],-0),true);assert.equal(Number.isNaN(actual[2]),true);assert.equal(actual[16],1);assert.equal(actual[17],6);assert.equal(context.workload(1,5)[16],0);assert.equal([NaN].includes(NaN),true);assert.equal([NaN].indexOf(NaN),-1);assert.equal(expectedFixedNumberTokens(row.id,1,0)[15],expectedFixedNumberTokens(row.id,1,0)[0]);guards+=9;}}
 assert.equal(guards,25);assert.throws(()=>expectedFixedNumber('unknown',0,0),/Unknown/);assertCLIInputs(rows);
});
test('prepared fixed Number drivers validate completeactualresults outside the timer',()=>{
 for(const row of rows){const printed=[],context=vm.createContext({console:{log:line=>printed.push(JSON.parse(line))},process:{argv:['node','test','17','7','0','1']},performance:{now:()=>1}});const source=fs.readFileSync(new URL(`./workloads/${row.id}.ts`,import.meta.url),'utf8'),driver=adaptTSDriver(fs.readFileSync(new URL('./driver.ts',import.meta.url),'utf8'),row);vm.runInContext(ts.transpileModule(source+'\n'+driver,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,context);assert.equal(printed.length,1);assert.deepEqual(printed[0].durations,[0]);validateExactObservation(printed[0],expectedFixedNumber(row.id,17,7));}
});
