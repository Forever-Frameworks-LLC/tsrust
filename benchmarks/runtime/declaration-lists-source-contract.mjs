import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertDeclarationListsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='declaration-lists'||row.id!=='declaration_lists')throw Error('Declaration list family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Declaration list fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete declaration-list scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Declaration list denominator');
 for(const text of ['while (i < n)','const base=(i*13+seed)%97,low=base%5,high=low+6,words=[input.repeat(1),String(base)],[first,second]=words;','let sum=0,steps=0,text="";','for(let a=low,b=high;a<b;a++,b--){sum+=b-a;steps++;}','for(let k=0,limit=2;k<limit;k++,text+=first.slice()){if(k===0){continue;}}','output.push([String(sum),String(steps),text,second.slice(),String(low+high)]);','return output;'])if(!containsCodeSnippet(source, text))throw Error('Actual declaration-list scenario: '+text);
 if(containsCodeSnippet(source, 'clone('))throw Error('Hidden extra clone');
}
