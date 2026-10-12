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
 if(row.id==='native_json_string_projection'){
  if(row.family!=='native-json'||!['native','typescript'].includes(kind))throw Error('JSON string projection identity');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('JSON string projection fingerprint');
  if(row.iterations!==2000||row.inputText!=='"hellohello"'||row.result!=='int'||row.operationsPerIteration!==1||row.operationUnit!=='One string guard, borrowed local, length and two searches')throw Error('JSON string projection denominator');
  if(kind==='native'&&!containsCodeSnippet(source," const owner=JSON.parse(input);\n let checksum=seed;let i=0;\n while(i<n){\n  if(typeof owner!=='string'){throw 'expected-string';}\n  const text=owner;checksum+=text.length+text.indexOf('hello')+text.lastIndexOf('hello');\n  i++;\n }\n return checksum;\n"))throw Error('JSON string projection operation placement');
  if(kind==='typescript'&&!containsCodeSnippet(source," const owner:unknown=JSON.parse(input);\n let checksum=seed;let i=0;\n while(i<n){\n  if(typeof owner!=='string'){throw 'expected-string';}\n  const text=owner;checksum+=text.length+text.indexOf('hello')+text.lastIndexOf('hello');\n  i++;\n }\n return checksum;\n"))throw Error('JSON string projection operation placement');
  return;
 }
 if(row.id==='native_json_string_views'){
  if(row.family!=='native-json'||!['native','typescript'].includes(kind))throw Error('JSON string view identity');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('JSON string view fingerprint');
  if(row.iterations!==2000||row.inputText!=='"hellohello"'||row.result!=='int'||row.operationsPerIteration!==1||row.operationUnit!=='One checked string extraction, length and two searches')throw Error('JSON string view denominator');
  if(kind==='native'&&!containsCodeSnippet(source," const owner=JSON.parse(input);\n let checksum=seed;let i=0;\n while(i<n){\n  match(asString(owner)){Ok(text)=>{checksum+=text.length+text.indexOf('hello')+text.lastIndexOf('hello');},Err(error)=>{throw error.message;}}\n  i++;\n }\n return checksum;\n"))throw Error('JSON string view operation placement');
  if(kind==='typescript'&&!containsCodeSnippet(source," const owner:unknown=JSON.parse(input);\n let checksum=seed;let i=0;\n while(i<n){\n  if(typeof owner!=='string'){throw 'expected-string';}\n  const text=owner;checksum+=text.length+text.indexOf('hello')+text.lastIndexOf('hello');\n  i++;\n }\n return checksum;\n"))throw Error('JSON string view operation placement');
  return;
 }
 if(row.id==='native_json_string_reads'){
  if(row.family!=='native-json'||!['native','typescript'].includes(kind))throw Error('JSON string reads identity');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('JSON string reads fingerprint');
  if(row.iterations!==2000||row.inputText!=='"hellohello"'||row.result!=='int'||row.operationsPerIteration!==1||row.operationUnit!=='One parse, string guard, length and five searches')throw Error('JSON string reads denominator');
  if(!containsCodeSnippet(source,"  let checksum = seed;\n  let i = 0;\n  while (i < n) {\n    const value = JSON.parse(input);\n    if (typeof value !== 'string') { throw 'Expected string'; }\n    if (!value.startsWith('hello') || !value.endsWith('hello') || !value.includes('ell')) { throw 'Expected matches'; }\n    checksum += value.length + value.indexOf('hello') + value.lastIndexOf('hello');\n    i++;\n  }\n  return checksum;\n"))throw Error('JSON string reads operation placement');
  return;
 }

 if(row.id==='native_json_refinement'){
  if(row.family!=='native-json'||!['native','typescript'].includes(kind))throw Error('JSON refinement identity');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('JSON refinement fingerprint');
  if(row.iterations!==2000||row.inputText!=='true'||row.result!=='int'||row.operationsPerIteration!==1||row.operationUnit!=='One parse, boolean guard and boolean payload branch')throw Error('JSON refinement denominator');
  if(!containsCodeSnippet(source,"  let checksum = seed;\n  let i = 0;\n  while (i < n) {\n    const value = JSON.parse(input);\n    if (typeof value !== 'boolean') { throw 'Expected boolean'; }\n    if (value) { checksum += 1; }\n    i++;\n  }\n  return checksum;\n"))throw Error('JSON refinement operation placement');
  return;
 }

 if(row.id==='native_json_views'){
  if(row.family!=='native-json'||!['native','typescript'].includes(kind))throw Error('JSON view identity');
  if(createHash('sha256').update(source).digest('hex')!==row[kind==='native'?'sourceSha256':'counterpartSha256'])throw Error('JSON view fingerprint');
  if(row.iterations!==2000||row.inputText!=='{"items":[42,null]}'||row.result!=='int'||row.operationsPerIteration!==1||row.operationUnit!=='One object lookup, array lookup and checked integer extraction')throw Error('JSON view denominator');
  const native=" while(i<n){\n  match(get(owner,'items')){Ok(items)=>{if(items!==undefined){\n   match(at(items,0)){Ok(child)=>{if(child!==undefined){\n    match(asInt(child)){Ok(value)=>{checksum+=value;},Err(error)=>{throw error.message;}}\n   }else{throw 'missing item';}},Err(error)=>{throw error.message;}}\n  }else{throw 'missing array';}},Err(error)=>{throw error.message;}}\n  i++;\n }\n";
  const ts=" while(i<n){\n  if(typeof owner!=='object'||owner===null||Array.isArray(owner)){throw 'expected object';}\n  const items=owner['items'];\n  if(items===undefined){throw 'missing array';}\n  if(!Array.isArray(items)){throw 'expected array';}\n  const child=items[0];\n  if(child===undefined){throw 'missing item';}\n  if(typeof child!=='number'||!Number.isSafeInteger(child)){throw 'expected integer';}\n  checksum+=child;\n  i++;\n }\n";
  for(const snippet of ['const owner=JSON.parse(input);','let checksum=seed;','let i=0;',kind==='native'?native:ts,'return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('JSON view operation placement');
  return;
 }
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

export function viewChecksum(n,seed,input){
 if(input!=='{"items":[42,null]}'||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*42))throw Error('JSON view oracle inputs');
 return seed+n*42;
}

export function refinementChecksum(n,seed,input){
 if(input!=='true'||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n))throw Error('JSON refinement oracle inputs');
 return seed+n;
}

export function stringReadsChecksum(n,seed,input){
 if(input!=='"hellohello"'||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*15))throw Error('JSON string reads oracle inputs');
 return seed+n*15;
}
