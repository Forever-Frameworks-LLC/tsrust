import fs from 'node:fs';
function replaceOnce(source,before,after){
 if(source.split(before).length!==2)throw Error(`JSON stream ABI drift: ${before}`);
 return source.replace(before,after);
}
export function streamInput(n){
 if(!Number.isSafeInteger(n)||n<0||n>100000)throw Error('JSON stream size');
 return '{"name":"hello","values":[20,17],"active":true}\n'.repeat(n);
}
function instrument(driver,call){
 let text=replaceOnce(driver,'let n: i64 = arguments[1].parse().unwrap();','let input=String::from(arguments[2].clone());\nlet n: i64 = arguments[1].parse().unwrap();');
 return replaceOnce(text,'function0(std::hint::black_box(n), std::hint::black_box(7))',call)+'\nfn probe_checksum(value:&i64)->i64 {*value}';
}
export function nativeStreamAllocation(emitted,driver,writer=false){
 if(!/^fn tsr_result_function0\(value0: i64, value1: i64, value2: &String\) -> Result<i64,TsrFunctionError/m.test(emitted))throw Error('JSON stream return ABI drift');
 return replaceOnce(emitted,'fn main()','fn source_main()')+'\n'+instrument(driver,`match tsr_result_function0(${writer?'n':'1'},7,&input){Ok(value)=>value,Err(_)=>panic!("valid JSONL rejected")}`);
}
export function directStreamAllocation(driver,writer=false){
 if(writer)return fs.readFileSync(new URL('./allocations/json-writer-baseline.rs',import.meta.url),'utf8')+'\n'+instrument(driver,'stream_write(n,std::hint::black_box(&input))');
 return fs.readFileSync(new URL('./allocations/json-stream-baseline.rs',import.meta.url),'utf8')+'\n'+instrument(driver,'stream(std::hint::black_box(&input))');
}
export function nodeStreamAllocation(counterpart,driver,writer=false){
 let text=replaceOnce(driver,'const n = Number(process.argv[2]);','const n = Number(process.argv[2]);\nconst input=process.argv[3]!;');
 text=replaceOnce(text,'workload(0, 7)','workload(0, 7, input)');
 text=replaceOnce(text,'workload(n, 7)',writer?'workload(n, 7, input)':'workload(1, 7, input)');
 return counterpart+'\nfunction probeChecksum(value:unknown):number{if(typeof value!=="number")throw Error("numeric checksum required");return value;}\n'+text;
}
