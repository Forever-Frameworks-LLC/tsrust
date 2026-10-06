import {createHash} from 'node:crypto';
export function assertMapIterationSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='map_iteration'||row.family!=='map-iteration')throw Error('Map iteration family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Map iteration fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete borrowed Map iteration scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Map iteration denominator');
 for(const text of ['for(const entry of values)','for(const [key,value] of values)','values.set("last",i+seed+2)','output.push([direct,pattern,String(count)])','return output;'])if(!source.includes(text))throw Error('Map iteration scenario: '+text);
}
