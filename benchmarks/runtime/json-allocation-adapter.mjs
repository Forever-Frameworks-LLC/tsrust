import fs from 'node:fs';
// Reuse the established allocation driver without weakening disposal accounting.
function replaceOnce(source,before,after){
 if(source.split(before).length!==2)throw Error(`JSON allocation ABI drift: ${before}`);
 return source.replace(before,after);
}
export const jsonAllocationKinds=['dynamic','typed','dynamic_strings','typed_strings','dynamic_nested','typed_nested','dynamic_selective','typed_selective','dynamic_invalid','typed_invalid','typed_mismatch','encode_dynamic','encode_typed','stream_owned','stream_borrowed'];
function shape(kind){
 if(!jsonAllocationKinds.includes(kind))throw Error('JSON allocation kind');
 return {dynamic:kind.startsWith('dynamic'),failure:kind.endsWith('_invalid')||kind.endsWith('_mismatch'),strings:kind.endsWith('_strings')||kind.endsWith('_invalid')||kind.endsWith('_mismatch'),nested:kind.endsWith('_nested')||kind.endsWith('_selective'),selective:kind.endsWith('_selective')};
}
export function jsonAllocationInput(n,kind='dynamic'){
 const {strings,nested,selective,failure}=shape(kind);
 if(!Number.isSafeInteger(n)||n<0||n>100000)throw Error('JSON allocation size must be an integer from 0 through 100000');
 if(failure)return '['+Array.from({length:n},()=>'"é🦀"').concat(kind.endsWith('_mismatch')?'7':'?').join(',')+']';
 if(nested){
  const items='['+Array.from({length:n},(_,i)=>`[${7+2*i},${8+2*i}]`).join(',')+']';
  return selective?'{"items":'+items+',"ignored":['+Array.from({length:n},()=>'{"text":"unused\\n\\u0061","nested":[true,null,{}]}').join(',')+']}':items;
 }
 return '['+Array.from({length:n},(_,i)=>strings?'"é🦀","line\\n\\u0061"':String(i+7)).join(',')+']';
}
export function jsonAllocationChecksum(n,kind='dynamic'){
 const {strings,nested,selective,failure}=shape(kind);
 if(!Number.isSafeInteger(n)||n<0||n>100000)throw Error('JSON allocation size');
 return kind.startsWith('encode_')?[...jsonAllocationInput(n,kind)].reduce((sum,char)=>sum+char.charCodeAt(0),0):failure?1:strings?n*12:nested?2*n*7+n*(2*n-1):n*7+n*(n-1)/2;
}
export function adaptJsonAllocation(emitted,driver,kind){
 const {dynamic,strings,nested,selective,failure}=shape(kind);
 const signature=/^fn tsr_result_function0\(value0: i64, value1: i64, value2: &String\) -> Result<(Record\d+|NumericArray<i64>|Array<String>|Array<NumericArray<i64>>),TsrFunctionError[^\n]+/m.exec(emitted);
 const expected=nested?'Array<NumericArray<i64>>':strings?'Array<String>':'NumericArray<i64>';
 if(!signature||((dynamic||selective)?!/^Record\d+$/.test(signature[1]):signature[1]!==expected))throw Error('JSON allocation return ABI drift');
 if(selective&&!dynamic&&!emitted.includes(`struct ${signature[1]} { field0: Array<NumericArray<i64>> }`))throw Error('JSON selective record ABI drift');
 const body=replaceOnce(emitted,'fn main()','fn source_main()');
 if(failure)return body+'\n'+instrumentDriver(driver,'match tsr_result_function0(std::hint::black_box(n), std::hint::black_box(7), &input) { Ok(_)=>0i64, Err(error)=>{drop(error);1i64} }')+'\nfn probe_checksum(value:&i64)->i64 {*value}';
 const checksum=nested
  ?dynamic?`value.value${selective?'["items"]':''}.as_array().unwrap().iter().flat_map(|row|row.as_array().unwrap()).map(|v|v.as_i64().unwrap()).sum()`
   :`value${selective?'.field0':''}.iter().flat_map(|row|row.iter()).sum()`
  :dynamic
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
 const {dynamic,strings,nested,selective,failure}=shape(kind);
 const ty=dynamic?'serde_json::Value':selective?'Selected':nested?'Vec<Vec<i64>>':strings?'Vec<std::string::String>':'Vec<i64>';
 const checksum=nested
  ?dynamic?`value${selective?'["items"]':''}.as_array().unwrap().iter().flat_map(|row|row.as_array().unwrap()).map(|v|v.as_i64().unwrap()).sum()`
   :`value${selective?'.items':''}.iter().flat_map(|row|row.iter()).sum()`
  :dynamic
  ?`value.as_array().unwrap().iter().map(|v|${strings?'v.as_str().unwrap().len() as i64':'v.as_i64().unwrap()'}).sum()`
  :strings?'value.iter().map(|v|v.len() as i64).sum()':'value.iter().sum()';
 if(failure)return 'type String=std::borrow::Cow<\'static,str>;\n'+instrumentDriver(driver,`serde_json::from_str::<${ty}>(std::hint::black_box(input.as_ref())).is_err() as i64`)+ '\nfn probe_checksum(value:&i64)->i64 {*value}';
 const prelude=selective&&!dynamic?fs.readFileSync(new URL('./allocations/json-selective-baseline.rs',import.meta.url),'utf8'):'';
 return prelude+'\n'+'type String=std::borrow::Cow<\'static,str>;\n'+instrumentDriver(driver,`serde_json::from_str::<${ty}>(std::hint::black_box(input.as_ref())).unwrap()`)+`\nfn probe_checksum(value:&${ty})->i64 {${checksum}}`;
}
export function nodeJsonAllocation(counterpart,driver,kind='dynamic'){
 const {strings,nested,selective,failure}=shape(kind);
 let adapted=replaceOnce(driver,'const n = Number(process.argv[2]);','const n = Number(process.argv[2]);\nconst input=process.argv[3]!;');
 adapted=replaceOnce(adapted,'workload(0, 7)','workload(0, 7, input)');
 adapted=replaceOnce(adapted,'workload(n, 7)','workload(n, 7, input)');
 if(failure){
  const decoding=replaceOnce(counterpart,'function workload(','function decodeWorkload(');
  return decoding+'\nfunction workload(n:number,seed:number,input:string):number{try{decodeWorkload(n,seed,input);return 0;}catch{return 1;}}\nfunction probeChecksum(value:unknown):number{if(value!==1)throw Error("invalid input accepted");return 1;}\n'+adapted;
 }
 const check=strings?'if(typeof item!=="string")throw Error("string output required");return sum+new TextEncoder().encode(item).length;':'if(typeof item!=="number"||!Number.isSafeInteger(item))throw Error("integer output required");return sum+item;';
 const preparation=(selective?'if(typeof value!=="object"||value===null||!("items" in value))throw Error("items record required");value=value.items;':'')+(nested?'if(!Array.isArray(value))throw Error("nested array required");for(const row of value)if(!Array.isArray(row))throw Error("row required");value=value.flat();':'');
 return counterpart+'\nfunction probeChecksum(value:unknown):number {'+preparation+'if(!Array.isArray(value))throw Error("array output required");return value.reduce((sum:number,item:unknown)=>{'+check+'},0);}\n'+adapted;
}
