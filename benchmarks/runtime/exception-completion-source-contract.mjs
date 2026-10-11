import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertExceptionCompletionSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='exception_completion'||row.family!=='exception-completion')throw Error('Exception family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Exception fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete exception completion scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Exception denominator');
 for(const text of ['if(value%3===0){throw value;}','return value+1;','return error+2;','finally{trace.push(value);}','finally{return value+4;}','const result=attempt(value,trace);','trace.join(",")','const owned=retained(input+String(value),value%2===0,trace);','if(early){return value;}','finally{trace.push(7);}','const field=retainedField({text:owned,stamp:11},value%2===0,trace);','if(early){return value.text;}','finally{trace.push(value.stamp);}','trace.join(","),field','return output;'])if(!containsCodeSnippet(source, text))throw Error('Exception scenario: '+text);
}
