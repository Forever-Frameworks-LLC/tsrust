import {createHash} from 'node:crypto';
export function assertCommaExpressionsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='comma_expressions'||row.family!=='comma-expressions')throw Error('Comma expression family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Comma expression fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete comma expression scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Comma expression denominator');
 for(const text of ['return observe(trace,value),value+1;','i++,j++','observe(trace,j),observe(trace,j+2),j*3','observe(trace,j+4),String(input)','trace.join(",")','return output;'])if(!source.includes(text))throw Error('Comma expression scenario: '+text);
}
