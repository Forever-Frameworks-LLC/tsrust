import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {numberParseIntIds} from './number-parse-int-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertNumberParseIntSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='number-parse-int'||!numberParseIntIds.includes(row.id))throw Error('Number.parseInt source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Number.parseInt source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete Number.parseInt scenario iteration')throw Error('Number.parseInt scenario denominator');
 if(row.resultEncoding!=='ieee-f64-tree-v1')throw Error('Number.parseInt observer protocol');
 if(!containsCodeSnippet(source, 'while (i < n)')||!containsCodeSnippet(source, 'return output;')||!containsCodeSnippet(source, 'output.push(['))throw Error('Actual Number.parseInt scenario and complete retention');
 if((source.match(/Number\.parseInt\(/g)??[]).length!==5)throw Error('Five actual Number.parseInt calls');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,radixes=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='parseInt'&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='Number'){calls++;if(node.arguments.length===2)radixes++;}
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==5||radixes!==2)throw Error('Actual typed Number.parseInt calls');
 }
}
