import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertArrayFromCollectionsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='array_from_collections'||row.family!=='array-from-collections')throw Error('Array.from collections family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Array.from collections fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete Array.from collection scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Array.from collections denominator');
 for(const text of ['const copied=Array.from(keys);keys.clear();','Array.from(new Set<','const entries=Array.from(values);','const labels=Array.from(values,([key,value],index)','const read=Array.from(rows,(entry,index)','entry[1].label+":"+String(entry[0]+index)','return output;'])if(!containsCodeSnippet(source, text))throw Error('Array.from collections scenario: '+text);
 if(kind==='native'&&!containsCodeSnippet(source, 'action:Once<Function<int>>;'))throw Error('Array.from noncloneable native values');
 if(kind==='typescript'&&!containsCodeSnippet(source, 'action:()=>number;'))throw Error('Array.from TypeScript callable counterpart');
}
