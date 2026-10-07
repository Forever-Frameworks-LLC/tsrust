import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {arrayEntriesIds} from './array-entries-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertArrayEntriesSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='array-entries'||!arrayEntriesIds.includes(row.id))throw Error('Array entries source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Array entries source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete owning array entry snapshot and independent mutation scenario iteration')throw Error('Array entries denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('Array entries observer protocol');
 for(const text of ['while (i < n)','return output;','output.push([','source.entries()','pairs[0][1]'])if(!containsCodeSnippet(source, text))throw Error('Actual array entries scenario and full retention');
 if(kind==='typescript'&&(!containsCodeSnippet(source, 'Array.from(source.entries(),')||!containsCodeSnippet(source, 'pair[1].slice()')&&!containsCodeSnippet(source, 'values:pair[1].values.slice()')))throw Error('Explicit independent counterpart snapshot');
 const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let entries=0,workload;
 function visit(node){if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='entries')entries++;if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
 visit(file);if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||entries!==1)throw Error('Actual typed entries call');
}
