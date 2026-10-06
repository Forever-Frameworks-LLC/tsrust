import {createHash} from 'node:crypto';
export function assertForPatternInitializersSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='for-pattern-initializers'||row.id!=='for_pattern_initializers')throw Error('For pattern family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('For pattern fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete classic-for pattern scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('For pattern denominator');
 for(const text of ['function bounds(base:int):Bounds{return {start:base%3,end:base%3+3};}','while (i < n)','for(let {start:k,end}=bounds(base);k<end;k++){if(k===base%3+1){continue;}sum+=k;}','for(let [j,stop=2]=[0];j<stop;j++){count+=j+1;}','for(const [first,...tail]=words;tail.length<3;tail.push("x")){text+=first+String(tail.length);}','output.push([String(sum),String(count),text,String(words.length)]);','return output;'])if(!source.includes(text))throw Error('Actual classic-for pattern scenario: '+text);
 if(source.includes('clone('))throw Error('Hidden extra clone');
}
