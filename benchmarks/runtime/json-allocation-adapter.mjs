// Reuse the established allocation driver without weakening disposal accounting.
function replaceOnce(source,before,after){
 if(source.split(before).length!==2)throw Error(`JSON allocation ABI drift: ${before}`);
 return source.replace(before,after);
}
export function jsonAllocationInput(n){
 if(!Number.isSafeInteger(n)||n<0||n>100000)throw Error('JSON allocation size must be an integer from 0 through 100000');
 return '['+Array.from({length:n},(_,i)=>String(i+7)).join(',')+']';
}
export function jsonAllocationChecksum(n){
 if(!Number.isSafeInteger(n)||n<0||n>100000)throw Error('JSON allocation size');
 return n*7+n*(n-1)/2;
}
export function adaptJsonAllocation(emitted,driver,kind){
 if(!['dynamic','typed'].includes(kind))throw Error('JSON allocation kind');
 const signature=/^fn tsr_result_function0\(value0: i64, value1: i64, value2: &String\) -> Result<(Record\d+|NumericArray<i64>),TsrFunctionError[^\n]+/m.exec(emitted);
 if(!signature||kind==='dynamic'&&!/^Record\d+$/.test(signature[1])||kind==='typed'&&signature[1]!=='NumericArray<i64>')throw Error('JSON allocation return ABI drift');
 const body=replaceOnce(emitted,'fn main()','fn source_main()');
 return body+'\n'+instrumentDriver(driver,'match tsr_result_function0(std::hint::black_box(n), std::hint::black_box(7), &input) { Ok(value)=>value, Err(_)=>panic!("valid JSON allocation input rejected") }')+'\n'+(kind==='dynamic'
  ?`fn probe_checksum(value:&${signature[1]})->i64 {value.value.as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).sum()}`
  :'fn probe_checksum(value:&NumericArray<i64>)->i64 {value.iter().sum()}');
}
function instrumentDriver(driver,call){
 let output=replaceOnce(driver,'fn main()','fn main()');
 output=replaceOnce(output,'let n: i64 = arguments[1].parse().unwrap();','let input = String::from(arguments[2].clone());\n        let n: i64 = arguments[1].parse().unwrap();');
 return replaceOnce(output,'function0(std::hint::black_box(n), std::hint::black_box(7))',call);
}
export function directJsonAllocation(driver,kind){
 if(!['dynamic','typed'].includes(kind))throw Error('JSON allocation kind');
 const ty=kind==='dynamic'?'serde_json::Value':'Vec<i64>';
 const checksum=kind==='dynamic'?'value.as_array().unwrap().iter().map(|v|v.as_i64().unwrap()).sum()':'value.iter().sum()';
 return 'type String=std::borrow::Cow<\'static,str>;\n'+instrumentDriver(driver,`serde_json::from_str::<${ty}>(std::hint::black_box(input.as_ref())).unwrap()`)+`\nfn probe_checksum(value:&${ty})->i64 {${checksum}}`;
}
export function nodeJsonAllocation(counterpart,driver){
 let adapted=replaceOnce(driver,'const n = Number(process.argv[2]);','const n = Number(process.argv[2]);\nconst input=process.argv[3]!;');
 adapted=replaceOnce(adapted,'workload(0, 7)','workload(0, 7, input)');
 adapted=replaceOnce(adapted,'workload(n, 7)','workload(n, 7, input)');
 return counterpart+'\nfunction probeChecksum(value:unknown):number {if(!Array.isArray(value))throw Error("array output required");return value.reduce((sum:number,item:unknown)=>{if(typeof item!=="number"||!Number.isSafeInteger(item))throw Error("integer output required");return sum+item;},0);}\n'+adapted;
}
