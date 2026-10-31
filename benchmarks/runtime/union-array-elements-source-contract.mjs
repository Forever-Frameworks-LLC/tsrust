import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {unionArrayIds} from './union-array-elements-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertUnionArraySource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='union-array-elements'||!unionArrayIds.includes(row.id))throw Error('Union array source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Union array source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete union array scenario iteration')throw Error('Union array scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('Union array observer protocol');
 if(!containsCodeSnippet(source, 'while (i < n)')||!containsCodeSnippet(source, 'return output;')||!containsCodeSnippet(source, 'output.push(['))throw Error('Actual union array scenario and complete retention');
 for(const needle of ['const values: (int | string | null)[] = [base, input.repeat(mode), null];','for (const value of values)','typeof value === "number"','values.join("|")','values.includes(null)','values.indexOf(base + 1)','values.at(-1)','values.filter((value) => typeof value === "string")'])
  if(!containsCodeSnippet(source, needle))throw Error('Actual union array operation: '+needle);
 if(row.id==='union_array_heap'&&!containsCodeSnippet(source,'let extra = 1; while (extra <= 16) { values.push(base + extra); extra++; }'))throw Error('Heap union growth control');
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let unionArray=false,calls=0,workload;
  function visit(node){
   if(ts.isArrayTypeNode(node)&&ts.isParenthesizedTypeNode(node.elementType)&&ts.isUnionTypeNode(node.elementType.type)&&node.elementType.type.types.length===3)unionArray=true;
   if(ts.isCallExpression(node)&&ts.isPropertyAccessExpression(node.expression)&&ts.isIdentifier(node.expression.expression)&&node.expression.expression.text==='values'&&['push','join','includes','indexOf','at','filter'].includes(node.expression.name.text))calls++;
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||!unionArray||calls!==(row.id==='union_array_heap'?8:7))throw Error('Actual typed union array calls');
 }
}
