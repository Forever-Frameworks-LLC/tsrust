import {createHash} from 'node:crypto';
export function assertNestedResultSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.id!=='nested_function_results'||row.family!=='nested-function-results')throw Error('Named result family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Named result fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One inferred named result scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Named result denominator');
 const scalar=kind==='native'?'int':'number';
 for(const text of [`function double(value: ${scalar}) { return value + value; }`,'function identity<T>(value: T) { return value; }',`function pair(value: ${scalar}) { return [value, value + 1]; }`,`function offset(extra: ${scalar}) { return value + extra; }`,'const values = pair(value);','output.push([double(value), identity(value + 3), offset(2), values[0], values[1], input.length]);'])if(!source.includes(text))throw Error('Named result scenario: '+text);
}
