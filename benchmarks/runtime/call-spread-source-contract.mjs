import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const ts=createRequire(import.meta.url)('typescript');
export function assertCallSpreadSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='call-spread')throw Error('Call-spread source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Call-spread source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete call-spread scenario iteration')throw Error('Call-spread scenario denominator');
 if(row.resultEncoding!==(row.id==='call_spread_math_number'?'ieee-f64-tree-v1':'array-shape-scalar-v1'))throw Error('Call-spread complete observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(')||!source.includes('...'))throw Error('Actual spread scenario/complete retention');
 if(kind==='typescript'){
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let spreads=0,workload;
  function visit(node){if(ts.isCallExpression(node))for(const arg of node.arguments)if(ts.isSpreadElement(arg))spreads++;if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||!spreads)throw Error('Actual typed call-spread syntax');
 }
}
