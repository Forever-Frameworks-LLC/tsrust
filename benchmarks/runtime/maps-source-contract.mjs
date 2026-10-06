import {createHash} from 'node:crypto';
export function assertMapsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='map_values'||row.family!=='maps')throw Error('Map family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Map fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete map scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Map denominator');
 const integer=kind==='native'?'int':'number';
 for(const text of [`function workload(n: ${integer}, seed: ${integer}, input: string): string[][]`,`new Map<${integer},${integer}>([[i+seed,1],[i+seed+1,2],[i+seed,3]])`,'values.get(i+seed)','values.set(i+seed+1,4)','values.has(i+seed+1)','values.delete(i+seed)',`new Map<${integer},string>([[1,String(input)],[2,"tail"]])`,'const copied=new Map(labels);','labels.clear();','copied.get(1)','values.clear();','return output;'])if(!source.includes(text))throw Error('Map scenario: '+text);
}
