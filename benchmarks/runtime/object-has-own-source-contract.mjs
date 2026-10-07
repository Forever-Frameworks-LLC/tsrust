import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertObjectHasOwnSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='object-has-own'||row.id!=='object_has_own')throw Error('Object membership family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Object membership fingerprint');
 if(row.operationsPerIteration!==9||row.iterations!==2048||row.operationUnit!=='One own-property query'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Object membership denominator');
 const receiver=kind==='native'?'input':'input as unknown as object';
 for(const text of ['while (i < n)','label:undefined','Object.hasOwn(row,"label")','Object.hasOwn(row,"missing")','Object.hasOwn(row,"toString")','Object.hasOwn(values,position)','Object.hasOwn(values,"length")','Object.hasOwn(values,"01")',`Object.hasOwn(${receiver},position)`,`Object.hasOwn(${receiver},"length")`,'Object.hasOwn(values,-0.0)','output.push([','return output;'])if(!containsCodeSnippet(source, text))throw Error('Actual membership queries and complete retention');
 if(containsCodeSnippet(source, 'clone('))throw Error('Hidden clone');
}
