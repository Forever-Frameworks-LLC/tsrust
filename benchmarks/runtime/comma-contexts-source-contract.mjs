import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertCommaContextsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='comma_contexts'||row.family!=='comma-contexts')throw Error('Comma expression family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Comma expression fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete comma receiver and guard scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Comma expression denominator');
 for(const text of ['(mark(trace,1),values).push(j+2)','(mark(trace,2),values).fill(8,1,2)','(mark(trace,3),values).reverse()','value!==(mark(trace,5),null)','typeof alternative)===(mark(trace,7),"string")','row.value!==(reset(row),null)','return output;'])if(!containsCodeSnippet(source, text))throw Error('Comma expression scenario: '+text);
}
