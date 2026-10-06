import {createHash} from 'node:crypto';
export function assertNestedFunctionCapturesSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='nested_function_captures'||row.family!=='nested-function-captures')throw Error('Nested capture family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Nested capture fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete nested capture scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Nested capture denominator');
 for(const text of ['total += amount;','history.push(amount);','add(amount);\n    add(amount);','function shifted(item: int): int { return item + base[0]; }','return depth(level - 1) + 1;','[1, 2].map(shifted)','let later: int;','function readLater(): int { return later * 2; }','later = value + 3;','function tick(): int { counts[0] += value; return counts[0]; }','const run = tick;','[total, history.length, mapped[0], mapped[1], depth(3), readLater(), counts[0]]','return output;'])if(!source.includes(text))throw Error('Nested capture scenario: '+text);
 if(source.indexOf('later = value + 3;')<source.indexOf('function readLater'))throw Error('Nested capture later initialization');
}
