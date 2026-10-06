import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {stringRawIds} from './string-raw-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertStringRawSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='string-raw'||!stringRawIds.includes(row.id))throw Error('String.raw source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('String.raw source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete String.raw scenario iteration')throw Error('String.raw scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('String.raw observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(['))throw Error('Actual String.raw scenario and complete retention');
 const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,spreads=0,workload;
 function visit(node){
  if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&node.expression.expression.getText(file)==='String'&&node.expression.name.text==='raw'){calls++;spreads+=node.arguments.filter(ts.isSpreadElement).length;}
  if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);
 }
 visit(file);
 if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==5||spreads!==2)throw Error('Actual typed String.raw calls');
}
