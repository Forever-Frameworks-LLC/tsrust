import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertNestedRestPatternsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='nested-rest-patterns'||row.id!=='nested_rest_patterns')throw Error('Nested rest family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Nested rest fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete nested-rest scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Nested rest denominator');
 for(const text of ['function describe([,...[text,fallback="tail"]]:string[])','while (i < n)','const [first,...[second,...tail]]=words;tail.push("end")','const [,...{length}]=words','const [head,...[text,amount]]=tuple','first.slice(),second.slice(),tail.join("|"),String(length),String(head),text,String(amount),describe([input.repeat(1),input.repeat(2)]),String(words.length)','return output;'])if(!containsCodeSnippet(source, text))throw Error('Actual nested patterns and complete retention');
 if(containsCodeSnippet(source, 'clone('))throw Error('Hidden extra clone');
}
