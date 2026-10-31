import {moduleStateABI,moduleStateDriver} from './module-state-contract.mjs';
import {isCopying,assertCopyingPolicy,validateCopyingObservation} from './copying-array-observation.mjs';
import {verifyCopyingResultABI,pruneCopyingContainers,copyingRecordObserver} from './copying-array-result-abi.mjs';
import {isRange,assertRangePolicy,validateRangeObservation} from './range-mutation-observation.mjs';
import {verifyRangeResultABI,pruneRangeContainers,rangeRecordObserver} from './range-mutation-result-abi.mjs';
import {isHigher,assertHigherPolicy} from './higher-order-observation.mjs';
import {verifyHigherResultABI,pruneHigherContainers,higherRecordObserver} from './higher-order-result-abi.mjs';
import {isDeclared,assertDeclaredPolicy,validateDeclaredObservation} from './declared-value-observation.mjs';
import {verifyDeclaredABI,verifyDeclaredReferenceABI,pruneObserverContainers as pruneDeclaredContainers,recordObserver as declaredRecordObserver} from './declared-value-abi.mjs';
import {isConcat,assertConcatPolicy} from './concat-observation.mjs';
import {verifyConcatABI,concatRustObserver} from './concat-abi.mjs';
import {isApproximate,approximateAdapterSources} from './approximate-harness.mjs';
import {isReduction,isReductionSchema} from './reduction-observation.mjs';
import {verifyReductionABI,pruneObserverContainers,recordObserver} from './reduction-abi.mjs';
import {isSliceResult} from './result-observation.mjs';
import fs from 'node:fs';
import {isNumberResult} from './number-observation.mjs';
const numberSupport = Object.fromEntries(['number-driver.rs','number-driver.ts','number-observation.rs','number-observation.ts'].map(name=>[name,fs.readFileSync(new URL(name,import.meta.url),'utf8')]));
export const numberAdapterSources = numberSupport;
const sliceSupport = Object.fromEntries(['slice-driver.rs','slice-driver.ts','slice-observation.rs','slice-observation.ts'].map(name=>[name,fs.readFileSync(new URL(name,import.meta.url),'utf8')]));
export const sliceAdapterSources = sliceSupport;
const reductionSupport = Object.fromEntries(['reduction-driver.rs','reduction-driver.ts','reduction-observation.rs','reduction-observation.ts','reduction-observation.mjs','reduction-abi.mjs'].map(name=>[name,fs.readFileSync(new URL(name,import.meta.url),'utf8')]));
export const reductionAdapterSources = reductionSupport;
function reductionRustObserver(workload,emitted,source){const binding=verifyReductionABI(emitted,workload,source);return pruneObserverContainers(reductionSupport['reduction-observation.rs'],binding,'ReductionTree')+'\n'+recordObserver(binding);}
const concatSupport=Object.fromEntries(['concat-observation.rs','concat-observation.ts','concat-driver.ts','concat-observation.mjs','concat-abi.mjs','concat-observer-policy.json'].map(name=>[name,fs.readFileSync(new URL(name,import.meta.url),'utf8')]));
export const concatAdapterSources=concatSupport;
function concatNativeObserver(workload,emitted,source){return concatRustObserver(concatSupport['concat-observation.rs'],verifyConcatABI(emitted,workload,source));}
const declaredSupport=Object.fromEntries(['declared-value-observation.rs','declared-value-observation.ts','declared-value-observation.mjs','declared-value-abi.mjs','declared-value-boolean-driver.rs','declared-value-driver.ts','declared-value-observer-policy.json'].map(name=>[name,fs.readFileSync(new URL(name,import.meta.url),'utf8')]));
export const declaredAdapterSources=declaredSupport;
function declaredNativeObserver(w,emitted,source,reference=false){const binding=(reference?verifyDeclaredReferenceABI:verifyDeclaredABI)(emitted,w,source);return pruneDeclaredContainers(declaredSupport['declared-value-observation.rs'],binding)+'\n'+declaredRecordObserver(binding);}
const higherSupport=Object.fromEntries(['higher-order-observation.mjs','higher-order-result-abi.mjs','higher-order-observer-policy.json','declared-value-observation.rs','declared-value-observation.ts','declared-value-boolean-driver.rs','declared-value-driver.ts'].map(name=>[name,fs.readFileSync(new URL(name,import.meta.url),'utf8')]));
export const higherAdapterSources=higherSupport;
function higherNativeObserver(w,emitted,source){const binding=verifyHigherResultABI(emitted,w,source);return pruneHigherContainers(declaredSupport['declared-value-observation.rs'],binding)+'\n'+higherRecordObserver(binding);}
const rangeSupport=Object.fromEntries(['range-mutation-observation.mjs','range-mutation-result-abi.mjs','range-mutation-observer-policy.json','range-mutation-observation.ts','concat-observation.rs','concat-driver.ts'].map(name=>[name,fs.readFileSync(new URL(name,import.meta.url),'utf8')]));
export const rangeAdapterSources=rangeSupport;
function rangeNativeObserver(w,emitted,source){const binding=verifyRangeResultABI(emitted,w,source);return pruneRangeContainers(concatSupport['concat-observation.rs'],binding)+'\n'+rangeRecordObserver(binding);}
const copyingSupport=Object.fromEntries(['copying-array-observation.mjs','copying-array-result-abi.mjs','copying-array-observer-policy.json','copying-array-observation.ts','concat-observation.rs','concat-driver.ts'].map(name=>[name,fs.readFileSync(new URL(name,import.meta.url),'utf8')]));
export const copyingAdapterSources=copyingSupport;
function copyingNativeObserver(w,emitted,source){const binding=verifyCopyingResultABI(emitted,w,source);return pruneCopyingContainers(concatSupport['concat-observation.rs'],binding)+'\n'+copyingRecordObserver(binding);}
// Argument/result hooks only; authored workload bodies are never rewritten.
export const rustResults = {'boolean[][]': 'Array<NumericArray<bool>>', 'number': 'f64', 'number[]': 'Vec<f64>', 'boolean[]': 'NumericArray<bool>', 'number[][]': 'Vec<Vec<f64>>', 'string[][]': 'Vec<Array<String>>', 'int[][][]': 'Vec<Array<NumericArray<i64>>>', 'int': 'i64', 'string[]': 'Array<String>', 'int[]': 'NumericArray<i64>', 'int[][]': 'Array<NumericArray<i64>>'};
export function assertABI(emitted, workload, {source} = {}) {
 if(workload.family==='module-state')return moduleStateABI(emitted,workload,source);
 if(isCopying(workload))return verifyCopyingResultABI(emitted,workload,source);
 if(isRange(workload))return verifyRangeResultABI(emitted,workload,source);
 if(isHigher(workload))return verifyHigherResultABI(emitted,workload,source);
 if(isDeclared(workload))return verifyDeclaredABI(emitted,workload,source);
 if(isConcat(workload))return verifyConcatABI(emitted,workload,source);
 if(isReduction(workload))return verifyReductionABI(emitted,workload,source);
 return ordinaryWorkloadBinding(emitted, workload, source);
}
// The workload may follow specialized helper functions in emitted Rust.
// Match its complete native signature; ambiguous candidates cannot be selected safely.
function ordinaryWorkloadBinding(emitted, workload, source) {
 const result = rustResults[workload.result];
 const parameters = `value0: i64, value1: i64${workload.inputText === undefined ? '' : ', value2: &String'}${workload.scalarCoordinates ? `, value${workload.inputText === undefined ? 2 : 3}: i64` : ''}`;
 const matches = [...emitted.matchAll(/\bfn ((?:tsr_result_)?function\d+)\(([^)\n]*)\) -> ([^\n{]+)\s*\{/g)]
  .filter(match => match[2] === parameters && (match[3].trim() === result || (['native-schema','native-json','environment','native-files','native-byte-views','native-byte-clone','result-match','caller-borrowed-record-stores','scoped-record-composition','path-relative','generic-callable-replacement','generic-value-adapters','generic-function-contexts','nested-function-rest','nested-function-defaults','nested-function-results','nested-function-owned-escape','fallible-record-callbacks','exception-completion','array-callback-errors','string-indexing'].includes(workload.family) && match[3].trim().startsWith(`Result<${result},`) && /^Result<.*TsrFunctionError\d+.*>$/.test(match[3].trim()))));
 // Nongeneric workload-first sources have a stable declaration index. A
 // same-signature helper must not make their actual workload ambiguous.
 const firstDeclaration = typeof source === 'string' ? /\bfunction\s+(\w+)\s*([<(])/.exec(source) : null;
 const declaredFirst = firstDeclaration?.[1] === 'workload' && firstDeclaration[2] === '(';
 const candidates = matches.length > 1 && declaredFirst ? matches.filter(match => match[1] === 'function0') : matches;
 if (!result || candidates.length !== 1 || !emitted.includes('fn main()')) throw new Error(`Emitter ABI changed for ${workload.id}: expected one complete workload signature`);
 const functionName = candidates[0][1];
 return {functionName, parameters, result, fallible:candidates[0][3].trim()!==result, signature: `fn ${functionName}(${parameters}) -> ${candidates[0][3].trim()}`};
}
function sliceRustObserver(workload, {emitted,source} = {}) {
 if(isReduction(workload))return pruneObserverContainers(sliceSupport['slice-observation.rs'],verifyReductionABI(emitted,workload,source),'SliceTree');
 let observer = sliceSupport['slice-observation.rs'];
 // Emission includes only containers actually used by the program.
 const containers = [
  [/\bArray</, 'impl<T:SliceTree> SliceTree for Array<T> {'],
  [/\bNumericArray</, 'impl<T:SliceTree+Copy+Default> SliceTree for NumericArray<T> {']
 ];
 for (const [needed, header] of containers) {
  if (needed.test(rustResults[workload.result])) continue;
  const start = observer.indexOf(header), end = observer.indexOf('\n}\n', start);
  if (start < 0 || end < start || observer.indexOf(header, start + 1) >= 0) throw new Error('Rank observer support changed');
  observer = observer.slice(0, start) + observer.slice(end + 3);
 }
 return observer;
}
export function rustChecksum(workload, {probe = false,emitted,source,reference=false} = {}) {
 if(isCopying(workload))return copyingNativeObserver(workload,emitted,source);
 if(isRange(workload))return rangeNativeObserver(workload,emitted,source);
 if(isHigher(workload)){verifyHigherResultABI(emitted,workload,source);if(workload.result==='boolean')return 'fn '+(probe?'probe_checksum':'benchmark_checksum')+'(value: '+(probe?'&bool':'bool')+') -> i64 { i64::from('+(probe?'*value':'value')+') }';if(workload.result.endsWith('[]'))return higherNativeObserver(workload,emitted,source);}
 if(isDeclared(workload)){(reference?verifyDeclaredReferenceABI:verifyDeclaredABI)(emitted,workload,source);if(workload.result==='boolean')return 'fn '+(probe?'probe_checksum':'benchmark_checksum')+'(value: '+(probe?'&bool':'bool')+') -> i64 { i64::from('+(probe?'*value':'value')+') }';if(workload.result.endsWith('[]'))return declaredNativeObserver(workload,emitted,source,reference);}
 if(isConcat(workload)){verifyConcatABI(emitted,workload,source);if(workload.result!=='int')return concatNativeObserver(workload,emitted,source);}
 if(isApproximate(workload))throw Error('Approximate protocol has no checksum');
 if(isReductionSchema(workload))return reductionRustObserver(workload,emitted,source);
 if (isSliceResult(workload)) return sliceRustObserver(workload,{emitted,source});
 if (isNumberResult(workload)) return numberSupport['number-observation.rs'];
 const type = rustResults[workload.result], name = probe ? 'probe_checksum' : 'benchmark_checksum', parameter = probe ? `&${type}` : type;
 if (workload.result === 'int') return `fn ${name}(value: ${parameter}) -> i64 { ${probe ? '*value' : 'value'} }`;
 let body;
 if (workload.result === 'string[]') body = 'for value in values.iter() { for scalar in value.chars() { result = (result * 131 + i64::from(u32::from(scalar))) % 1_000_000_007; } result = (result * 131 + 255) % 1_000_000_007; }';
 if (workload.result === 'int[]') body = 'for value in values.iter() { result = (result * 131 + *value) % 1_000_000_007; }';
 if (workload.result === 'int[][]') body = 'for row in values.iter() { for value in row.iter() { result = (result * 131 + *value) % 1_000_000_007; } result = (result * 131 + 255) % 1_000_000_007; }';
 return `fn ${name}(values: ${parameter}) -> i64 { let mut result = 0_i64; ${body} result }`;
}
export function adaptRustDriver(driver, workload, {probe = false,emitted,source,reference=false} = {}) {
 if(workload.family==='module-state')return moduleStateDriver(driver,workload,emitted,source);
 if(isCopying(workload))verifyCopyingResultABI(emitted,workload,source);
 if(isRange(workload))verifyRangeResultABI(emitted,workload,source);
 if(isHigher(workload))verifyHigherResultABI(emitted,workload,source);
 if(isDeclared(workload))(reference?verifyDeclaredReferenceABI:verifyDeclaredABI)(emitted,workload,source);
 if(isConcat(workload))verifyConcatABI(emitted,workload,source);
 if(isReduction(workload))verifyReductionABI(emitted,workload,source);
 if(isApproximate(workload)){if(probe)throw Error('Approximate profiles require distinct untimed wrapper');return approximateAdapterSources['approximate-driver.rs'];}
 if(!probe&&isCopying(workload))driver=copyingNativeObserver(workload,emitted,source)+'\n'+sliceSupport['slice-driver.rs'].replace('__FLOATING__',String(workload.family==='number'));
 else if(!probe&&isRange(workload))driver=rangeNativeObserver(workload,emitted,source)+'\n'+sliceSupport['slice-driver.rs'].replace('__FLOATING__',String(workload.family==='number'));
 else if(!probe&&isHigher(workload)&&workload.result==='boolean')driver=declaredSupport['declared-value-boolean-driver.rs'];
 else if(!probe&&isHigher(workload)&&workload.result.endsWith('[]'))driver=higherNativeObserver(workload,emitted,source)+'\n'+sliceSupport['slice-driver.rs'].replace('__FLOATING__','false');
 else if(!probe&&isDeclared(workload)&&workload.result==='boolean')driver=declaredSupport['declared-value-boolean-driver.rs'];
 else if(!probe&&isDeclared(workload)&&workload.result.endsWith('[]'))driver=declaredNativeObserver(workload,emitted,source,reference)+'\n'+sliceSupport['slice-driver.rs'].replace('__FLOATING__','false');
 else if(!probe&&isConcat(workload)&&workload.result!=='int')driver=concatNativeObserver(workload,emitted,source)+'\n'+sliceSupport['slice-driver.rs'].replace('__FLOATING__',String(workload.resultEncoding==='ieee-f64-tree-v1'));
 else if(!probe&&isReductionSchema(workload))driver=reductionRustObserver(workload,emitted,source)+'\n'+reductionSupport['reduction-driver.rs'];
 else if (!probe && isSliceResult(workload)) driver = sliceRustObserver(workload,{emitted,source}) + '\n' + sliceSupport['slice-driver.rs'].replace('__FLOATING__', String(workload.resultEncoding === 'ieee-f64-tree-v1'));
 else if (!probe && isNumberResult(workload)) driver = numberSupport['number-observation.rs'] + '\n' + numberSupport['number-driver.rs'];
 else if (!probe) driver = driver.replace('fn benchmark_checksum(value: i64) -> i64 { value }', rustChecksum(workload,{emitted,source,reference}));
 const declarations = [];
 if (workload.inputText !== undefined) declarations.push(`let input = String::from(arguments[${probe ? 2 : 5}].clone());`);
 if (workload.scalarCoordinates) declarations.push(workload.inputText !== undefined
  ? 'let coordinate_width = i64::try_from(input.chars().count()).expect("coordinate count exceeds int range");'
  : `let coordinate_width = ${workload.literalScalarCount}_i64;`);
 if (declarations.length) {
  const marker = probe ? '    let n: i64' : '    let iterations: i64';
  if (!driver.includes(marker)) throw new Error('Argument preparation hook changed');
  driver = driver.replace(marker, declarations.map(line => `    ${line}\n`).join('') + marker);
 }
 const argumentsCode = `${workload.inputText === undefined ? '' : ', &input'}${workload.scalarCoordinates ? ', coordinate_width' : ''}`;
 if (argumentsCode) {
  if (probe) driver = driver.replace('std::hint::black_box(7)))', `std::hint::black_box(7)${argumentsCode}))`);
  else driver = driver.replaceAll('std::hint::black_box(seed),', `std::hint::black_box(seed)${argumentsCode},`);
 }
 if (emitted !== undefined && !isCopying(workload) && !isRange(workload) && !isHigher(workload) && !isDeclared(workload) && !isConcat(workload) && !isReduction(workload)) {
  const {functionName,parameters,result,fallible} = ordinaryWorkloadBinding(emitted, workload, source);
  if(fallible){
   const args=parameters.split(', ').map(parameter=>parameter.split(':')[0]).join(',');
   driver=driver.replace(/\bfunction0(?=\s*\()/g,'benchmark_checked_workload');
   driver+=`\nfn benchmark_checked_workload(${parameters})->${result}{match ${functionName}(${args}){Ok(value)=>value,Err(_)=>{eprintln!("benchmark workload returned an unexpected checked error");std::process::exit(1)}}}\n`;
   return driver;
  }
  if (functionName !== 'function0') {
   if (!/\bfunction0\s*\(/.test(driver)) throw new Error('Workload invocation hook changed');
   driver = driver.replace(/\bfunction0(?=\s*\()/g, functionName);
  }
 }
 return driver;
}
export function adaptTSDriver(driver, workload) {
 if(isCopying(workload))assertCopyingPolicy(workload);
 if(isRange(workload))assertRangePolicy(workload);
 if(isHigher(workload))assertHigherPolicy(workload);
 if(isDeclared(workload))assertDeclaredPolicy(workload);
 if(isConcat(workload))assertConcatPolicy(workload);
 if(isApproximate(workload))return approximateAdapterSources['approximate-driver.ts'];
 if(isCopying(workload))driver=copyingSupport['copying-array-observation.ts']+'\n'+concatSupport['concat-driver.ts'].replace('observeConcat','observeCopying').replace('__CONCAT_DOMAIN__',JSON.stringify(workload.method==='toSpliced'?({int:'nested_int',number:'nested_number',boolean:'nested_boolean',string:'nested_string',nested:'deep_int',record:'nested_record',deep:'nested_deep_int',recursive:'nested_recursive_record'})[workload.family]:({nested:'nested_int',deep:'deep_int',recursive:'recursive_record'})[workload.family]??workload.family));
 else if(isRange(workload))driver=rangeSupport['range-mutation-observation.ts']+'\n'+concatSupport['concat-driver.ts'].replace('observeConcat','observeRange').replace('__CONCAT_DOMAIN__',JSON.stringify(({nested:'nested_int',deep:'deep_int',recursive:'recursive_record'})[workload.family]??workload.family));
 else if(isHigher(workload)&&(workload.result==='boolean'||workload.result.endsWith('[]')))driver=declaredSupport['declared-value-observation.ts']+'\n'+declaredSupport['declared-value-driver.ts'].replace('__DECLARED_RESULT__',JSON.stringify(workload.result));
 else if(isDeclared(workload)&&(workload.result==='boolean'||workload.result.endsWith('[]')))driver=declaredSupport['declared-value-observation.ts']+'\n'+declaredSupport['declared-value-driver.ts'].replace('__DECLARED_RESULT__',JSON.stringify(workload.result));
 else if(isConcat(workload)&&workload.result!=='int')driver=concatSupport['concat-observation.ts']+'\n'+concatSupport['concat-driver.ts'].replace('__CONCAT_DOMAIN__',JSON.stringify(workload.concatDomain));
 else if(isReductionSchema(workload))driver=reductionSupport['reduction-observation.ts']+'\n'+reductionSupport['reduction-driver.ts'].replace('__RESULT_TYPE__',JSON.stringify(workload.result));
 else if (isSliceResult(workload)) driver = sliceSupport['slice-observation.ts'] + '\n' + sliceSupport['slice-driver.ts'].replace('__FLOATING__', String(workload.resultEncoding === 'ieee-f64-tree-v1'));
 else if (isNumberResult(workload)) driver = numberSupport['number-observation.ts'] + '\n' + numberSupport['number-driver.ts'];
 if (workload.scalarCoordinates) {
  const value = workload.inputText === undefined ? String(workload.literalScalarCount) : 'Array.from(argv[4]).length';
  driver = driver.replace('let expected:', `const coordinateWidth = ${value};\nlet expected:`);
 }
 const argumentsCode = `${workload.inputText === undefined ? '' : ', argv[4]'}${workload.scalarCoordinates ? ', coordinateWidth' : ''}`;
 return argumentsCode ? driver.replaceAll('workload(iterations, seed)', `workload(iterations, seed${argumentsCode})`) : driver;
}

export function assertCLIInputs(workloads) {
 if (workloads.some(workload => workload.inputText?.includes("\u0000"))) throw new Error("CLI string inputs cannot contain NUL; use literal or separate correctness coverage");
}

// Scalar Number memory observation is formatted only after all native storage
// counters/disposal have been observed. f64 is copied without observer allocation.
export function adaptNumberMemoryDriver(driver, {native = false} = {}) {
 if (!native) return driver.replace('const checksum = probeChecksum(result);','const {checksum, exactBits} = observeNumber(result as number);').replace('JSON.stringify({checksum,','JSON.stringify({checksum, exactBits,');
 const hooks = [
  ['input_requested_bytes, after_output_drop) = {','input_requested_bytes, after_output_drop, number_value) = {'],
  ['let checksum = probe_checksum(&value);','let checksum = probe_checksum(&value);\n        let number_value = value;'],
  ['input_requested_bytes, after_output_drop)\n','input_requested_bytes, after_output_drop, number_value)\n'],
  ['    println!(','    let exact_bits = number_exact_bits(&number_value);\n    println!('],
  ['\\"checksum\\":{checksum},','\\"checksum\\":{checksum},\\"exactBits\\":{exact_bits:?},']
 ];
 for(const [marker,replacement]of hooks) {
  if (!driver.includes(marker)) throw new Error('Number memory driver hook changed');
  driver=driver.replace(marker,replacement);
 }
 return driver;
}

// Source-only signature guard for compact authored forEach comparisons. Actual
// emitted scalar ABI remains checked independently by assertABI. No body rewrite.
export function assertForEachSource(source,workload) {
 if(!/^array_for_each_(?:int|number|boolean|string|record|nested_(?:int|number|boolean|string|record)|deep_int|recursive_record|discard_(?:string|record|nested|callable))$/.test(workload.id)||!['int','number'].includes(workload.result))throw Error('Unknown forEach scalar source');
 const declarations=[...source.matchAll(/\bfunction\s+workload\s*\(([^)]*)\)\s*:\s*(int|number)\s*\{/g)];
 const expected='n:int,seed:int'+(workload.inputText===undefined?'':',input:string');
 if(declarations.length!==1||declarations[0][1].replace(/\s/g,'')!==expected||declarations[0][2]!==workload.result)throw Error('forEach workload source ABI changed');
}
