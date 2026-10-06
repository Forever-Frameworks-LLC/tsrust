import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {objectIsIds} from './object-is-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertObjectIsSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='object-is'||!objectIsIds.includes(row.id))throw Error('Object.is source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Object.is source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete Object.is scenario iteration')throw Error('Object.is scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('Object.is observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(['))throw Error('Actual Object.is scenario and complete retention');
 if((source.match(/Object\.is\(/g)??[]).length!==6)throw Error('Six actual Object.is calls');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='is'&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='Object'&&node.arguments.length===2)calls++;
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==6)throw Error('Actual typed Object.is calls');
 }
}
