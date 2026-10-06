import {createHash} from 'node:crypto';
export function assertDeferredGenericSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.id!=='generic_function_deferred'||row.family!=='generic-function-deferred')throw Error('Deferred generic family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Deferred generic fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One deferred generic callable scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Deferred generic denominator');
 for(const text of ['const action = identity;','const pick = first;','const length = size;','const choose = offset;','const numberResult = action(base);','const stringLength = action("owner").length;','const result = action([base + 1, base + 2]);','const arrayFirst = result[0];','const headResult = pick(base + 2, base + 3);','const selected = choose(base + 4);','const remaining = length(values);','output.push([numberResult, stringLength, arrayFirst, headResult, selected, remaining, input.length]);'])if(!source.includes(text))throw Error('Deferred generic scenario: '+text);
 const removal=kind==='native'?'values.remove(0)':'values.shift()!';
 const integer=kind==='native'?'int':'number';
 for(const text of ['function identity<T>(value: T): T { return value; }',`function first<T>(...values: T[]): T { return ${removal}; }`,`function size<T>(values: T[]): ${integer} { return values.length; }`,`function offset<T>(value: T, index: ${integer} = 0): T { return value; }`])if(!source.includes(text))throw Error('Deferred generic declaration: '+text);
}
