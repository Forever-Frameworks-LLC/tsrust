import test from 'node:test';import fs from 'node:fs';import vm from 'node:vm';import assert from 'node:assert/strict';
import ts from './node_modules/typescript/lib/typescript.js';
import {observeNumber,validateNumberObservation} from './number-observation.mjs';
import {expectedMath,expectedMathBits,sqrtRationalBits,addFiniteBits} from './math-oracle.mjs';
import {expectedNumberResult,expectedChecksum} from './oracle.mjs';
import {adaptTSDriver} from './adapters.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url)));

test('five actual Math sources match120 independent scalar-result cases including ordinary sqrt values',()=>{
 let cases=0;
 for(const row of manifest.workloads.filter(row=>/^math_(abs|floor|ceil|trunc|sqrt)$/.test(row.id))) {
  const context=vm.createContext({console});const source=fs.readFileSync(new URL(`./workloads/${row.id}.ts`,import.meta.url),'utf8');
  vm.runInContext(ts.transpileModule(source,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,context);
  for(const n of [0,1,2,17,24,127])for(const seed of [0,7,19,97]) {
   const expected=expectedMath(row.id,n,seed);validateNumberObservation(observeNumber(context.workload(n,seed)),expected);
   assert.deepEqual(expectedNumberResult(row.id,n,seed),expected);assert.equal(expectedChecksum(row.id,n,seed),expected.checksum);cases++;
  }
  for(const [n,seed]of[[-1,0],[0,-1],[1000001,0],[0,1000001]])assert.throws(()=>expectedMath(row.id,n,seed),/Bounded/);
  assert.equal(fs.readFileSync(new URL(`./workloads/${row.id}.tsr`,import.meta.url),'utf8').indexOf('function workload'),0);
 }
 assert.equal(cases,120);
});

test('all110 direct Math IEEE cases cover signs,nonfinite,fractional,boundary and irrational values',()=>{
 const finite=(numerator,denominator=1n,negativeZero=false)=>({numerator,denominator,negativeZero}),huge=(2n**53n-1n)<<971n;
 const domain=[[NaN,{kind:'nan'}],[Infinity,{kind:'infinity'}],[-Infinity,{kind:'infinity',negative:true}],[-0,finite(0n,1n,true)],[0,finite(0n)],[-.25,finite(-1n,4n)],[.25,finite(1n,4n)],[-1.75,finite(-7n,4n)],[1.75,finite(7n,4n)],[-2.5,finite(-5n,2n)],[2.5,finite(5n,2n)],[Number.MIN_VALUE,finite(1n,1n<<1074n)],[-Number.MIN_VALUE,finite(-1n,1n<<1074n)],[Number.MAX_VALUE,finite(huge)],[-Number.MAX_VALUE,finite(-huge)],[4503599627370495.5,finite(9007199254740991n,2n)],[4503599627370496,finite(4503599627370496n)],[9007199254740991,finite(9007199254740991n)],[2,finite(2n)],[10,finite(10n)],[.1,finite(3602879701896397n,1n<<55n)],[25,finite(25n)]];
 let cases=0;for(const method of ['abs','floor','ceil','trunc','sqrt'])for(const [value,reference]of domain){assert.equal(observeNumber(Math[method](value)).exactBits,'n:'+expectedMathBits(method,reference),`${method}:${value}`);cases++;}
 assert.equal(cases,110);assert.throws(()=>expectedMath('unknown',0,0),/Unknown/);assert.throws(()=>expectedMathBits('unknown',finite(0n)),/Unknown/);
});

test('sqrt oracle rounds exact midpoint ties to even and prepared driver retains ordered IEEE accumulation',()=>{
 const midpoint=(1n<<53n)+1n;assert.equal(sqrtRationalBits(midpoint*midpoint,1n<<106n),'3ff0000000000000');
 const oddMidpoint=(1n<<53n)+3n;assert.equal(sqrtRationalBits(oddMidpoint*oddMidpoint,1n<<106n),'3ff0000000000002');
 assert.equal(sqrtRationalBits(4n),'4000000000000000');assert.equal(sqrtRationalBits(0n,1n,true),'8000000000000000');assert.equal(sqrtRationalBits(-1n),'nan');
 assert.equal(addFiniteBits(observeNumber(.1).exactBits.slice(2),observeNumber(.2).exactBits.slice(2)),observeNumber(.1+.2).exactBits.slice(2));
 const row=manifest.workloads.find(row=>row.id==='math_sqrt'),printed=[];
 const context=vm.createContext({console:{log:line=>printed.push(JSON.parse(line))},process:{argv:['node','test','17','7','0','1']},performance:{now:()=>1}});
 const source=fs.readFileSync(new URL('./workloads/math_sqrt.ts',import.meta.url),'utf8'),driver=adaptTSDriver(fs.readFileSync(new URL('./driver.ts',import.meta.url),'utf8'),row);
 vm.runInContext(ts.transpileModule(source+'\n'+driver,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,context);
 assert.equal(printed.length,1);validateNumberObservation(printed[0],expectedMath(row.id,17,7));assert.deepEqual(printed[0].durations,[0]);
});
