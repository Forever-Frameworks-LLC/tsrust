import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertStringConcatenationSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='string_concatenation'||row.family!=='string-concatenation')throw Error('Concatenation family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Concatenation fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete scalar concatenation scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Concatenation denominator');
 for(const text of ['input+k,k+input','text+=k;text+=flag;','"fraction:"+(Number(k%4)/2.0)','"flag:"+flag','"absent:"+null+undefined','"union:"+nullable','return output;'])if(!containsCodeSnippet(source, text))throw Error('Concatenation scenario: '+text);
}
