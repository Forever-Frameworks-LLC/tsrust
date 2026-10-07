import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertNestedRestSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.id!=='nested_function_rest'||row.family!=='nested-function-rest')throw Error('Named rest family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Named rest fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One captured rest callable scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Named rest denominator');
 const scalar=kind==='native'?'int':'number';
 for(const text of [`function sum(...values: ${scalar}[]): ${scalar} {`,`function front(first: ${scalar} = base, ...tail: ${scalar}[]): ${scalar} { return first + tail.length; }`,'for (const value of values) { total += value; }','const action = sum;','const empty = action();','const supplied = action(2, 3);','const pack = [4, 5];','const spread = action(...pack);','const omitted = prefix();','const undefinedFirst = prefix(undefined, 1, 2);','const explicit = prefix(3, 4);','output.push([empty, supplied, spread, omitted, undefinedFirst, explicit, input.length]);'])if(!containsCodeSnippet(source, text))throw Error('Named rest scenario: '+text);
}
