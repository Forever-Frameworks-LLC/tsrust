import fs from 'node:fs';
// Reusable schema plans are built before operation counters and dropped afterward.
function replaceOnce(source,before,after){
 if(source.split(before).length!==2)throw Error(`Schema allocation ABI drift: ${before}`);
 return source.replace(before,after);
}
export function reusableSchemaAllocation(emitted,driver){
 driver=replaceOnce(driver,'        ALLOCS.store(0, Relaxed);','        ALLOCS.store(0, Relaxed);');
 const constructors=[...emitted.matchAll(/^fn (?<call>function\d+)\(\) -> (?<schema>Record\d+) \{/gm),...emitted.matchAll(/^fn (?<call>tsr_result_function\d+)\(\) -> Result<(?<schema>Record\d+),TsrFunctionError[^\n]+/gm)];
 const bindings=constructors.flatMap(constructor=>[...emitted.matchAll(new RegExp(`^fn (tsr_result_function\\d+)\\(value0: &${constructor.groups.schema}, value1: &String\\) -> Result<NumericArray<i64>,TsrFunctionError`, 'gm'))].map(workload=>({constructor,workload:workload[1]})));
 if(bindings.length!==1)throw Error('Reusable schema return/borrow ABI drift');
 const binding=bindings[0],name=binding.constructor.groups.call;
 const constructor=name.startsWith('tsr_result_')?`match ${name}(){Ok(value)=>value,Err(_)=>panic!("schema construction failed")}`:`${name}()`;
 let adapted=replaceOnce(driver,'let n: i64 = arguments[1].parse().unwrap();',`let input = String::from(arguments[2].clone());
        let before_schema = LIVE.load(Relaxed);
        let schema = ${constructor};
        let schema_requested_bytes = LIVE.load(Relaxed) - before_schema;
        let n: i64 = arguments[1].parse().unwrap();`);
 adapted=replaceOnce(adapted,'input_requested_bytes, after_output_drop) = {','input_requested_bytes, after_output_drop, schema_requested_bytes) = {');
 adapted=replaceOnce(adapted,'let input_requested_bytes = base_live - base_before_input;','let input_requested_bytes = base_live - base_before_input - schema_requested_bytes;');
 adapted=replaceOnce(adapted,'function0(std::hint::black_box(n), std::hint::black_box(7))',`match ${binding.workload}(std::hint::black_box(&schema), std::hint::black_box(&input)){Ok(value)=>value,Err(_)=>panic!("schema input rejected")}`);
 adapted=replaceOnce(adapted,'input_requested_bytes, after_output_drop)\n','input_requested_bytes, after_output_drop, schema_requested_bytes)\n');
 adapted=replaceOnce(adapted,'\\"inputRequestedBytes\\":{input_requested_bytes},','\\"inputRequestedBytes\\":{input_requested_bytes},\\"schemaRequestedBytes\\":{schema_requested_bytes},');
 return replaceOnce(emitted,'fn main()','fn source_main()')+'\n'+adapted+'\nfn probe_checksum(value:&NumericArray<i64>)->i64{value.iter().sum()}';
}

export function directSchemaObjectAllocation(driver,directJsonAllocation){
 const source=directJsonAllocation(driver,'typed');
 return fs.readFileSync(new URL('./allocations/schema-object-baseline.rs',import.meta.url),'utf8')+'\n'+replaceOnce(source,'serde_json::from_str::<Vec<i64>>(std::hint::black_box(input.as_ref())).unwrap()','serde_json::from_str::<SchemaPayload>(std::hint::black_box(input.as_ref())).unwrap().items');
}
