import fs from 'node:fs';
// Reuse the established allocation driver without weakening disposal accounting.
function replaceOnce(source,before,after){
 if(source.split(before).length!==2)throw Error(`JSON allocation ABI drift: ${before}`);
 return source.replace(before,after);
}
export const jsonAllocationKinds=['raw','raw_strings','raw_deep','raw_invalid','raw_selective','dynamic','typed','dynamic_strings','typed_strings','dynamic_nested','typed_nested','dynamic_selective','typed_selective','dynamic_invalid','typed_invalid','typed_mismatch','encode_dynamic','encode_typed','stream_owned','stream_borrowed','stream_write_owned','stream_write_borrowed','dynamic_deep','encode_dynamic_strings','encode_typed_strings','encode_dynamic_deep','encode_dynamic_object','encode_typed_object'];
function shape(kind){
 if(!jsonAllocationKinds.includes(kind))throw Error('JSON allocation kind');
 return {dynamic:kind.startsWith('dynamic'),failure:kind.endsWith('_invalid')||kind.endsWith('_mismatch'),strings:kind.endsWith('_strings')||kind.endsWith('_invalid')||kind.endsWith('_mismatch'),nested:kind.endsWith('_nested')||kind.endsWith('_selective'),selective:kind.endsWith('_selective')};
}
export function jsonAllocationInput(n,kind='dynamic'){
 const {strings,nested,selective,failure}=shape(kind);
 if(!Number.isSafeInteger(n)||n<0||n>100000)throw Error('JSON allocation size must be an integer from 0 through 100000');
 if(kind==='raw_selective')return '{"payload":'+jsonAllocationInput(n,'raw')+',"tag":42,"ignored":['+Array.from({length:n},()=> '"unused\\n\\u0061"').join(',')+']}';
 if(kind.endsWith('_object'))return '{"active":true,"items":'+jsonAllocationInput(n,'typed_nested')+',"name":"é🦀"}';
 if(kind==='raw_deep'||kind==='dynamic_deep'||kind==='encode_dynamic_deep')return '['.repeat(31)+jsonAllocationInput(n,'dynamic')+']'.repeat(31);
 if(kind.startsWith('encode_')&&strings)return '['+Array.from({length:n},()=>'"é🦀","line\\na"').join(',')+']';
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
 if(kind==='raw_strings')return n===0?2:24*n+1;
 if(kind==='raw'||kind==='raw_deep'||kind==='raw_selective'){let bytes=2;for(let i=0;i<n;i++)bytes+=(i?1:0)+String(i+7).length;return bytes+(kind==='raw_deep'?62:0);}
 return kind.startsWith('encode_')?new TextEncoder().encode(jsonAllocationInput(n,kind)).reduce((sum,byte)=>sum+byte,0):failure?1:strings?n*12:nested?2*n*7+n*(2*n-1):n*7+n*(n-1)/2;
}
export function adaptJsonAllocation(emitted,driver,kind){
 if(['raw','raw_strings','raw_deep','raw_invalid','raw_selective'].includes(kind)){
  const signature=/^fn tsr_result_function0<'(\w+)>\(value0: i64, value1: i64, value2: &'\1 String\) -> Result<(Record\d+)<'\1>,TsrFunctionError/m.exec(emitted);
  if(!signature)throw Error('JSON raw borrowing ABI drift');
  if(kind==='raw_invalid')return replaceOnce(emitted,'fn main()','fn source_main()')+'\n'+instrumentDriver(driver,'match tsr_result_function0(std::hint::black_box(n),std::hint::black_box(7),&input){Ok(_)=>0i64,Err(error)=>{drop(error);1i64}}')+'\nfn probe_checksum(value:&i64)->i64{*value}';
  let adapted=instrumentDriver(driver,'match tsr_result_function0(std::hint::black_box(n),std::hint::black_box(7),&input){Ok(value)=>value,Err(_)=>panic!("valid raw input rejected")}');
  adapted=replaceOnce(adapted,'let checksum = probe_checksum(&value);',`assert_eq!(value.fragment.text().as_ptr(),input${kind==='raw_selective'?'[11..]':''}.as_ptr());\nlet checksum = probe_checksum(&value);`);
  return replaceOnce(emitted,'fn main()','fn source_main()')+'\n'+adapted+`\nfn probe_checksum(value:&${signature[2]}<'_>)->i64{value.fragment.text().len() as i64}`;
 }

 const {dynamic,strings,nested,selective,failure}=shape(kind);
 const signature=/^fn tsr_result_function0\(value0: i64, value1: i64, value2: &String\) -> Result<(Record\d+|NumericArray<i64>|Array<String>|Array<NumericArray<i64>>),TsrFunctionError[^\n]+/m.exec(emitted);
 const expected=nested?'Array<NumericArray<i64>>':strings?'Array<String>':'NumericArray<i64>';
 if(!signature||((dynamic||selective)?!/^Record\d+$/.test(signature[1]):signature[1]!==expected))throw Error('JSON allocation return ABI drift');
 if(selective&&!dynamic&&!emitted.includes(`struct ${signature[1]} { field0: Array<NumericArray<i64>> }`))throw Error('JSON selective record ABI drift');
 const body=replaceOnce(emitted,'fn main()','fn source_main()');
 if(failure)return body+'\n'+instrumentDriver(driver,'match tsr_result_function0(std::hint::black_box(n), std::hint::black_box(7), &input) { Ok(_)=>0i64, Err(error)=>{drop(error);1i64} }')+'\nfn probe_checksum(value:&i64)->i64 {*value}';
 const checksum=kind==='dynamic_deep'
  ?'let mut leaf=&value.value;for _ in 0..31 {let rows=leaf.as_array().unwrap();assert_eq!(rows.len(),1);leaf=&rows[0];}leaf.as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).sum()'
  :nested
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
 if(kind==='raw_selective')return fs.readFileSync(new URL('./allocations/json-raw-selective-baseline.rs',import.meta.url),'utf8')+"\ntype String=std::borrow::Cow<'static,str>;\n"+instrumentDriver(driver,'{let value:SelectedRaw=serde_json::from_str(std::hint::black_box(input.as_ref())).unwrap();assert_eq!(value.tag,42);value.payload.get()}')+'\nfn probe_checksum(value:&&str)->i64{value.len() as i64}';
 if(kind==='raw_invalid')return "type String=std::borrow::Cow<'static,str>;\n"+instrumentDriver(driver,'{let mut decoder=serde_json::Deserializer::from_str(std::hint::black_box(input.as_ref()));<serde::de::IgnoredAny as serde::Deserialize>::deserialize(&mut decoder).and_then(|_|decoder.end()).is_err() as i64}')+'\nfn probe_checksum(value:&i64)->i64{*value}';
 if(['raw','raw_strings','raw_deep'].includes(kind))return "type String=std::borrow::Cow<'static,str>;\n"+instrumentDriver(driver,'{let mut decoder=serde_json::Deserializer::from_str(std::hint::black_box(input.as_ref()));<serde::de::IgnoredAny as serde::Deserialize>::deserialize(&mut decoder).unwrap();decoder.end().unwrap();input.as_ref()}')+'\nfn probe_checksum(value:&&str)->i64{value.len() as i64}';

 const {dynamic,strings,nested,selective,failure}=shape(kind);
 const ty=dynamic?'serde_json::Value':selective?'Selected':nested?'Vec<Vec<i64>>':strings?'Vec<std::string::String>':'Vec<i64>';
 const checksum=kind==='dynamic_deep'
  ?'let mut leaf=value;for _ in 0..31 {let rows=leaf.as_array().unwrap();assert_eq!(rows.len(),1);leaf=&rows[0];}leaf.as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).sum()'
  :nested
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
 if(kind==='raw_selective')return counterpart+`\nfunction probeChecksum(value:unknown):number{const end=input.indexOf(',"tag":42,');if(value!==input.slice(11,end))throw Error("raw payload changed");return new TextEncoder().encode(value as string).length;}\n`+adapted;
 if(['raw','raw_strings','raw_deep'].includes(kind))return counterpart+'\nfunction probeChecksum(value:unknown):number{if(value!==input)throw Error("raw source changed");return new TextEncoder().encode(input).length;}\n'+adapted;
 if(failure){
  const decoding=replaceOnce(counterpart,'function workload(','function decodeWorkload(');
  return decoding+'\nfunction workload(n:number,seed:number,input:string):number{try{decodeWorkload(n,seed,input);return 0;}catch{return 1;}}\nfunction probeChecksum(value:unknown):number{if(value!==1)throw Error("invalid input accepted");return 1;}\n'+adapted;
 }
 const check=strings?'if(typeof item!=="string")throw Error("string output required");return sum+new TextEncoder().encode(item).length;':'if(typeof item!=="number"||!Number.isSafeInteger(item))throw Error("integer output required");return sum+item;';
 const preparation=(kind==='dynamic_deep'?'for(let depth=0;depth<31;depth++){if(!Array.isArray(value)||value.length!==1)throw Error("nested wrapper required");value=value[0];}':'')+(selective?'if(typeof value!=="object"||value===null||!("items" in value))throw Error("items record required");value=value.items;':'')+(nested?'if(!Array.isArray(value))throw Error("nested array required");for(const row of value)if(!Array.isArray(row))throw Error("row required");value=value.flat();':'');
 return counterpart+'\nfunction probeChecksum(value:unknown):number {'+preparation+'if(!Array.isArray(value))throw Error("array output required");return value.reduce((sum:number,item:unknown)=>{'+check+'},0);}\n'+adapted;
}
