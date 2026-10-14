import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import {reusableSchemaAllocation,directSchemaObjectAllocation} from './schema-allocation-adapter.mjs';
import {directJsonAllocation,jsonAllocationInput,jsonAllocationChecksum} from './json-allocation-adapter.mjs';
const driver=fs.readFileSync(new URL('./allocation-driver.rs',import.meta.url),'utf8');
test('reusable schema plan construction is excluded and disposal remains checked',()=>{
 for(const constructor of ['fn function0() -> Record2 {','fn tsr_result_function0() -> Result<Record2,TsrFunctionError0<String>> {']){
  const emitted=constructor+'todo!()}\nfn tsr_result_function1(value0: &Record2, value1: &String) -> Result<NumericArray<i64>,TsrFunctionError1<String>> {todo!()}\nfn main(){}';
  const adapted=reusableSchemaAllocation(emitted,driver);
  assert(adapted.indexOf('let schema =')<adapted.indexOf('ALLOCS.store(0')); 
  assert.match(adapted,/base_live - base_before_input - schema_requested_bytes/);
  assert.match(adapted,/schemaRequestedBytes/);
  assert.match(adapted,/liveBytesAfterOutputDrop/);
  assert.match(adapted,/liveBytesAfterDrop/);
  assert.match(adapted,/black_box\(&schema\)/);
  assert.throws(()=>reusableSchemaAllocation(emitted.replace('value0: &Record2','value0: Record2'),driver));
  assert.throws(()=>reusableSchemaAllocation(emitted,driver.replace('ALLOCS.store(0','ALLOCS.store(1')));
 }
});

test('reusable object reference enforces its shape and independently observes every integer',()=>{
 assert.equal(jsonAllocationInput(3,'schema_reusable_record'),'{"items":[7,8,9]}');
 assert.equal(jsonAllocationChecksum(3,'schema_reusable_record'),24);
 const direct=directSchemaObjectAllocation(driver,directJsonAllocation);
 assert.match(direct,/Error::unknown_field/);
 assert.match(direct,/from_str::<SchemaPayload>/);
 assert.match(direct,/unwrap\(\).items/);
 assert.match(direct,/value.iter\(\).sum\(\)/);
});

test('reusable schema binding pairs constructor and workload by exact borrowed type',()=>{
 const workload='fn tsr_result_function10(value0: &Record16, value1: &String) -> Result<NumericArray<i64>,TsrFunctionError10<String>> {todo!()}';
 for(const constructor of ['fn function11() -> Record16 {todo!()}', 'fn tsr_result_function11() -> Result<Record16,TsrFunctionError11<String>> {todo!()}']){
  const source=workload+'\n'+constructor+'\nfn main() {}';
  const adapted=reusableSchemaAllocation(source,driver);
  assert.match(adapted,/let schema = (match tsr_result_)?function11\(\)/);
  assert.match(adapted,/match tsr_result_function10\(std::hint/);
  assert.throws(()=>reusableSchemaAllocation(source+'\n'+workload.replace('function10','function12'),driver),/ABI drift/);
  assert.throws(()=>reusableSchemaAllocation(source.replace('value0: &Record16','value0: &Record15'),driver),/ABI drift/);
 }
});
