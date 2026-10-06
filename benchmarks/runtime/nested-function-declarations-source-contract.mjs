import {createHash} from 'node:crypto';
export function assertNestedFunctionDeclarationsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='nested_function_declarations'||row.family!=='nested-function-declarations')throw Error('Nested declaration family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Nested declaration fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete lexical declaration scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Nested declaration denominator');
 for(const text of ['function double(value: int): int','factorial(value - 1)','return odd(value - 1)','return even(value - 1)','shadowed = double(value);','return value * 3','[value, value + 1].map(double)','[first, factorial(5), even(value) ? 1 : 0, shadowed, mapped[0], mapped[1], scoped + value]','let value = first + 3;', '(value: int): int => value + 1', 'scoped = increment(value);', 'return output;'])if(!source.includes(text))throw Error('Nested declaration scenario: '+text);
 if(source.indexOf('shadowed = double(value);')>source.lastIndexOf('function double'))throw Error('Nested forward shadow call');
}
