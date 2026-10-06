import {createHash} from 'node:crypto';
export function assertIterableCallSpreadSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='iterable_call_spread'||row.family!=='iterable-call-spread')throw Error('Iterable call spread family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Iterable call spread fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete iterable call spread scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Iterable call spread denominator');
 for(const text of ['const sum=total(...keys);keys.clear();','const characters=letters(...input);','const combined=pair(...values);','const selected=first(...new Set<','const text=label(...rows);','const raw=String.raw(','return output;'])if(!source.includes(text))throw Error('Iterable call spread scenario: '+text);
 if(kind==='native'&&!source.includes('action:Once<Function<int>>;'))throw Error('Noncloneable native values');
 if(kind==='typescript'&&(!source.includes('action:()=>number;')||!source.includes('function pair(...entries:[number,number][]')||!source.includes('function first<T>(...values:T[]')||!source.includes('function label(...entries:[number,ActionRow][]')))throw Error('TypeScript callable counterpart');
}
