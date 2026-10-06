import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import vm from 'node:vm';
import ts from './node_modules/typescript/lib/typescript.js';
import {observeNumber,isNumberResult,validateNumberObservation} from './number-observation.mjs';
import {expectedNumber,rationalBits,tokensObservation,referenceComparison} from './number-oracle.mjs';
import {adaptTSDriver,adaptRustDriver,assertABI} from './adapters.mjs';
const manifest=JSON.parse(fs.readFileSync(new URL('./manifest.json',import.meta.url)));

test('sixteen Number counterparts and scalar memory probe match independent rational/IEEE references',()=>{
 let cases=0;
 for(const row of [...manifest.workloads.filter(row=>/^number_(add|subtract|multiply|divide|remainder|negate|equal|not_equal|less|less_equal|greater|greater_equal|from_int|from_boolean|parse_decimal|array_operations)$/.test(row.id)),{id:'number_value',result:'number'}]) {
  const context=vm.createContext({console});
  const location=row.id==='number_value'?'allocations':'workloads';
  vm.runInContext(ts.transpileModule(fs.readFileSync(new URL(`./${location}/${row.id}.ts`,import.meta.url),'utf8'),{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,context);
  for(const n of [0,1,2,17,127])for(const seed of [0,7,19,97]) {
   const value=context.workload(n,seed),actual=isNumberResult(row)?observeNumber(value):{checksum:value};
   assert.deepEqual(actual,expectedNumber(row.id,n,seed));cases++;
  }
  for(const [n,seed]of[[-1,0],[0,-1],[1000001,0],[0,1000001]])assert.throws(()=>expectedNumber(row.id,n,seed),/Bounded/);
 }
 assert.equal(cases,340);
});

test('floating transport preserves zero/infinity signs, canonicalizes NaN, rejects missing/mismatched tokens',()=>{
 for(const [value,token]of[[0,'0000000000000000'],[-0,'8000000000000000'],[Infinity,'7ff0000000000000'],[-Infinity,'fff0000000000000'],[NaN,'nan']]) {
  const expected=tokensObservation([token]);assert.deepEqual(observeNumber(value),expected);validateNumberObservation(observeNumber(value),expected);
 }
 const view=new DataView(new ArrayBuffer(8));view.setBigUint64(0,0xfff8000000000001n);assert.deepEqual(observeNumber(view.getFloat64(0)),tokensObservation(['nan']));
 assert.notDeepEqual(observeNumber(0),observeNumber(-0));assert.notDeepEqual(observeNumber([0,-0]),observeNumber([-0,0]));
 validateNumberObservation(observeNumber([]),tokensObservation([],true));
 const zero=observeNumber(0);
 for(const actual of [{checksum:zero.checksum},{...zero,exactBits:'n:0'},{...zero,checksum:NaN},{...zero,exactBits:'a:'},{...zero,exactBits:observeNumber(-0).exactBits},{...zero,checksum:zero.checksum+1}])assert.throws(()=>validateNumberObservation(actual,zero),/Number/);
 assert.equal(rationalBits(1n,10n),observeNumber(.1).exactBits.slice(2));
 assert.equal(rationalBits(1n,1n<<1074n),observeNumber(Number.MIN_VALUE).exactBits.slice(2));
 assert.equal(rationalBits(1n,1n<<1075n),'0000000000000000');assert.equal(rationalBits(3n,1n<<1075n),'0000000000000002');
 assert.equal(rationalBits(1n<<1024n),observeNumber(Infinity).exactBits.slice(2));
});

test('six comparison operators match independent IEEE ordering over 384 nonfinite/zero/finite combinations',()=>{
 const values=[NaN,Infinity,-Infinity,-0,0,1.5,-2.25,2.25],refs=['nan','positiveInfinity','negativeInfinity',0n,0n,6n,-9n,9n];
 for(let a=0;a<8;a++)for(let b=0;b<8;b++)for(const [op,call]of Object.entries({equal:(a,b)=>a===b,not_equal:(a,b)=>a!==b,less:(a,b)=>a<b,less_equal:(a,b)=>a<=b,greater:(a,b)=>a>b,greater_equal:(a,b)=>a>=b}))assert.equal(call(values[a],values[b]),referenceComparison(op,refs[a],refs[b]));
 for(const [text,token]of[['','0000000000000000'],[' -0 ','8000000000000000'],['invalid','nan'],['1e309','7ff0000000000000']])assert.deepEqual(observeNumber(Number(text)),tokensObservation([token]));
 assert.throws(()=>expectedNumber('unknown',0,0),/Unknown/);
});

test('prepared Number TS wrapper observes exact results after timing and int wrappers retain their protocol',()=>{
 for(const id of ['number_add','number_array_operations']) {
  const row=manifest.workloads.find(row=>row.id===id),printed=[];
  const context=vm.createContext({console:{log:line=>printed.push(JSON.parse(line))},process:{argv:['node','test','2','7','0','1']},performance:{now:()=>1}});
  const authored=fs.readFileSync(new URL(`./workloads/${id}.ts`,import.meta.url),'utf8');
  const driver=adaptTSDriver(fs.readFileSync(new URL('./driver.ts',import.meta.url),'utf8'),row);
  vm.runInContext(ts.transpileModule(authored+'\n'+driver,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,context);
  assert.equal(printed.length,1);validateNumberObservation(printed[0],expectedNumber(id,2,7));assert.deepEqual(printed[0].durations,[0]);
  assertABI(`fn function0(value0: i64, value1: i64) -> ${row.result==='number'?'f64':'Vec<f64>'} {} fn main() {}`,row);
  assert.throws(()=>assertABI('fn function0(value0: i64, value1: i64) -> i64 {} fn main() {}',row),/ABI changed/);
  const native=adaptRustDriver(fs.readFileSync(new URL('./driver.rs',import.meta.url),'utf8'),row);
  assert.ok(native.indexOf('durations.push(start.elapsed()')<native.indexOf('validate(value)'));
 }
 const driver=fs.readFileSync(new URL('./driver.ts',import.meta.url),'utf8');assert.equal(adaptTSDriver(driver,{result:'int'}),driver);
});

test('Number memory wrapper uses exact bits after heap measurements; native digest does not allocate',async()=>{
 const {adaptNumberMemoryDriver,rustChecksum}=await import('./adapters.mjs');
 const nodeDriver=adaptNumberMemoryDriver(fs.readFileSync(new URL('./allocation-driver.ts',import.meta.url),'utf8'));
 const context=vm.createContext({process:{argv:['node','probe','8'],memoryUsage:()=>({heapUsed:1024})},gc:()=>{},console:{log:line=>context.output=JSON.parse(line)}});
 const source=fs.readFileSync(new URL('./allocations/number_value.ts',import.meta.url),'utf8');
 const support=fs.readFileSync(new URL('./number-observation.ts',import.meta.url),'utf8');
 vm.runInContext(ts.transpileModule(source+'\n'+support+'\n'+nodeDriver,{compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText,context);
 validateNumberObservation(context.output,expectedNumber('number_value',8,7));assert.equal(context.output.retainedHeapDelta,0);
 const native=adaptNumberMemoryDriver(fs.readFileSync(new URL('./allocation-driver.rs',import.meta.url),'utf8'),{native:true});
 assert.ok(native.indexOf('let after_drop =') < native.indexOf('let exact_bits = number_exact_bits'));
 assert.match(native,/input_requested_bytes, after_output_drop, number_value/);
 assert.throws(()=>adaptNumberMemoryDriver('missing hook',{native:true}),/hook changed/);
 const checksum=rustChecksum({result:'number'},{probe:true});
 const digest=checksum.slice(checksum.indexOf('fn number_checksum'),checksum.indexOf('fn number_exact_bits'));
 assert.doesNotMatch(digest,/String|Vec|format!|collect|Box|push/);
});
