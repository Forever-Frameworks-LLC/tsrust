import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {stringCodeAtIds} from './string-code-at-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertStringCodeAtSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='string-code-at'||!stringCodeAtIds.includes(row.id))throw Error('String code at source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('String code at source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete string code at scenario iteration')throw Error('String code at scenario denominator');
 if(row.resultEncoding!=='ieee-f64-tree-v1')throw Error('String code at observer protocol');
 if(!containsCodeSnippet(source, 'while (i < n)')||!containsCodeSnippet(source, 'return output;')||!containsCodeSnippet(source, 'output.push(['))throw Error('Actual string code at scenario and complete retention');
 if((source.match(/text\.codePointAt\(/g)??[]).length!==3||(source.match(/text\.charCodeAt\(/g)??[]).length!==2)throw Error('Actual codePointAt and charCodeAt calls');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let points=0,units=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='text'){
    if(node.expression.name.text==='codePointAt')points++;
    if(node.expression.name.text==='charCodeAt'&&node.arguments.length===1)units++;}
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||points!==3||units!==2)throw Error('Actual typed string code at calls');
 }
}
