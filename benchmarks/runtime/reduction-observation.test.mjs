// Gate51 focused source preparation. Tests are unexecuted until parent grants checks.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {observeReduction,validateReductionObservation,isReduction} from './reduction-observation.mjs';
import {verifyReductionABI,assertReductionSource} from './reduction-abi.mjs';
import {assertABI,adaptRustDriver,rustChecksum} from './adapters.mjs';
import {validateExactObservation,requiresExactResult} from './result-observation.mjs';
import {observeNumber} from './number-observation.mjs';
import {validationProtocol} from './approximate-scalar.mjs';
import {expectedReduction,explicitExpected} from './reduction-oracle.mjs';
import {expectedReductionObservation} from './reduction-results.mjs';
import {reportCategory} from './report.mjs';
const rows=JSON.parse(fs.readFileSync(new URL('manifest.json',import.meta.url))).workloads.filter(isReduction);
const read=name=>fs.readFileSync(new URL(name,import.meta.url),'utf8');
const row=rows.find(w=>w.id==='array_reduce_record'),source=read('workloads/'+row.id+'.tsr');
// Synthetic ABI parser fixture only: NOT accepted native compiler evidence.
const emitted="type String = std::borrow::Cow<'static, str>;\nstruct Record7 { field0: i64, field1: String, field2: Vec<Vec<i64>>, field3: i64, field4: i64 }\nfn function0(value0: i64, value1: i64) -> Record7 {}\nfn main() {}";
test('28 reduction domains preserve scalar results, exact ranks and Arrays categories',()=>{
 assert.equal(rows.length,28);
 for(const w of rows){assert.equal(reportCategory(w.id),'arrays');assert.doesNotThrow(()=>assertReductionSource(read('workloads/'+w.id+'.tsr'),w));for(const n of [0,1,2,17])for(const seed of [0,1,2,7,1000000]){const expected=expectedReductionObservation(w.id,n,seed);assert.ok(Number.isSafeInteger(expected.checksum));if(requiresExactResult(w))validateExactObservation(expected,expected);}}
 for(const cell of explicitExpected)assert.deepEqual(expectedReduction(cell.id,cell.n,cell.seed),cell.value);
});
test('actual ABI derives Record7 and nested Vec fields instead of trusting provisional names',()=>{
 for(const result of ['boolean','string','Row','boolean[][]','Row[][]'])assert.equal(validationProtocol({...row,result}),'legacy-exact');
 for(const invalid of [{...row,reductionObserver:undefined},{...row,result:'int'},{...row,approximateValidation:{protocol:'mpfr-directed-f64-scalar-v1'}},{...row,resultEncoding:'unknown-exact'}])assert.throws(()=>validationProtocol(invalid));
 const binding=verifyReductionABI(emitted,row,source);assert.equal(binding.record.name,'Record7');assert.equal(binding.resultType,'Record7');assert.deepEqual(binding.containers,['Vec']);assert.equal(binding.record.fields[2].type,'Vec<Vec<i64>>');
 assert.deepEqual(assertABI(emitted,row,{source}),binding);
 const wrapper=adaptRustDriver(read('driver.rs'),row,{emitted,source});assert.match(wrapper,/impl ReductionTree for Record7/);assert.doesNotMatch(wrapper,/for Array<T>|for NumericArray<T>|Record0/);assert.ok(wrapper.indexOf('start.elapsed()')<wrapper.indexOf('validate(value)'));
 const checksum=rustChecksum(row,{probe:true,emitted,source});assert.match(checksum,/fn probe_checksum/);assert.doesNotMatch(checksum,/Record0|impl<T:ReductionTree> ReductionTree for Array/);
 for(const bad of [emitted.replace('field0: i64','field0: f64'),emitted.replace('field0:','field5:'),emitted.replace('field4: i64','field4: i64, field5: bool'),emitted.replace('Vec<Vec<i64>>','Vec<i64>'),emitted.replace('value1: i64','value1: f64'),emitted.replace('-> Record7','-> &Record7'),emitted.replace("Cow<'static, str>","Cow<'static, [u8]>"),emitted+'\nfn function0(value0:i64,value1:i64)->Record7 {}'])assert.throws(()=>verifyReductionABI(bad,row,source));
 assert.throws(()=>verifyReductionABI(emitted,row,source.replace('text:string','text:int')));
 assert.throws(()=>adaptRustDriver(read('driver.rs'),row));
});
test('record aliases and container support are bound to actual emitted declarations',()=>{
 const nested=rows.find(w=>w.id==='array_reduce_nested_record'),authored=read('workloads/'+nested.id+'.tsr');
 const alias=emitted.replace('-> Record7','-> Vec<RecordArray7>')+'\ntype RecordArray7=Vec<Record7>;';
 assert.equal(verifyReductionABI(alias,nested,authored).resultType,'Vec<RecordArray7>');
 assert.throws(()=>verifyReductionABI(alias.replace('Vec<Record7>;','Vec<f64>;'),nested,authored));
 assert.throws(()=>verifyReductionABI(alias.replace('Vec<Record7>;','RecordArray7;'),nested,authored));
 assert.throws(()=>verifyReductionABI(alias.replace('Vec<Record7>;','Array<Record7>;'),nested,authored));
});
test('complete record/Boolean/String tokens reject missing fields, type erasure, NUL and rank changes',()=>{
 const value={total:1,text:'a\u0000😀e\u0301',rows:[[],[2,3]],trace:4,visits:5},expected=observeReduction(value,'Row');validateReductionObservation(expected,expected);
 const nested=observeReduction([[],[value]],'Row[][]');validateReductionObservation(nested,nested);
 assert.throws(()=>observeReduction({...value,visits:undefined},'Row'));assert.throws(()=>observeReduction({...value,extra:0},'Row'));assert.throws(()=>observeReduction({...value,text:'\ud800'},'Row'));assert.throws(()=>observeReduction([[1]],'boolean[][]'));
 for(const token of [expected.exactBits.replace('i(5)','b(1)'),expected.exactBits.replace(',i(5)',''),expected.exactBits.replace('97,0,','97,1,'),expected.exactBits+'junk',expected.exactBits.replace('i(1)','i(01)'),expected.exactBits.replace('a(),',''),expected.exactBits.replace('s(97','s(55296')])assert.throws(()=>validateReductionObservation({...expected,exactBits:token},expected));
 const bool=observeReduction(true,'boolean');assert.throws(()=>validateReductionObservation({...bool,exactBits:'r:i(1)'},bool));assert.throws(()=>validateReductionObservation({...bool,exactBits:'r:b(2)'},bool));assert.throws(()=>validateReductionObservation({...bool,checksum:bool.checksum+1},bool));
 const empty=observeReduction([[]],'boolean[][]');assert.throws(()=>validateReductionObservation({...empty,exactBits:'r:a()'},empty));
 assert.throws(()=>validateExactObservation(observeNumber(-0),observeNumber(0)));
});
test('independent reduction domains reject unsupported IDs, coordinates and unbounded construction',()=>{
 for(const args of [['bad',1,1],['array_reduce_int',-1,1],['array_reduce_number',1.5,1],['array_reduce_string',129,1],['array_reduce_int',1,-1],['array_reduce_int',1,1000001]])assert.throws(()=>expectedReduction(...args));
});
