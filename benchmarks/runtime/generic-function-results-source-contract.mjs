import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertGenericResultSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.id!=='generic_function_results'||row.family!=='generic-function-results')throw Error('Generic result family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Generic result fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One result-context generic alias scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Generic result denominator');
 const integer=kind==='native'?'int':'number';
 for(const text of ['function main() {}','function empty<T>(): T[] { return []; }','const plain = empty;',`const numbers: ${integer}[] = plain();`,'const strings: string[] = plain();','numbers.push(base);','strings.push("value");','function fresh<T>(): T[] { calls += 1; return []; }','const action = fresh;',`const ownedNumbers: ${integer}[] = action();`,'const ownedStrings: string[] = action();','ownedNumbers.push(base + 1);','ownedStrings.push("owner");','first = ownedNumbers[0];','length = ownedStrings[0].length;','output.push([numbers[0], strings[0].length, first, length, calls, input.length]);'])if(!containsCodeSnippet(source, text))throw Error('Generic result scenario: '+text);
}
