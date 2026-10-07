import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {mathF16roundIds} from './math-f16round-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertMathF16roundSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='math-f16round'||!mathF16roundIds.includes(row.id))throw Error('Math.f16round source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Math.f16round source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete Math.f16round scenario iteration')throw Error('Math.f16round scenario denominator');
 if(row.resultEncoding!=='ieee-f64-tree-v1')throw Error('Math.f16round observer protocol');
 if(!containsCodeSnippet(source, 'while (i < n)')||!containsCodeSnippet(source, 'return output;')||!containsCodeSnippet(source, 'output.push(['))throw Error('Actual Math.f16round scenario and complete retention');
 if((source.match(/Math\.f16round\(/g)??[]).length!==5)throw Error('Five actual Math.f16round calls');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='f16round'&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='Math'&&node.arguments.length===1)calls++;
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==5)throw Error('Actual typed Math.f16round calls');
 }
}
