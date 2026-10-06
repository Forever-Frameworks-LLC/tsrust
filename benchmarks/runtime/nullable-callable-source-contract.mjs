import {createHash} from 'node:crypto';
export function assertNullableCallableSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='nullable-callable-transport')throw Error('Nullable callable source kind/family');
 const expected=kind==='native'?row.sourceSha256:row.counterpartSha256;
 if(createHash('sha256').update(source).digest('hex')!==expected)throw Error('Nullable callable source fingerprint');
 if(row.operationsPerIteration!==1||row.result!=='int[][]'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Nullable callable scenario/observer contract');
 if((source.match(/output\.push\(/g)??[]).length!==2||!source.includes('while (i < n)')||!source.includes('return output;'))throw Error('Complete result and effect retention');
 if(kind==='typescript'&&!source.startsWith('type int = number;\n'))throw Error('Int counterpart substitution');
}
