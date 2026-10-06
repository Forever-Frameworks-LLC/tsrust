import {createHash} from 'node:crypto';
export function assertStringIterationSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='string_iteration'||row.family!=='string-iteration')throw Error('String iteration family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('String iteration fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete Unicode string iteration scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('String iteration denominator');
 for(const text of ['for(const character of input)','for(const outer of input)','for(const inner of "ab")','if(character!=="é")','output.push([String(i+seed)+":"+trace,String(count),filtered,nested])','return output;'])if(!source.includes(text))throw Error('String iteration scenario: '+text);
}
