import {containsCodeSnippet} from './source-contract-tokens.mjs';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
const ts=createRequire(import.meta.url)('typescript');
export function assertOptionalDefaultParameterSource(source,row,kind){
 if(!['native','typescript'].includes(kind)||row.family!=='optional-default-parameters')throw Error('Optional/default source kind/family');
 if(createHash('sha256').update(source).digest('hex')!==(kind==='native'?row.sourceSha256:row.counterpartSha256))throw Error('Optional/default source fingerprint');
 if(row.operationsPerIteration!==1||row.iterations!==2048||row.operationUnit!=='One complete optional/default-parameter scenario iteration')throw Error('Optional/default scenario denominator');
 if(row.resultEncoding!==(row.id==='default_parameter_number_value'?'ieee-f64-tree-v1':'array-shape-scalar-v1'))throw Error('Optional/default complete observer protocol');
 if(!containsCodeSnippet(source, 'while (i < n)')||!containsCodeSnippet(source, 'return output;')||!containsCodeSnippet(source, 'output.push('))throw Error('Complete scenario result/effect retention');
 if(kind==='typescript'){
  const file=ts.createSourceFile(row.id+'.ts',source,ts.ScriptTarget.Latest,true,ts.ScriptKind.TS);let defaults=0,optional=0,workload;
  function visit(node){if(ts.isParameter(node)){if(node.initializer)defaults++;if(node.questionToken)optional++;}if(ts.isFunctionDeclaration(node)&&node.name?.text==='workload')workload=node;ts.forEachChild(node,visit);}visit(file);
  if(file.parseDiagnostics.length||!workload||workload.parameters.length!==3||!workload.type||!(defaults+optional))throw Error('Actual typed optional/default parameter syntax');
 }
}
