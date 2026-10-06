import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {numberToStringIds} from './number-to-string-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertNumberToStringSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='number-to-string'||!numberToStringIds.includes(row.id))throw Error('Number toString source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Number toString source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete number toString scenario iteration')throw Error('Number toString scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('Number toString observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(['))throw Error('Actual toString scenario and complete retention');
 if((source.match(/\.toString\(/g)??[]).length!==5)throw Error('Five actual toString calls');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,radixes=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='toString'){calls++;if(node.arguments.length===1)radixes++;}
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==5||radixes!==4)throw Error('Actual typed toString calls');
 }
}
