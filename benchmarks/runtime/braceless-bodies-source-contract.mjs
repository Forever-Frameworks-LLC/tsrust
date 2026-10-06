import {createHash} from 'node:crypto';
export function assertBracelessBodiesSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='braceless-bodies'||row.id!=='braceless_bodies')throw Error('Braceless body family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Braceless body fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete single-statement-body scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Braceless body denominator');
 for(const text of [' if(value<20)return "low";\n else if(value<60)return "mid";\n else return "high";','while (i < n)','for(let k=0;k<base%5+1;k++)sum+=k;','for(const part of [input.repeat(1),String(base)])text+=part;','for(let a=0;a<3;a++)for(let b=0;b<3;b++)if(a===b)odd+=1;else if(a+b===2)odd+=10;','while(down>1)down--;','do down+=2;while(down<4);','output.push([grade(base),String(sum),text,String(odd),String(down)]);','return output;'])if(!source.includes(text))throw Error('Actual single-statement-body scenario: '+text);
}
