import fs from 'node:fs';
// Encoding probes construct owned inputs before allocation counters reset.
function replaceOnce(source,before,after){
 if(source.split(before).length!==2)throw Error(`JSON encoding ABI drift: ${before}`);
 return source.replace(before,after);
}
function instrument(driver,setup,call){
 let text=replaceOnce(driver,'let n: i64 = arguments[1].parse().unwrap();',setup+'\nlet n: i64 = arguments[1].parse().unwrap();');
 text=replaceOnce(text,'function0(std::hint::black_box(n), std::hint::black_box(7))',call);
 return replaceOnce(text,'let checksum = probe_checksum(&value);','assert_eq!(&*value, arguments[2].as_str());\nlet checksum = probe_checksum(&value);');
}
export function nativeEncodingAllocation(emitted,driver,kind){
 if(['encode_raw','encode_raw_strings','encode_raw_deep'].includes(kind)){
  const signature=/^fn tsr_result_function0<'native_input2_0>\(value0: i64, value1: i64, value2: &(Record\d+)<'native_input2_0>\) -> Result<String,TsrFunctionError[^\n]+/m.exec(emitted);
  if(!signature||!emitted.includes(`struct ${signature[1]}<'a>{fragment:tsrust_native_json::raw::Fragment<'a>}`))throw Error('JSON raw encoding ABI drift');
  const setup=`let input=${signature[1]}{fragment:tsrust_native_json::raw::borrow(&arguments[2],tsrust_native_json::Limits{bytes:16777216,depth:64,entries:1000000,nodes:1000000}).unwrap()};`;
  return replaceOnce(emitted,'fn main()','fn source_main()')+'\n'+instrument(driver,setup,'match tsr_result_function0(std::hint::black_box(n),std::hint::black_box(7),&input){Ok(value)=>value,Err(_)=>panic!("valid raw encoding rejected")}')+'\nfn probe_checksum(value:&String)->i64{value.bytes().map(i64::from).sum()}';
 }

 const signature=/^fn tsr_result_function0\(value0: i64, value1: i64, value2: &(NumericArray<i64>|Array<String>|Record\d+)\) -> Result<String,TsrFunctionError[^\n]+/m.exec(emitted);
 if(!signature||(kind==='encode_typed'?signature[1]!=='NumericArray<i64>':kind==='encode_typed_strings'?signature[1]!=='Array<String>':!['encode_dynamic','encode_dynamic_strings','encode_dynamic_deep','encode_dynamic_object','encode_typed_object'].includes(kind)||!/^Record\d+$/.test(signature[1])))throw Error('JSON encoding return ABI drift');
 if(kind==='encode_typed_object'&&!emitted.includes(`struct ${signature[1]} { field0: bool, field1: Array<NumericArray<i64>>, field2: String }`))throw Error('JSON object encoding layout drift');
 const setup=kind==='encode_typed_object'
  ?`let raw:serde_json::Value=serde_json::from_str(&arguments[2]).unwrap();let mut items=Array::from([]);for row in raw["items"].as_array().unwrap(){let mut values=NumericArray::from([]);for v in row.as_array().unwrap(){values.push(v.as_i64().unwrap());}items.push(values);}let input=${signature[1]} {field0:raw["active"].as_bool().unwrap(),field1:items,field2:String::from(raw["name"].as_str().unwrap().to_owned())};drop(raw);`
  :kind==='encode_typed'
  ?'let mut input=NumericArray::from([]); for value in serde_json::from_str::<Vec<i64>>(&arguments[2]).unwrap(){input.push(value);}'
  :kind==='encode_typed_strings'?'let mut input=Array::from([]);for value in serde_json::from_str::<Vec<std::string::String>>(&arguments[2]).unwrap(){input.push(String::from(value));}'
  :`let input=${signature[1]} {value:serde_json::from_str(&arguments[2]).unwrap()};`;
 return replaceOnce(emitted,'fn main()','fn source_main()')+'\n'+instrument(driver,setup,'match tsr_result_function0(std::hint::black_box(n),std::hint::black_box(7),&input){Ok(value)=>value,Err(_)=>panic!("valid encoding input rejected")}')+'\nfn probe_checksum(value:&String)->i64 {value.bytes().map(i64::from).sum()}';
}
export function directEncodingAllocation(driver,kind){
 if(['encode_raw','encode_raw_strings','encode_raw_deep'].includes(kind))return instrument(driver,'let input: &serde_json::value::RawValue=serde_json::from_str(&arguments[2]).unwrap();','serde_json::to_string(std::hint::black_box(input)).unwrap()')+'\nfn probe_checksum(value:&String)->i64{value.bytes().map(i64::from).sum()}';

 if(!['encode_typed','encode_dynamic','encode_typed_strings','encode_dynamic_strings','encode_dynamic_deep','encode_dynamic_object','encode_typed_object'].includes(kind))throw Error('JSON encoding kind');
 if(kind==='encode_typed_object')return fs.readFileSync(new URL('./allocations/json-object-encoding-baseline.rs',import.meta.url),'utf8')+'\n'+instrument(driver,'let input=prepare_object(&arguments[2]);','serde_json::to_string(std::hint::black_box(&input)).unwrap()')+'\nfn probe_checksum(value:&String)->i64 {value.bytes().map(i64::from).sum()}';
 const ty=kind==='encode_typed'?'Vec<i64>':kind==='encode_typed_strings'?'Vec<String>':'serde_json::Value';
 return instrument(driver,`let input=serde_json::from_str::<${ty}>(&arguments[2]).unwrap();`,'serde_json::to_string(std::hint::black_box(&input)).unwrap()')+'\nfn probe_checksum(value:&String)->i64 {value.bytes().map(i64::from).sum()}';
}
export function nodeEncodingAllocation(counterpart,driver,kind){
 if(['encode_raw','encode_raw_strings','encode_raw_deep'].includes(kind)){
  let adapted=replaceOnce(driver,'const n = Number(process.argv[2]);','const n = Number(process.argv[2]);\nconst inputText=process.argv[3]!;JSON.parse(inputText);const input=inputText;');
  adapted=replaceOnce(adapted,'workload(0, 7)','workload(0, 7, input)');
  adapted=replaceOnce(adapted,'workload(n, 7)','workload(n, 7, input)');
  return counterpart+'\nfunction probeChecksum(value:unknown):number{if(typeof value!=="string"||value!==inputText)throw Error("raw encoding mismatch");return new TextEncoder().encode(value).reduce((sum,byte)=>sum+byte,0); }\n'+adapted;
 }

 let adapted=replaceOnce(driver,'const n = Number(process.argv[2]);','const n = Number(process.argv[2]);\nconst inputText=process.argv[3]!;\nconst input=JSON.parse(inputText);');
 adapted=replaceOnce(adapted,'workload(0, 7)','workload(0, 7, input)');
 adapted=replaceOnce(adapted,'workload(n, 7)','workload(n, 7, input)');
 return counterpart+'\nfunction probeChecksum(value:unknown):number{if(typeof value!=="string"||value!==inputText)throw Error("encoding mismatch");return new TextEncoder().encode(value).reduce((sum,byte)=>sum+byte,0);}\n'+adapted;
}
