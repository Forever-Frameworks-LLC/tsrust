import test from 'node:test';import fs from 'node:fs';import vm from 'node:vm';import assert from 'node:assert/strict';
import ts from './node_modules/typescript/lib/typescript.js';
import {expectedPredicate,predicateInputs,classifyPredicate} from './number-predicate-oracle.mjs';
import {expectedChecksum} from './oracle.mjs';
const methods={number_is_finite:'isFinite',number_is_nan:'isNaN',number_is_integer:'isInteger',number_is_safe_integer:'isSafeInteger'};

test('four actual predicate sources match independent rational classifications across140 complete cases',()=>{
 let cases=0;
 for(const id of Object.keys(methods)) {
  const context=vm.createContext({console});
  vm.runInContext(ts.transpileModule(fs.readFileSync(new URL(`./workloads/${id}.ts`,import.meta.url),'utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,context);
  for(const n of [0,1,2,17,24,49,257])for(const seed of [0,7,19,23,97]) {assert.equal(context.workload(n,seed),expectedPredicate(id,n,seed));assert.equal(expectedChecksum(id,n,seed),expectedPredicate(id,n,seed));cases++;}
  for(const [n,seed]of[[-1,0],[0,-1],[1000001,0],[0,1000001],[.5,0],[0,NaN]])assert.throws(()=>expectedPredicate(id,n,seed),/Bounded/);
  assert.equal(fs.readFileSync(new URL(`./workloads/${id}.tsr`,import.meta.url),'utf8').indexOf('function workload'),0);
 }
 assert.equal(cases,140);
});

test('all96 predicate input classifications cover nonfinite signs,subnormals,and integer/safe boundaries',()=>{
 const values=[NaN,Infinity,-Infinity,-0,0,1,-1,1.5,-1.5,Number.MIN_VALUE,-Number.MIN_VALUE,Number.MAX_VALUE,-Number.MAX_VALUE,9007199254740991,-9007199254740991,9007199254740992,-9007199254740992,4503599627370495.5,4503599627370496,4503599627370497,10,-10,.25,-.25];
 for(const method of Object.values(methods))for(let i=0;i<24;i++)assert.equal(Number[method](values[i]),classifyPredicate(method,predicateInputs[i]),`${method}:${i}`);
 assert.equal(Number.isInteger(Number.MAX_VALUE),true);assert.equal(Number.isSafeInteger(Number.MAX_VALUE),false);
 for(const value of [0,-0]){assert.equal(Number.isInteger(value),true);assert.equal(Number.isSafeInteger(value),true);}
});

test('static Node predicates do not use global coercions; unknown oracle methods fail closed',()=>{
 for(const method of Object.values(methods))for(const value of ['1',null,undefined,true,{},[]])assert.equal(Number[method](value),false);
 assert.throws(()=>expectedPredicate('unknown',0,0),/Unknown/);assert.throws(()=>classifyPredicate('unknown',predicateInputs[0]),/Unknown/);
});
