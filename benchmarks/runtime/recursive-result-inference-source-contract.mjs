import {createHash} from 'node:crypto';
import {codeTokens} from './source-contract-tokens.mjs';
const spelling=text=>codeTokens(text,true).map(token=>token.text).join(' ');
const required=["function factorial(count: int)", "return count * factorial(count - 1);", "function parity(count: int)", "!parity(count - 1)", "function label(count: int)", "\"x\" + label(count - 1)", "function depth(count: int)", "return 1 + depth(count - 1);", "const fibonacci = function fib(count: int)", "fib(count - 1) + fib(count - 2)", "[factorial(value), parity(value) ? 1 : 0, label(value).length, depth(value), steps.length, fibonacci(value)]"];
export function assertRecursiveResultInferenceSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='recursive_result_inference'||row.family!=='recursive-result-inference')throw Error('Recursive result inference family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Recursive result inference fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete recursive inference scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Recursive result inference denominator');
 const tokens=spelling(source);
 const stated=['function factorial(count: int):','function parity(count: int):','function label(count: int):','function depth(count: int):','function fib(count: int):'].filter(text=>tokens.includes(spelling(text)));
 if(kind==='native'&&stated.length!==0)throw Error('Recursive result inference native annotation: '+stated[0]);
 if(kind==='typescript'&&stated.length!==5)throw Error('Recursive result inference counterpart annotations');
 for(const text of required)if(!tokens.includes(spelling(text)))throw Error('Recursive result inference scenario: '+text);
}
