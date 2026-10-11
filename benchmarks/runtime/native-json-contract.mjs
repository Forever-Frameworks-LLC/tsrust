import {createHash} from 'node:crypto';
import {containsCodeSnippet} from './source-contract-tokens.mjs';
export const jsonInput=' { "a": [1,true,null], "b": "hello" } ';
// Fixed ASCII output length, independently counted. Neither runtime's JSON
// parser/encoder is used to calculate the expected checksum.
export function jsonChecksum(n,seed,input){
 if(input!==jsonInput||!Number.isSafeInteger(n)||n<0||!Number.isSafeInteger(seed)||!Number.isSafeInteger(seed+n*31))throw Error('JSON oracle inputs');
 return seed+n*31;
}
export function assertNativeJsonSource(source,row,kind){
 if(row.id!=='native_json_roundtrip'||row.family!=='native-json'||!['native','typescript'].includes(kind))throw Error('JSON workload identity');
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
 for(const snippet of ['let checksum = seed;','let i = 0;',kind==='native'?native:ts,'return checksum;'])if(!containsCodeSnippet(source,snippet))throw Error('JSON operation placement');
}
