import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import {destructuringIds} from './destructuring-oracle.mjs';
const ts=createRequire(import.meta.url)('typescript');
export function assertDestructuringSource(source,row,kind) {
 if(!['native','typescript'].includes(kind)||row.family!=='destructuring'||!destructuringIds.includes(row.id))throw Error('Destructuring source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Destructuring source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete destructuring scenario iteration')throw Error('Destructuring scenario denominator');
 if(row.resultEncoding!=='array-shape-scalar-v1')throw Error('Destructuring observer protocol');
 if(!containsCodeSnippet(source, 'while (i < n)')||!containsCodeSnippet(source, 'return output;')||!containsCodeSnippet(source, 'output.push(['))throw Error('Actual destructuring scenario and complete retention');
 for(const needle of ['const [count, word] = pair(base, input);','const { x, y, label } = point;','for (const [name, amount] of pairs)'])
  if(!containsCodeSnippet(source, needle))throw Error('Actual destructuring pattern: '+needle);
 if(kind==='typescript') {
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let arrays=0,objects=0,workload;
  function visit(node){
   if(ts.isVariableDeclaration(node)&&ts.isArrayBindingPattern(node.name)&&node.name.elements.length===2)arrays++;
   if(ts.isVariableDeclaration(node)&&ts.isObjectBindingPattern(node.name)&&node.name.elements.length===3)objects++;
   if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}
  visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||arrays!==2||objects!==1)throw Error('Actual typed destructuring patterns');
 }
}
