import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {arrayLocaleStringIds} from './array-locale-string-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertArrayLocaleStringSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='array-locale-string'||!arrayLocaleStringIds.includes(row.id))throw Error('array toLocaleString source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('array toLocaleString source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete array toLocaleString scenario iteration')throw Error('array toLocaleString scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('array toLocaleString observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(['))throw Error('Actual array toLocaleString scenario and complete retention');
 const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,workload;
 function visit(node){
  if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.name.text==='toLocaleString'&&node.arguments.length===0)calls++;
  if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);
 }
 visit(file);
 if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==8)throw Error('Actual typed array toLocaleString calls');
}
