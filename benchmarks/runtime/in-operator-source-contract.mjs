import {createHash} from 'node:crypto';
export function assertInOperatorSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='in-operator'||row.id!=='in_operator')throw Error('In operator family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('In operator fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete in-operator and key-loop scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('In operator denominator');
 for(const text of ["while (i < n)", "const present=wanted in sample;", "const positioned=index in values;", "const fractional=(Number(index)+0.5) in values;", "const length=\"length\" in values;", "for(const key in sample){names+=key+\",\";}", "for(const key in values){indices+=key;}", "output.push([String(present),String(positioned),String(fractional),String(length),names,indices,sample.name,String(sample.amount)]);", "return output;"])if(!source.includes(text))throw Error('Actual in-operator scenario: '+text);
}
