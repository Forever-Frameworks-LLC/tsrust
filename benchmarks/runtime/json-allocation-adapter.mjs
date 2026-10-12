// Reuse the established allocation driver without weakening disposal accounting.
function replaceOnce(source,before,after){
 if(source.split(before).length!==2)throw Error(`JSON allocation ABI drift: ${before}`);
 return source.replace(before,after);
}
export const jsonAllocationKinds=['dynamic','typed','dynamic_strings','typed_strings'];
function shape(kind){
 if(!jsonAllocationKinds.includes(kind))throw Error('JSON allocation kind');
 return {dynamic:kind.startsWith('dynamic'),strings:kind.endsWith('_strings')};
}
export function jsonAllocationInput(n,kind='dynamic'){
 const {strings}=shape(kind);
 if(!Number.isSafeInteger(n)||n<0||n>100000)throw Error('JSON allocation size must be an integer from 0 through 100000');
 return '['+Array.from({length:n},(_,i)=>strings?'"é🦀","line\\n\\u0061"':String(i+7)).join(',')+']';
}
export function jsonAllocationChecksum(n,kind='dynamic'){
 const {strings}=shape(kind);
 if(!Number.isSafeInteger(n)||n<0||n>100000)throw Error('JSON allocation size');
 return strings?n*12:n*7+n*(n-1)/2;
}
export function adaptJsonAllocation(emitted,driver,kind){
 const {dynamic,strings}=shape(kind);
 const signature=/^fn tsr_result_function0\(value0: i64, value1: i64, value2: &String\) -> Result<(Record\d+|NumericArray<i64>|Array<String>),TsrFunctionError[^\n]+/m.exec(emitted);
 const expected=strings?'Array<String>':'NumericArray<i64>';
 if(!signature||(dynamic?!/^Record\d+$/.test(signature[1]):signature[1]!==expected))throw Error('JSON allocation return ABI drift');
 const body=replaceOnce(emitted,'fn main()','fn source_main()');
 const checksum=dynamic
  ?`value.value.as_array().unwrap().iter().map(|v|${strings?'v.as_str().unwrap().len() as i64':'v.as_i64().unwrap()'}).sum()`
  :strings?'value.iter().map(|v|v.len() as i64).sum()':'value.iter().sum()';
 return body+'\n'+instrumentDriver(driver,'match tsr_result_function0(std::hint::black_box(n), std::hint::black_box(7), &input) { Ok(value)=>value, Err(_)=>panic!("valid JSON allocation input rejected") }')+`\nfn probe_checksum(value:&${signature[1]})->i64 {${checksum}}`;
}
function instrumentDriver(driver,call){
 let output=replaceOnce(driver,'fn main()','fn main()');
 output=replaceOnce(output,'let n: i64 = arguments[1].parse().unwrap();','let input = String::from(arguments[2].clone());\n        let n: i64 = arguments[1].parse().unwrap();');
 return replaceOnce(output,'function0(std::hint::black_box(n), std::hint::black_box(7))',call);
}
export function directJsonAllocation(driver,kind){
 const {dynamic,strings}=shape(kind);
 const ty=dynamic?'serde_json::Value':strings?'Vec<std::string::String>':'Vec<i64>';
 const checksum=dynamic
  ?`value.as_array().unwrap().iter().map(|v|${strings?'v.as_str().unwrap().len() as i64':'v.as_i64().unwrap()'}).sum()`
  :strings?'value.iter().map(|v|v.len() as i64).sum()':'value.iter().sum()';
 return 'type String=std::borrow::Cow<\'static,str>;\n'+instrumentDriver(driver,`serde_json::from_str::<${ty}>(std::hint::black_box(input.as_ref())).unwrap()`)+`\nfn probe_checksum(value:&${ty})->i64 {${checksum}}`;
}
export function nodeJsonAllocation(counterpart,driver,kind='dynamic'){
 const {strings}=shape(kind);
 let adapted=replaceOnce(driver,'const n = Number(process.argv[2]);','const n = Number(process.argv[2]);\nconst input=process.argv[3]!;');
 adapted=replaceOnce(adapted,'workload(0, 7)','workload(0, 7, input)');
 adapted=replaceOnce(adapted,'workload(n, 7)','workload(n, 7, input)');
 const check=strings?'if(typeof item!=="string")throw Error("string output required");return sum+new TextEncoder().encode(item).length;':'if(typeof item!=="number"||!Number.isSafeInteger(item))throw Error("integer output required");return sum+item;';
 return counterpart+'\nfunction probeChecksum(value:unknown):number {if(!Array.isArray(value))throw Error("array output required");return value.reduce((sum:number,item:unknown)=>{'+check+'},0);}\n'+adapted;
}
