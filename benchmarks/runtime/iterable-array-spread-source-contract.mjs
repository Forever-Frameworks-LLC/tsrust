import {createHash} from 'node:crypto';
export function assertIterableArraySpreadSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='iterable_array_spread'||row.family!=='iterable-array-spread')throw Error('Iterable array spread family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Iterable array spread fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete iterable array spread scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Iterable array spread denominator');
 for(const text of ['const copied=[0,...keys,i+seed+3];keys.clear();','const characters=[...input,..."é🙂"];','const entries=[...values];','const moved=[...rows];','for(const [key,value] of entries)','return output;'])if(!source.includes(text))throw Error('Iterable array spread scenario: '+text);
 if(kind==='native'&&!source.includes('action:Once<Function<int>>;'))throw Error('Noncloneable native values');
 if(kind==='typescript'&&!source.includes('action:()=>number;'))throw Error('TypeScript callable counterpart');
}
