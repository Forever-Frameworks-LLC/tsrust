import {createRequire} from 'node:module';
import {isCopying,validateCopyingObservation} from './copying-array-observation.mjs';
import {isRange,validateRangeObservation} from './range-mutation-observation.mjs';
import {isHigher,validateHigherObservation} from './higher-order-observation.mjs';
import {isDeclared,validateDeclaredObservation} from './declared-value-observation.mjs';
import {isConcat,validateConcatObservation} from './concat-observation.mjs';
import {execFileSync} from 'node:child_process';
import {fileURLToPath} from 'node:url';
import test from 'node:test';import fs from 'node:fs';import vm from 'node:vm';import assert from 'node:assert/strict';
import ts from './node_modules/typescript/lib/typescript.js';
import {expectedSlices,expectedSliceObservation,expectedSliceQueries,normalizeBound} from './slice-oracle.mjs';
import {observeSliceTree} from './slice-observation.mjs';
import {isSliceResult,requiresExactResult,validateExactObservation} from './result-observation.mjs';
import {expectedExactResult,expectedChecksum} from './oracle.mjs';
import {adaptTSDriver,adaptRustDriver,assertABI,rustResults} from './adapters.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url)));

test('four complete retained slice sources match96 independent fullshape/content references',()=>{
 let cases=0;
 for(const row of manifest.workloads.filter(row=>row.id.startsWith('array_slice_'))) {
  const context=vm.createContext({console});vm.runInContext(ts.transpileModule(fs.readFileSync(new URL(`./workloads/${row.id}.ts`,import.meta.url),'utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,context);
  for(const n of [0,1,2,16,17,65])for(const seed of [0,7,19,97]) {
   const actual=context.workload(n,seed,...(row.inputText===undefined?[]:[row.inputText])),expected=expectedSliceObservation(row.id,n,seed,row.inputText);
   validateExactObservation(observeSliceTree(actual,{numbers:row.id==='array_slice_number'}),expected);
   assert.deepEqual(expectedExactResult(row.id,n,seed,row.inputText),expected);assert.equal(expectedChecksum(row.id,n,seed,row.inputText),expected.checksum);cases++;
  }
  for(const [n,seed]of[[-1,0],[2001,0],[0,-1],[0,1000001],[.5,0]])assert.throws(()=>expectedSlices(row.id,n,seed),/Bounded/);
  assert.equal(fs.readFileSync(new URL(`./workloads/${row.id}.tsr`,import.meta.url),'utf8').indexOf('function workload'),0);
 }
 assert.equal(cases,96);
});

test('query/rank observers preserve all16bounds,emptyrows,scalarstrings and signedzero',()=>{
 assert.equal(normalizeBound(-9223372036854775808n,8),0);assert.equal(normalizeBound(9223372036854775807n,8),8);
 assert.deepEqual(expectedSliceQueries(8).map(({first,last})=>last-first),[8,8,7,1,8,8,0,2,0,2,0,0,0,0,2,7]);
 assert.notDeepEqual(observeSliceTree([[],[[]]]),observeSliceTree([[[]],[]]));assert.notDeepEqual(observeSliceTree([[''],[]]),observeSliceTree([[],['']]));
 assert.notDeepEqual(observeSliceTree([[0,-0]],{numbers:true}),observeSliceTree([[-0,0]],{numbers:true}));
 const observed=observeSliceTree([[0,-0,NaN,Infinity,-Infinity],[]],{numbers:true});validateExactObservation(observed,observed);
 assert.equal(observeSliceTree([['a\0😀']]).exactBits,'t:[[s(97,0,128512)]]');
 assert.throws(()=>expectedSlices('unknown',0,0),/Unknown/);assert.throws(()=>expectedSlices('array_slice_string',0,0,'\ud800'),/Unicode/);
});

test('exact tree validation fails closed on missingtokens,malformedshape,wrongempties or noncanonicalNaN',()=>{
 const expected=observeSliceTree([[],[[]]]);for(const actual of [{checksum:expected.checksum},{...expected,exactBits:'t:[,[[]]]'},{...expected,exactBits:'t:[[[]],[]]'},{...expected,exactBits:'t:[s(55296)]'},{...expected,exactBits:'t:[s(1114112)]'},{...expected,exactBits:'t:[9999999999999999999]'}])assert.throws(()=>validateExactObservation(actual,expected),/array/);
 for(const token of ['m:[7ff8000000000000]','m:[[nan]]extra','m:[nan,]','m:[s()]','m:0'])assert.throws(()=>validateExactObservation({checksum:1,exactBits:token},{checksum:1,exactBits:token}),/array/);
 assert.equal(requiresExactResult({result:'number'}),true);assert.equal(requiresExactResult({result:'int[][]'}),false);
});

test('prepared rankdrivers observe actualfullvalues aftertiming with readonlyinput outside allocationcounters',()=>{
 assert.equal(rustResults['string[][]'],'Vec<Array<String>>');assert.equal(rustResults['int[][][]'],'Vec<Array<NumericArray<i64>>>');
 for(const row of manifest.workloads.filter(isSliceResult)) {
  const printed=[],context=vm.createContext({TextEncoder,TextDecoder,require:createRequire(import.meta.url),console:{log:line=>printed.push(JSON.parse(line))},process:{argv:['node','test','17','7','0','1',row.inputText??'']},performance:{now:()=>1}});
  const source=fs.readFileSync(new URL(`./workloads/${row.id}.ts`,import.meta.url),'utf8'),driver=adaptTSDriver(fs.readFileSync(new URL('./driver.ts',import.meta.url),'utf8'),row);
  vm.runInContext(ts.transpileModule(source+'\n'+driver,{compilerOptions:{alwaysStrict:true,target:ts.ScriptTarget.ES2022}}).outputText,context);
  assert.equal(printed.length,1);
  const validate=isCopying(row)?validateCopyingObservation:isRange(row)?validateRangeObservation:isHigher(row)?validateHigherObservation:isDeclared(row)?validateDeclaredObservation:isConcat(row)?validateConcatObservation:validateExactObservation;
  try { validate(printed[0],expectedExactResult(row.id,17,7,row.inputText),row); } catch(error) { throw new Error(row.id+': '+error.message,{cause:error}); }
  assert.deepEqual(printed[0].durations,[0]);
  const nativeSource=fs.readFileSync(new URL(`./workloads/${row.id}.tsr`,import.meta.url),'utf8');
  const emitted=execFileSync(fileURLToPath(new URL('../../target/editor/debug/tsrust',import.meta.url)),['emit',fileURLToPath(new URL(`./workloads/${row.id}.tsr`,import.meta.url))],{encoding:'utf8',timeout:10000,maxBuffer:16*1024*1024});
  const native=adaptRustDriver(fs.readFileSync(new URL('./driver.rs',import.meta.url),'utf8'),row,{emitted,source:nativeSource});
  if(!isCopying(row)&&!isRange(row)&&!isHigher(row)&&!isDeclared(row)&&!isConcat(row)&&row.result==='number[][]'){assert.equal(native.includes('SliceTree for Array<T>'),false);assert.equal(native.includes('SliceTree for NumericArray<T>'),false);}
  if(!isCopying(row)&&!isRange(row)&&!isHigher(row)&&!isDeclared(row)&&!isConcat(row)&&row.result==='string[][]'){assert.equal(native.includes('SliceTree for Array<T>'),true);assert.equal(native.includes('SliceTree for NumericArray<T>'),false);}
  assert.ok(native.indexOf('durations.push(start.elapsed()')<native.indexOf('validate(value)'));
  assertABI(emitted,row,{source:nativeSource});
  if(row.inputText!==undefined){assert.equal((native.match(/black_box\(seed\), &input/g)||[]).length,2);const probe=adaptRustDriver(fs.readFileSync(new URL('./allocation-driver.rs',import.meta.url),'utf8'),row,{probe:true,emitted,source:nativeSource});assert.ok(probe.indexOf('let input =')<probe.indexOf('let base_live'));}
 }
});
