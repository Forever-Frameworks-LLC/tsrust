import {createHash} from 'node:crypto';
export function assertTypeAssertionsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='type_assertions'||row.family!=='type-assertions')throw Error('Type assertions family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Type assertions fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete checked assertion scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Type assertions denominator');
 const integer=kind==='native'?'int':'number';
 for(const text of [`function workload(n: ${integer}, seed: ${integer}, input: string): string[][]`,`[] as ${integer}[]`,'values.push(value);','value satisfies number','const widened=value as number;',`value as ${integer}|string`,'{value:checked} satisfies Row','typeof alternative===','String(widened+0.5)','String(row.value)','input+rendered','return output;'])if(!source.includes(text))throw Error('Type assertions scenario: '+text);
}
