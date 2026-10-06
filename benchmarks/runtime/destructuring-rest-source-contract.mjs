import {createHash} from 'node:crypto';
export function assertDestructuringRestSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='destructuring-rest'||row.id!=='destructuring_rest')throw Error('Rest/spread family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Rest/spread fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete rest and object-spread scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Rest/spread denominator');
 for(const text of ['while (i < n)','const [first,...rest]=words;rest.push("tail")','const [,...tupleTail]=pair','const merged={before:base,...row,count:base+1,after:seed}','const {name:label,...fields}=row','first.slice(),rest.join("|"),tupleTail[0].slice(),label,String(fields.count),Object.keys(fields).join(","),merged.name.slice(),String(merged.count),Object.keys(merged).join(","),String(words.length),String(rest.length)','return output;'])if(!source.includes(text))throw Error('Actual rest/spread and complete retention');
 if(source.includes('clone('))throw Error('Hidden extra clone');
}
