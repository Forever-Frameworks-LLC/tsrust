import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {arrayKeysValuesIds} from './array-keys-values-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertArrayKeysValuesSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='array-keys-values'||!arrayKeysValuesIds.includes(row.id))throw Error('Array keys/values source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Array keys/values source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete array keys/values scenario iteration')throw Error('Array keys/values scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('Array keys/values observer protocol');
 const method=row.id==='array_keys_int'?'keys':'values';
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('values.'+method+'()'))throw Error('Actual '+method+'() call and complete retention');
 if(kind==='native'){if(!source.includes('output.push(values.'+method+'());'))throw Error('Native '+method+'() result retention');}
 else {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let collected=0,workload;
  function visit(node){
   if(ts.isSpreadElement(node)&&ts.isCallExpression(node.expression)&&ts.isPropertyAccessExpression(node.expression.expression)&&node.expression.expression.name.text===method&&node.expression.arguments.length===0)collected++;
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||collected!==1)throw Error('Actual typed '+method+'() collection');
 }
}
