import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {mathRandomIds} from './math-random-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertMathRandomSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='math-random'||!mathRandomIds.includes(row.id))throw Error('Math.random source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Math.random source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete Math.random scenario iteration')throw Error('Math.random scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('Math.random observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(['))throw Error('Actual Math.random scenario and complete retention');
 if((source.match(/Math\.random\(\)/g)??[]).length!==2)throw Error('Two actual Math.random calls');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='random'&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='Math'&&node.arguments.length===0)calls++;
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==2)throw Error('Actual typed Math.random calls');
 }
}
