import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {objectValuesIds} from './object-values-entries-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertObjectValuesSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='object-values-entries'||!objectValuesIds.includes(row.id))throw Error('Object values source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Object values source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete record values and entries scenario iteration')throw Error('Object values scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('Object values observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(['))throw Error('Actual object values scenario and complete retention');
 for(const needle of ['{ gamma: true, alpha: base, beta: input.slice() }','{ beta: input.slice(), gamma: false, alpha: base }','const values = Object.values(row);','for (const value of values)','const lengths = Object.values(sides);','for (const pair of Object.entries(row))','Object.entries(sides).length'])
  if(!source.includes(needle))throw Error('Actual object values operation: '+needle);
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let values=0,entries=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='Object'&&node.arguments.length===1){
    if(node.expression.name.text==='values')values++;if(node.expression.name.text==='entries')entries++;}
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||values!==2||entries!==2)throw Error('Actual typed object values calls');
 }
}
