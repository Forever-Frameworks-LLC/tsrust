import {createHash} from 'node:crypto';
export function assertDestructuringAssignmentSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='destructuring-assignment'||row.id!=='destructuring_assignment')throw Error('Destructuring assignment family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Destructuring assignment fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete destructuring-assignment scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Destructuring assignment denominator');
 for(const text of ['while (i < n)','[left,right]=[right,left];','for(let turn=0;turn<base%4+2;[f0,f1]=[f1,f0+f1],turn++){}','[count,word]=ends(base,input);','const cell:Cell={x,y,label};','({x:cx,label:text}=cell);','output.push([left,right,String(f0),String(count),word,String(cx+cell.y),text]);','return output;'])if(!source.includes(text))throw Error('Actual destructuring-assignment scenario: '+text);
 if(source.includes('clone('))throw Error('Hidden extra clone');
}
