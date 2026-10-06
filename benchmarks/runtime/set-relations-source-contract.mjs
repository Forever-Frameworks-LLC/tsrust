import {createHash} from 'node:crypto';
export function assertSetRelationsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='set_relations'||row.family!=='set-relations')throw Error('Set relation family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Set relation fingerprint');
 if(kind==='typescript'&&!source.startsWith('/// <reference lib="esnext.collection" />\n'))throw Error('Set relation declaration library');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete set relation scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Set relation denominator');
 for(const text of ['small.isSubsetOf(large)','large.isSupersetOf(small)','small.isDisjointFrom(apart)','small.isDisjointFrom(large)','names.isDisjointFrom(labels)','names.isSubsetOf(labels)','return output;'])if(!source.includes(text))throw Error('Set relation scenario: '+text);
}
