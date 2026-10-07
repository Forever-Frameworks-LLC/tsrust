import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const ts=createRequire(import.meta.url)('typescript');
export function assertRestParameterSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='rest-parameters')throw Error('Rest source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Rest source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete rest-parameter scenario iteration')throw Error('Rest scenario denominator');
 if(row.resultEncoding!==(row.id==='rest_parameter_number_pack'?'ieee-f64-tree-v1':'array-shape-scalar-v1'))throw Error('Rest complete observer protocol');
 if(!containsCodeSnippet(source, 'while (i < n)')||!containsCodeSnippet(source, 'return output;')||!containsCodeSnippet(source, 'output.push(')||!containsCodeSnippet(source, '...'))throw Error('Actual rest scenario/complete retention');
 if(kind==='typescript'){
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let rest=0,workload;
  function visit(node){if(ts.isParameter(node)&&node.dotDotDotToken){if(!node.type||!ts.isArrayTypeNode(node.type))throw Error('Homogeneous rest array annotation');rest++;}if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||!rest)throw Error('Actual typed rest-parameter syntax');
 }
}
