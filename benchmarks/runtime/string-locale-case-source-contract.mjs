import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {stringLocaleCaseIds} from './string-locale-case-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertStringLocaleCaseSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='string-locale-case'||!stringLocaleCaseIds.includes(row.id))throw Error('String locale case source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('String locale case source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete string locale case scenario iteration')throw Error('String locale case scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('String locale case observer protocol');
 if(!source.includes('while (i < n)')||!source.includes('return output;')||!source.includes('output.push(['))throw Error('Actual string locale case scenario and complete retention');
 if((source.match(/text\.toLocaleLowerCase\(\)/g)??[]).length!==1||(source.match(/text\.toLocaleUpperCase\(\)/g)??[]).length!==1)throw Error('Actual toLocaleLowerCase and toLocaleUpperCase calls');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let calls=0,workload;
  function visit(node){
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='text'&&['toLocaleLowerCase','toLocaleUpperCase'].includes(node.expression.name.text)&&node.arguments.length===0)calls++;
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||calls!==2)throw Error('Actual typed string locale case calls');
 }
}
