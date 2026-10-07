import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {numberLocaleStringIds} from './number-locale-string-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertNumberLocaleStringSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='number-locale-string'||!numberLocaleStringIds.includes(row.id))throw Error('numeric toLocaleString source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('numeric toLocaleString source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete numeric toLocaleString scenario iteration')throw Error('numeric toLocaleString scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('numeric toLocaleString observer protocol');
 if(!containsCodeSnippet(source, 'while (i < n)')||!containsCodeSnippet(source, 'return output;')||!containsCodeSnippet(source, 'output.push(['))throw Error('Actual numeric toLocaleString scenario and complete retention');
 const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,workload;
 function visit(node){
  if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='toLocaleString'&&node.arguments.length===0)calls++;
  if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);
 }
 visit(file);
 if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==5)throw Error('Actual typed numeric toLocaleString calls');
}
