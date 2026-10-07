import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {stringNormalizeIds} from './string-normalize-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertStringNormalizeSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='string-normalize'||!stringNormalizeIds.includes(row.id))throw Error('string normalization source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('string normalization source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete string normalization scenario iteration')throw Error('string normalization scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('string normalization observer protocol');
 if(!containsCodeSnippet(source, 'while (i < n)')||!containsCodeSnippet(source, 'return output;')||!containsCodeSnippet(source, 'output.push(['))throw Error('Actual string normalization scenario and complete retention');
 const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,workload;
 function visit(node){
  if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='normalize')calls++;
  if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);
 }
 visit(file);
 if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==7)throw Error('Actual typed string normalization calls');
}
