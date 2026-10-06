import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {stringAtIds} from './string-at-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertStringAtSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='string-at'||!stringAtIds.includes(row.id))throw Error('String at source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('String at source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete string at scenario iteration')throw Error('String at scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('String at observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(['))throw Error('Actual at() scenario and complete retention');
 if(kind==='native'){if((source.match(/text\.at\(/g)??[]).length!==2)throw Error('Two native string at() selections');}
 else {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let selections=0,points=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='at'&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='letters')selections++;
   if(ts.isSpreadElement(node)&&ts.isIdentifier(node.expression)&&node.expression.text==='text')points++;
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||selections!==2||points!==1)throw Error('Actual typed code-point selection');
 }
}
