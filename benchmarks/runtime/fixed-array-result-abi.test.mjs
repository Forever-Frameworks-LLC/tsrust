import test from 'node:test';
import assert from 'node:assert/strict';
import {assertABI,rustChecksum} from './adapters.mjs';
import {fixedArrayResults} from './fixed-array-result-abi.mjs';
const workload={id:'fixed_rows',result:'string[][]',resultEncoding:'array-shape-scalar-v1'};
const contract={function:'function0',result:'Vec<[String; 5]>',element:'String',width:5,rank:2};
const declaration='fn function0(value0: i64, value1: i64) -> Vec<[String; 5]> { Vec::new() }\nfn main() {}';
const emitted=`// tsrust-fixed-array-result ${JSON.stringify(contract)}\n${declaration}`;
test('fixed row ABI requires a matching compiler contract and complete signature',()=>{
 assert.equal(assertABI(emitted,workload).result,'Vec<[String; 5]>');
 assert.equal(assertABI(emitted,workload).fallible,false);
 assert.throws(()=>assertABI(declaration,workload),/Emitter ABI changed/);
 assert.throws(()=>assertABI(emitted,{...workload,result:'int[][]'}),/Emitter ABI changed/);
 assert.throws(()=>fixedArrayResults(emitted.replace('"width":5','"width":4')),/disagrees/);
 assert.throws(()=>fixedArrayResults(emitted.replace('"rank":2','"rank":1')),/disagrees/);
 assert.throws(()=>fixedArrayResults(emitted+'\n'+emitted),/Invalid/);
 assert.throws(()=>fixedArrayResults(emitted.replace('"width":5','"width":-1')),/Invalid/);
 assert.throws(()=>fixedArrayResults(emitted.replace('"function":"function0"','"function":null')),/Invalid/);
 assert.throws(()=>fixedArrayResults(emitted.replace('"function":"function0"','"function":["function0"]')),/Invalid/);
 assert.throws(()=>fixedArrayResults('// tsrust-fixed-array-result {broken}'),/Malformed/);
});
test('fixed row observation preserves full array boundaries without unused containers',()=>{
 const observer=rustChecksum(workload,{emitted});
 assert.match(observer,/SliceTree for \[T;N\]/);
 assert.doesNotMatch(observer,/SliceTree for Array<T>|SliceTree for NumericArray<T>/);
 assert.match(observer,/StringStart/);
 assert.match(observer,/ArrayStart/);
 assert.match(observer,/slice_exact/);
});
