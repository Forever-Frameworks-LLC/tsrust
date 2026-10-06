import {createHash} from 'node:crypto';
export function assertCapturedGenericSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.id!=='generic_function_captures'||row.family!=='generic-function-captures')throw Error('Captured generic family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Captured generic fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One captured generic environment scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Captured generic denominator');
 const integer=kind==='native'?'int':'number';
 for(const text of ['function main() {}',`function counter(start: ${integer}) {`,`function visit<T>(value: T): ${integer} { calls += 1; return calls; }`,'return visit;','const first = counter(base);','const second = counter(base + 10);','const firstCount = first(base);','const secondCount = second("value");','const sharedCount = first([base, base + 1]);','function select<T>(value: T): T { local += 1; return value; }','const action = select;','numberResult = action(base);','stringLength = action("value").length;','const result = action([base + 3, base + 4]);','arrayFirst = result[0];','const finalCount = second(base);','output.push([firstCount, secondCount, sharedCount, numberResult, stringLength, arrayFirst, local, finalCount, input.length]);'])if(!source.includes(text))throw Error('Captured generic scenario: '+text);
}
