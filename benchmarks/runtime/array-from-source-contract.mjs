import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {arrayFromIds} from './array-from-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertArrayFromSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='array-from'||!arrayFromIds.includes(row.id))throw Error('Array.from source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Array.from source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete Array.from scenario iteration')throw Error('Array.from scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('Array.from observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(Array.from('))throw Error('Actual Array.from call and complete retention');
 const mapped=row.id.includes('_map_');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,mappers=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='from'&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='Array'){calls++;if(node.arguments.length===2&&ts.isArrowFunction(node.arguments[1]))mappers++;}
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==1||mappers!==(mapped?1:0))throw Error('Actual typed Array.from call');
 } else if(mapped!==/Array\.from\([a-z]+, \(/.test(source))throw Error('Actual Array.from mapping function');
}
