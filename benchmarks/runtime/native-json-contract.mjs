import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export const jsonInput=' { "a": [1,true,null], "b": "hello" } ';
// Fixed ASCII output length, independently counted. Neither runtime's JSON
// parser/encoder is used to calculate the expected checksum.
export function jsonChecksum(n,seed,input){
 if(input!==jsonInput||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*31))throw Error('JSON oracle inputs');
 return seed+n*31;
}
export function scalarChecksum(n,seed,input){
 if(input!=='42'||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*42))throw Error('JSON scalar oracle inputs');
 return seed+n*42;
}
export function assertNativeJsonSource(source,row,kind){
 if(row.id==='native_json_inspection'){
  if(row.family!=='native-json'||!['native','typescript'].includes(kind))throw Error('JSON inspection identity');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('JSON inspection fingerprint');
  if(row.iterations!==2000||row.inputText!=='[null]'||row.result!=='int'||row.operationsPerIteration!==1||row.operationUnit!=='Two parses, array inspection and two null comparisons')throw Error('JSON inspection denominator');
  for(const snippet of ['let checksum = seed;','let i = 0;',`while (i < n) {
    const value = JSON.parse(input);
    if (!Array.isArray(value) || value === null) { throw 'Expected array'; }
    if (JSON.parse('null') !== null) { throw 'Expected null'; }
    checksum += 2;
    i++;
  }`,'return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('JSON inspection operation placement');
  return;
 }
 if(row.id==='native_json_typeof'){
  if(row.family!=='native-json'||!['native','typescript'].includes(kind))throw Error('JSON typeof identity');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('JSON typeof fingerprint');
  if(row.iterations!==2000||row.inputText!=='42'||row.result!=='int'||row.operationsPerIteration!==1||row.operationUnit!=='One parse and runtime typeof inspection')throw Error('JSON typeof denominator');
  for(const snippet of ['let checksum = seed;','let i = 0;',`while (i < n) {
    const label = typeof JSON.parse(input);
    if (label !== 'number') { throw 'Unexpected JSON type'; }
    checksum += label.length;
    i++;
  }`,'return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('JSON typeof operation placement');
  return;
 }
 if(row.id==='native_json_scalars'){
  if(row.family!=='native-json'||!['native','typescript'].includes(kind))throw Error('JSON scalar identity');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('JSON scalar fingerprint');
  if(row.iterations!==2000||row.inputText!=='42'||row.result!=='int'||row.operationsPerIteration!==1||row.operationUnit!=='Three parses and four checked scalar extractions')throw Error('JSON scalar denominator');
  const native=`while (i < n) {
    const value = JSON.parse(input);
    match (asInt(value)) { Ok(value) => { checksum += value; }, Err(error) => { throw error.message; } }
    match (asNumber(value)) { Ok(value) => { if (value !== 42.0) { throw 'number'; } }, Err(error) => { throw error.message; } }
    match (asBoolean(JSON.parse('true'))) { Ok(value) => { if (!value) { throw 'boolean'; } }, Err(error) => { throw error.message; } }
    match (takeString(JSON.parse('"hello"'))) { Ok(value) => { if (value !== 'hello') { throw 'string'; } }, Err(error) => { throw error.message; } }
    i++;
  }`;
  const ts=`while (i < n) {
    const value: unknown = JSON.parse(input);
    if (typeof value !== 'number' || !Number.isSafeInteger(value)) { throw 'integer'; }
    checksum += value;
    if (typeof value !== 'number' || value !== 42.0) { throw 'number'; }
    const boolean: unknown = JSON.parse('true');
    if (typeof boolean !== 'boolean' || !boolean) { throw 'boolean'; }
    const string: unknown = JSON.parse('"hello"');
    if (typeof string !== 'string' || string !== 'hello') { throw 'string'; }
    i++;
  }`;
  for(const snippet of ['let checksum = seed;','let i = 0;',kind==='native'?native:ts,'return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('JSON scalar operation placement');
  return;
 }

 if(!['native_json_roundtrip','native_json_namespace'].includes(row.id)||row.family!=='native-json'||!['native','typescript'].includes(kind))throw Error('JSON workload identity');
 if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('JSON source fingerprint');
 if(row.iterations!==2000||row.inputText!==jsonInput||row.result!=='int'||row.operationsPerIteration!==1||row.operationUnit!=='One parse, compact encode and exact output comparison')throw Error('JSON denominator');
 const native=`while (i < n) {
  match (parse(input)) {
   Ok(value) => {
    match (stringify(value)) {
     Ok(text) => {
      if (text !== '{"a":[1,true,null],"b":"hello"}') {throw "Unexpected JSON output";}
      checksum += text.length;
     }, Err(error) => {throw error.message;}
    }
   }, Err(error) => {throw error.message;}
  }
  i++;
 }`;
 const ts=`while (i < n) {
  const text = JSON.stringify(JSON.parse(input));
  if (text !== '{"a":[1,true,null],"b":"hello"}') {throw "Unexpected JSON output";}
  checksum += text.length;
  i++;
 }`;
 for(const snippet of ['let checksum = seed;','let i = 0;',kind==='native'&&row.id==='native_json_roundtrip'?native:ts,'return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('JSON operation placement');
}

export function typeofChecksum(n,seed,input){
 if(input!=='42'||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*6))throw Error('JSON typeof oracle inputs');
 return seed+n*6;
}

export function inspectionChecksum(n,seed,input){
 if(input!=='[null]'||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*2))throw Error('JSON inspection oracle inputs');
 return seed+n*2;
}
