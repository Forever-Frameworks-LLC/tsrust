import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {arrayLiteralSpreadIds} from './array-literal-spread-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertArrayLiteralSpreadSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='array-literal-spread'||!arrayLiteralSpreadIds.includes(row.id))throw Error('Array-literal spread source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Array-literal spread source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete array-literal spread scenario iteration')throw Error('Array-literal spread scenario denominator');
 if(row.resultEncoding!==(row.id.endsWith('_number')?'ieee-f64-tree-v1':'array-shape-scalar-v1'))throw Error('Array-literal spread observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(')||!source.includes('...'))throw Error('Actual array-literal spread and complete retention');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let spreads=0,workload;
  function visit(node){if(ts.isArrayLiteralExpression(node))for(const item of node.elements)if(ts.isSpreadElement(item))spreads++;if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||!spreads)throw Error('Actual typed array-literal spread syntax');
 }
}
