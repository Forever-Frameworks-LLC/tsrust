import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {stringFromCodesIds} from './string-from-codes-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertStringFromCodesSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='string-from-codes'||!stringFromCodesIds.includes(row.id))throw Error('String from codes source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('String from codes source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete string from codes scenario iteration')throw Error('String from codes scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('String from codes observer protocol');
 if(!containsCodeSnippet(source, 'while (i < n)')||!containsCodeSnippet(source, 'return output;')||!containsCodeSnippet(source, 'output.push(['))throw Error('Actual string from codes scenario and complete retention');
 if((source.match(/String\.fromCodePoint\(/g)??[]).length!==2||(source.match(/String\.fromCharCode\(/g)??[]).length!==1||!containsCodeSnippet(source, 'String.fromCodePoint(...codes)'))throw Error('Actual String.fromCodePoint and String.fromCharCode calls');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let points=0,units=0,spreads=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='String'){
    if(node.expression.name.text==='fromCodePoint'){points++;if(node.arguments.some(ts.isSpreadElement))spreads++;}
    if(node.expression.name.text==='fromCharCode'&&node.arguments.length===3)units++;}
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||points!==2||units!==1||spreads!==1)throw Error('Actual typed string from codes calls');
 }
}
