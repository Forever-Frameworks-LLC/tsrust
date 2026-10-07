import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
export function assertScalarUnaryPlusSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.id!=='scalar_unary_plus'||row.family!=='scalar-unary-plus')throw Error('Unary plus family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Unary plus fingerprint');
 if(row.iterations!==2048||row.operationsPerIteration!==1||row.operationUnit!=='One complete scalar unary conversion scenario'||row.resultEncoding!=='array-shape-scalar-v1')throw Error('Unary plus denominator');
 const observed=kind==='typescript'?source.replaceAll('+(nullable as any)','+nullable').replaceAll('+(null as any)','+null').replaceAll('+(undefined as any)','+undefined'):source;
 for(const text of ['String(+text)','String(+flag)','String(+null)','String(+undefined)','String(+nullable)','String(Number(nullable))','String(Number.isNaN(+input))','return output;'])if(!containsCodeSnippet(observed,text))throw Error('Unary plus scenario: '+text);
}
