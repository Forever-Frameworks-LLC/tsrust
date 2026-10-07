import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertDestructuringDefaultsSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='destructuring-defaults'||row.id!=='destructuring_defaults')throw Error('Destructuring defaults family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Destructuring defaults fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete lazy destructuring default scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Destructuring defaults denominator');
 for(const text of ['while (i < n)','function describe({label="none",amount=7}:Row)','const [text=fallback(trace,input.repeat(1)),missing="tail"]=values','const {amount=8}=row','text.slice(),missing.slice(),describe(row),String(amount),String(trace.length)','return output;'])if(!containsCodeSnippet(source, text))throw Error('Actual lazy default patterns and complete retention');
 if(containsCodeSnippet(source, 'clone('))throw Error('Hidden clone');
}
