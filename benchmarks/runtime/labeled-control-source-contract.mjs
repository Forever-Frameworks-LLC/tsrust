import {createHash} from 'node:crypto';
export function assertLabeledControlSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='labeled-control'||row.id!=='labeled_control')throw Error('Labeled control family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Labeled control fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete labeled-control scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Labeled control denominator');
 for(const text of ["while (i < n)", "outer:for(let row=0;row<4;row++,updates++)", "if(row===skip)continue outer;", "if(row===3)break outer;", "trace+=String(row)+String(col);", "again:while(count<base%4+1)", "if(x===1)continue again;", "postLoop:do{post++;for(;;){continue postLoop;}}while(post<base%3+1);", "done:{if(base%2===0)break done;selected=2;}", "first:second:for(const value of [base,base+1])", "break first;", "output.push([trace,String(updates),String(count),String(post),String(selected),String(once),input+String(base)]);", "return output;"])if(!source.includes(text))throw Error('Actual labeled-control scenario: '+text);
}
