import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertGenericContextSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.id!=='generic_function_contexts'||row.family!=='generic-function-contexts')throw Error('Generic context family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Generic context fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One contextual generic callable scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Generic context denominator');
 const numberValue=kind==='native'?'const numbers: Mutable<Function<int, [value: int]>> = select;':'const numbers: (value: number) => number = select;';
 for(const text of [numberValue,'calls += 1; return value;','numberResult = numbers(base);','stringLength = strings("owner").length;','const result = arrays([base + 1, base + 2]);','arrayFirst = result[0];','headResult = first(values);','remaining = values.length;','output.push([numberResult, stringLength, arrayFirst, headResult, remaining, calls, input.length]);'])if(!containsCodeSnippet(source, text))throw Error('Generic context scenario: '+text);
 const removal=kind==='native'?'values.remove(0)':'values.shift()!';if(!containsCodeSnippet(source, removal))throw Error('Generic context removal');
}
