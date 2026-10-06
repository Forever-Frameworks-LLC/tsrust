import {createHash} from 'node:crypto';
export function assertMapGroupBySource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='map_group_by'||row.family!=='map-group-by')throw Error('Map grouping family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Map grouping fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete Map grouping scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Map grouping denominator');
 for(const text of ['const groups=Map.groupBy(source,','visits+=index+1;return value%2;','source.push(k+4);','const characters=Map.groupBy(input,','Array.from(groups.keys()).join','String(characters.size),alpha,other','return output;'])if(!source.includes(text))throw Error('Map grouping scenario: '+text);
}
