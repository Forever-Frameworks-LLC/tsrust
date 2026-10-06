import {createHash} from 'node:crypto';
export function assertBorrowedOptionalFallbackSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='borrowed-optional-fallback'||row.id!=='borrowed_optional_fallback')throw Error('Borrowed optional fallback family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Borrowed optional fallback fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One three-element borrowed optional fallback scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Borrowed optional fallback denominator');
 for(const text of ['while (i < n)','const rows:(Row|null)[]','for(const item of rows)','const label=item?.label??"none"','label.toUpperCase(),label.slice(),String(i)','return output;'])if(!source.includes(text))throw Error('Actual borrowed optional selections and complete retention');
 if(source.includes('clone('))throw Error('Hidden clone');
}
